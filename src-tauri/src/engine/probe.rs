use std::collections::HashMap;
use std::process::Command;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::models::StreamDetails;

static STREAM_INF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"#EXT-X-STREAM-INF:[^\n]*"#).expect("Invalid stream inf regex")
});

static BANDWIDTH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"BANDWIDTH=(\d+)"#).expect("Invalid bandwidth regex")
});

static RESOLUTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"RESOLUTION=(\d+x\d+)"#).expect("Invalid resolution regex")
});

static BITRATE_HINT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(?:^|[/_\.-])(\d{3,5})k(?:bps)?(?:[/_\.-]|$)"#)
        .expect("Invalid bitrate hint regex")
});

static PROBE_CACHE: LazyLock<RwLock<HashMap<(String, String, String, String), (Instant, StreamDetails)>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes

pub async fn get_cached_probe_details(
    sid: &str,
    drama_id: &str,
    ep_id: &str,
    channel: &str,
) -> Option<StreamDetails> {
    let key = (
        sid.to_string(),
        drama_id.to_string(),
        ep_id.to_string(),
        channel.to_string(),
    );
    let cache_read = PROBE_CACHE.read().await;
    if let Some((instant, ref details)) = cache_read.get(&key) {
        if instant.elapsed() < CACHE_TTL {
            return Some(details.clone());
        }
    }
    None
}

pub async fn set_cached_probe_details(
    sid: &str,
    drama_id: &str,
    ep_id: &str,
    channel: &str,
    details: StreamDetails,
) {
    let key = (
        sid.to_string(),
        drama_id.to_string(),
        ep_id.to_string(),
        channel.to_string(),
    );
    let mut cache_write = PROBE_CACHE.write().await;
    cache_write.insert(key, (Instant::now(), details));
}

pub async fn clear_probe_cache() {
    let mut cache_write = PROBE_CACHE.write().await;
    cache_write.clear();
}

fn format_number_commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    out
}

fn resolution_with_tier(width: u32, height: u32) -> String {
    let tier = match height {
        h if h >= 2160 => "4K",
        h if h >= 1080 => "1080p",
        h if h >= 720 => "720p",
        h if h >= 480 => "480p",
        h => return format!("{}x{}", width, h),
    };
    format!("{}x{} ({})", width, height, tier)
}

pub fn parse_m3u8_master_tags(m3u8_content: &str) -> Option<StreamDetails> {
    for cap in STREAM_INF_RE.captures_iter(m3u8_content) {
        let stream_match = cap.get(0)?;
        let line = stream_match.as_str();

        let bandwidth = BANDWIDTH_RE
            .captures(line)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u64>().ok())?;

        let res_str = RESOLUTION_RE
            .captures(line)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())?;

        let parts: Vec<&str> = res_str.split('x').collect();
        let (width, height) = if parts.len() == 2 {
            let w = parts[0].parse::<u32>().unwrap_or(0);
            let h = parts[1].parse::<u32>().unwrap_or(0);
            (w, h)
        } else {
            (0, 0)
        };

        let formatted_res = if width > 0 && height > 0 {
            resolution_with_tier(width, height)
        } else {
            res_str
        };

        let mut kbps = (bandwidth + 500) / 1000;

        // If bandwidth yields 0 or seems unrealistically low (< 50 kbps for a video stream),
        // try to extract a bitrate hint from the subsequent URI line or the stream-inf line.
        if kbps < 50 {
            let rest = &m3u8_content[stream_match.end()..];
            let next_line = rest
                .lines()
                .map(|l| l.trim())
                .find(|l| !l.is_empty() && !l.starts_with('#'))
                .unwrap_or("");

            if let Some(hint_cap) = BITRATE_HINT_RE
                .captures(next_line)
                .or_else(|| BITRATE_HINT_RE.captures(line))
            {
                if let Some(val) = hint_cap.get(1).and_then(|m| m.as_str().parse::<u64>().ok()) {
                    kbps = val;
                }
            }

            // If still unrealistically low, provide a reasonable baseline estimate from resolution
            if kbps < 50 && height > 0 {
                kbps = match height {
                    h if h >= 2160 => 6000,
                    h if h >= 1080 => 2500,
                    h if h >= 720 => 1200,
                    h if h >= 480 => 800,
                    _ => 500,
                };
            }
        }

        let formatted_bitrate = if kbps > 0 {
            format!("{} kbps", format_number_commas(kbps))
        } else {
            "未知".to_string()
        };

        return Some(StreamDetails {
            resolution: formatted_res,
            bitrate: formatted_bitrate,
            video_codec: "H.264 / AVC".to_string(),
            audio_codec: "AAC".to_string(),
            fps: Some(30),
            duration_secs: None,
        });
    }

    None
}

#[derive(Deserialize)]
struct FfprobeStream {
    codec_name: Option<String>,
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    streams: Option<Vec<FfprobeStream>>,
    format: Option<FfprobeFormat>,
}

pub fn parse_ffprobe_json_output(json_str: &str) -> Result<StreamDetails> {
    let output: FfprobeOutput = serde_json::from_str(json_str)
        .map_err(|e| anyhow!("Failed to parse ffprobe json: {}", e))?;

    let streams = output.streams.unwrap_or_default();

    let mut video_stream: Option<&FfprobeStream> = None;
    let mut audio_stream: Option<&FfprobeStream> = None;

    for s in &streams {
        if s.codec_type.as_deref() == Some("video") && video_stream.is_none() {
            video_stream = Some(s);
        } else if s.codec_type.as_deref() == Some("audio") && audio_stream.is_none() {
            audio_stream = Some(s);
        }
    }

    let video = video_stream.ok_or_else(|| anyhow!("No video stream found in ffprobe output"))?;

    let width = video.width.unwrap_or(0);
    let height = video.height.unwrap_or(0);
    let resolution = if width > 0 && height > 0 {
        resolution_with_tier(width, height)
    } else {
        "Unknown".to_string()
    };

    let video_codec = video.codec_name.clone().unwrap_or_else(|| "h264".to_string());
    let audio_codec = audio_stream
        .and_then(|a| a.codec_name.clone())
        .unwrap_or_else(|| "aac".to_string());

    let fps = video.r_frame_rate.as_deref().and_then(|rate| {
        let parts: Vec<&str> = rate.split('/').collect();
        if parts.len() == 2 {
            let num: f32 = parts[0].parse().ok()?;
            let den: f32 = parts[1].parse().ok()?;
            if den > 0.0 {
                return Some((num / den).round() as u32);
            }
        }
        None
    });

    let raw_bitrate = video
        .bit_rate
        .as_deref()
        .and_then(|b| b.parse::<u64>().ok())
        .unwrap_or(2_000_000);

    let kbps = raw_bitrate / 1000;
    let bitrate = format!("{} kbps", format_number_commas(kbps));

    let duration_secs = output
        .format
        .as_ref()
        .and_then(|f| f.duration.as_deref())
        .and_then(|d| d.parse::<f64>().ok());

    Ok(StreamDetails {
        resolution,
        bitrate,
        video_codec,
        audio_codec,
        fps,
        duration_secs,
    })
}

pub fn probe_with_ffprobe(
    ffprobe_path: &str,
    stream_url: &str,
    referer: Option<&str>,
) -> Result<StreamDetails> {
    let mut cmd = Command::new(ffprobe_path);
    cmd.arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg("-show_format")
        .arg("-allowed_segment_extensions")
        .arg("ALL")
        .arg("-extension_picky")
        .arg("0")
        .arg("-analyzeduration")
        .arg("3000000")
        .arg("-probesize")
        .arg("3000000");

    let mut headers = "User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36\r\n".to_string();
    if let Some(ref_url) = referer {
        headers.push_str(&format!("Referer: {}\r\n", ref_url));
    }
    cmd.arg("-headers").arg(headers);

    cmd.arg(stream_url);

    let output = cmd.output().map_err(|e| anyhow!("Failed to spawn ffprobe: {}", e))?;
    if !output.status.success() {
        return Err(anyhow!("ffprobe exited with non-zero status"));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_json_output(&json_str)
}

pub async fn probe_stream_core(
    client: &reqwest::Client,
    drama_id: &str,
    ep_id: &str,
    channel: &str,
    ffprobe_path: Option<&str>,
    source_id: Option<&str>,
) -> Result<StreamDetails, String> {
    let session = crate::engine::stream::prepare_episode_stream(
        client,
        drama_id,
        ep_id,
        channel,
        source_id,
        ffprobe_path,
    )
    .await?;

    Ok(session.stream_details)
}
