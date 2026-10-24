use std::sync::LazyLock;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub use crate::models::{DownloadProgress, DownloadTaskSpec};

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]").expect("Invalid ANSI regex")
});

static PROGRESS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?:progress:\s*)?(\d+(?:\.\d+)?)%\s*(?:speed:\s*)?(-?[0-9.]+\s*[kmgb]?b(?:/s|ps))\s*(?:eta:\s*)?(\d+:\d{2}:\d{2}|--:--:--)"#
    ).expect("Invalid progress regex")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub threads: u32,
    pub tmp_dir: String,
    pub retry_count: u32,
    pub binary_path: Option<String>,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            threads: 16,
            tmp_dir: "/tmp/bingeget_chunks".to_string(),
            retry_count: 5,
            binary_path: None,
        }
    }
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

    // Disable log file output to avoid permission errors when writing to /usr/bin/Logs
    args.push("--no-log".to_string());

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
    let clean_line = if line.contains('\x1b') {
        ANSI_RE.replace_all(line, "").to_string()
    } else {
        line.to_string()
    };

    if let Some(caps) = PROGRESS_RE.captures(&clean_line) {
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
