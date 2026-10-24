use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTaskSpec {
    pub m3u8_url: String,
    pub custom_title: String,
    pub season: u32,
    pub episode: u32,
    pub save_dir: String,
    pub referer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub threads: u32,
    pub tmp_dir: String,
    pub retry_count: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            threads: 16,
            tmp_dir: "/tmp/bingeget_chunks".to_string(),
            retry_count: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
}

pub fn build_download_args(spec: &DownloadTaskSpec, config: &DownloadConfig) -> Vec<String> {
    let mut args = Vec::new();

    // Positional URL
    args.push(spec.m3u8_url.clone());

    // Save directory
    args.push("--save-dir".to_string());
    args.push(spec.save_dir.clone());

    // Jellyfin naming: {custom_title} - S{season:02d}E{episode:02d}
    let save_name = format!(
        "{} - S{:02}E{:02}",
        spec.custom_title.trim(),
        spec.season,
        spec.episode
    );
    args.push("--save-name".to_string());
    args.push(save_name);

    // Temp chunks directory
    args.push("--tmp-dir".to_string());
    args.push(config.tmp_dir.clone());

    // Threads
    args.push("--thread-count".to_string());
    args.push(config.threads.to_string());

    // Retry count
    args.push("--download-retry-count".to_string());
    args.push(config.retry_count.to_string());

    // Auto select best video/audio
    args.push("--auto-select".to_string());

    // Output container format
    args.push("-M".to_string());
    args.push("format=mp4".to_string());

    // Clean up temporary chunks
    args.push("--del-after-done".to_string());

    // Headers
    args.push("--header".to_string());
    args.push("User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());

    if !spec.referer.is_empty() {
        args.push("--header".to_string());
        args.push(format!("Referer: {}", spec.referer));
    }

    args
}

pub fn parse_progress_line(line: &str) -> Option<DownloadProgress> {
    // Matches patterns like:
    // Progress: 45.2%  Speed: 18.50MB/s  ETA: 00:00:32
    // or [12:34:56] 80.5% 12.3MB/s 00:01:10
    let re = Regex::new(
        r#"(?:Progress:\s*)?(\d+(?:\.\d+)?)%\s*(?:Speed:\s*)?([0-9.]+\s*[kKmMgG]?[bB]/s)\s*(?:ETA:\s*)?(\d{2}:\d{2}:\d{2})"#
    ).ok()?;

    if let Some(caps) = re.captures(line) {
        let percent: f32 = caps.get(1)?.as_str().parse().ok()?;
        let speed = caps.get(2)?.as_str().to_string();
        let eta = caps.get(3)?.as_str().to_string();

        return Some(DownloadProgress {
            percent,
            speed,
            eta,
        });
    }

    None
}
