use crate::extractors::chinaq::ChinaQExtractor;
use crate::extractors::SiteExtractor;
use crate::models::SeriesMetadata;

pub async fn parse_series_metadata_core(url: &str) -> Result<SeriesMetadata, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    let chinaq = ChinaQExtractor::new();
    if chinaq.can_handle(url) {
        let client = reqwest::Client::new();
        return chinaq
            .fetch_series_metadata(&client, url)
            .await
            .map_err(|e| e.to_string());
    }

    Err(format!("Unsupported website URL: {}", url))
}

pub async fn fetch_episode_m3u8_core(
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
    let client = reqwest::Client::new();
    chinaq
        .fetch_m3u8_for_episode(&client, drama_id, ep_id, channel)
        .await
        .map_err(|e| e.to_string())
}
