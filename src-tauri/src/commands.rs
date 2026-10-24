use std::path::Path;
use crate::extractors::chinaq::ChinaQExtractor;
use crate::extractors::SiteExtractor;
use crate::models::{SeriesMetadata, SystemEnvironment};

pub fn detect_system_environment_core() -> SystemEnvironment {
    let check_path = |name: &str, preferred: &str| -> Option<String> {
        if Path::new(preferred).is_file() {
            return Some(preferred.to_string());
        }
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                let full = dir.join(name);
                if full.is_file() {
                    return Some(full.to_string_lossy().to_string());
                }
            }
        }
        None
    };

    let nm3u8dl_path = check_path("n-m3u8dl-re", "/usr/bin/n-m3u8dl-re");
    let ffmpeg_path = check_path("ffmpeg", "/usr/bin/ffmpeg");
    let is_ready = nm3u8dl_path.is_some() && ffmpeg_path.is_some();

    SystemEnvironment {
        nm3u8dl_path,
        ffmpeg_path,
        is_ready,
    }
}

pub async fn parse_series_metadata_core(
    client: &reqwest::Client,
    url: &str,
) -> Result<SeriesMetadata, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    let chinaq = ChinaQExtractor::new();
    if chinaq.can_handle(url) {
        return chinaq
            .fetch_series_metadata(client, url)
            .await
            .map_err(|e| e.to_string());
    }

    Err(format!("Unsupported website URL: {}", url))
}

pub async fn fetch_episode_m3u8_core(
    client: &reqwest::Client,
    drama_id: &str,
    ep_id: &str,
    channel: &str,
) -> Result<String, String> {
    if drama_id.trim().is_empty() {
        return Err("Drama ID cannot be empty".to_string());
    }
    if ep_id.trim().is_empty() {
        return Err("Episode ID cannot be empty".to_string());
    }
    if channel.trim().is_empty() {
        return Err("Channel ID cannot be empty".to_string());
    }

    let chinaq = ChinaQExtractor::new();
    chinaq
        .fetch_m3u8_for_episode(client, drama_id, ep_id, channel)
        .await
        .map_err(|e| e.to_string())
}
