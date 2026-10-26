use crate::engine::probe::{
    get_cached_probe_details, parse_m3u8_master_tags, probe_with_ffprobe, set_cached_probe_details,
};
use crate::engine::proxy::{build_proxy_url, cache_playlist, get_cached_playlist, start_streaming_proxy};
use crate::extractors::global_registry;
use crate::models::{StreamDetails, StreamSession};

pub async fn prepare_episode_stream(
    client: &reqwest::Client,
    drama_id: &str,
    ep_id: &str,
    channel: &str,
    source_id: Option<&str>,
    ffprobe_path: Option<&str>,
) -> Result<StreamSession, String> {
    let drama_id = drama_id.trim();
    let ep_id = ep_id.trim();
    let channel = channel.trim();

    if drama_id.is_empty() {
        return Err("Drama ID cannot be empty".to_string());
    }
    if ep_id.is_empty() {
        return Err("Episode ID cannot be empty".to_string());
    }
    if channel.is_empty() {
        return Err("Channel ID cannot be empty".to_string());
    }

    let sid = source_id.unwrap_or("chinaq");
    let registry = global_registry();
    let extractor = registry
        .get_by_source_id(sid)
        .ok_or_else(|| format!("Extractor not found for source: {}", sid))?;

    // 1. Resolve upstream M3U8 URL and build Referer header
    let m3u8_url = extractor
        .fetch_m3u8_for_episode(client, drama_id, ep_id, channel)
        .await
        .map_err(|e| format!("Failed to fetch M3U8 URL: {}", e))?;

    let referer = extractor.build_stream_referer(drama_id, ep_id, channel, None);

    // 2. Ensure streaming proxy server is running
    let proxy_port = start_streaming_proxy(client.clone())
        .await
        .map_err(|e| e.to_string())?;

    // 3. Fetch upstream M3U8 playlist ONCE and cache it for both proxy and probing
    let m3u8_content = match get_cached_playlist(&m3u8_url).await {
        Some(content) => content,
        None => {
            let resp = client
                .get(&m3u8_url)
                .header(
                    "User-Agent",
                    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                )
                .header("Referer", &referer)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch upstream M3U8 playlist: {}", e))?;

            if !resp.status().is_success() {
                return Err(format!(
                    "Upstream M3U8 playlist request returned status: {}",
                    resp.status()
                ));
            }

            let text = resp
                .text()
                .await
                .map_err(|e| format!("Failed to read upstream M3U8 playlist body: {}", e))?;

            // Store in proxy cache so that local streaming proxy serves it without refetching
            cache_playlist(&m3u8_url, &text).await;
            text
        }
    };

    // 4. Probe stream telemetry (using the already fetched playlist or cache)
    let stream_details = if let Some(cached) = get_cached_probe_details(sid, drama_id, ep_id, channel).await {
        cached
    } else {
        let details = if let Some(master_details) = parse_m3u8_master_tags(&m3u8_content) {
            master_details
        } else if let Some(probe_bin) = ffprobe_path {
            probe_with_ffprobe(probe_bin, &m3u8_url, Some(&referer))
                .unwrap_or_else(|_| StreamDetails {
                    resolution: "未知 (需 ffprobe)".to_string(),
                    bitrate: "未知".to_string(),
                    video_codec: "未知".to_string(),
                    audio_codec: "未知".to_string(),
                    fps: None,
                    duration_secs: None,
                })
        } else {
            StreamDetails {
                resolution: "未知 (需 ffprobe)".to_string(),
                bitrate: "未知".to_string(),
                video_codec: "未知".to_string(),
                audio_codec: "未知".to_string(),
                fps: None,
                duration_secs: None,
            }
        };

        set_cached_probe_details(sid, drama_id, ep_id, channel, details.clone()).await;
        details
    };

    // 5. Construct local streaming proxy URL
    let proxy_url = Some(build_proxy_url(proxy_port, &m3u8_url, &referer));

    Ok(StreamSession {
        m3u8_url,
        referer,
        proxy_url,
        stream_details,
    })
}
