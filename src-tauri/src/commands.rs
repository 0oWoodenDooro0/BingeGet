use std::path::Path;
use tauri::State;

use crate::engine::probe::probe_stream_core;
use crate::engine::queue::run_download_queue;
pub use crate::engine::queue::start_download_queue_core;
use crate::extractors::chinaq::ChinaQExtractor;
use crate::extractors::SiteExtractor;
use crate::models::{
    DownloadQueuePayload, PreviewStreamInfo, SearchResult, SeriesMetadata, StreamDetails,
    SystemEnvironment,
};

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

pub async fn search_series_core(
    client: &reqwest::Client,
    keyword: &str,
) -> Result<Vec<SearchResult>, String> {
    let kw = keyword.trim();
    if kw.is_empty() {
        return Err("Keyword cannot be empty".to_string());
    }

    let mut all_results = Vec::new();
    let extractors: Vec<Box<dyn SiteExtractor>> = vec![
        Box::new(ChinaQExtractor::new()),
    ];

    for extractor in extractors {
        match extractor.search(client, kw).await {
            Ok(results) => all_results.extend(results),
            Err(e) => {
                eprintln!("Search error on {}: {}", extractor.name(), e);
            }
        }
    }

    Ok(all_results)
}

#[tauri::command]
pub fn get_system_environment() -> SystemEnvironment {
    detect_system_environment_core()
}

#[tauri::command]
pub async fn parse_series_metadata(
    client: State<'_, reqwest::Client>,
    url: String,
) -> Result<SeriesMetadata, String> {
    parse_series_metadata_core(&client, &url).await
}

#[tauri::command]
pub async fn fetch_episode_m3u8(
    client: State<'_, reqwest::Client>,
    drama_id: String,
    ep_id: String,
    channel: String,
) -> Result<String, String> {
    fetch_episode_m3u8_core(&client, &drama_id, &ep_id, &channel).await
}

#[tauri::command]
pub async fn search_series(
    client: State<'_, reqwest::Client>,
    keyword: String,
) -> Result<Vec<SearchResult>, String> {
    search_series_core(&client, &keyword).await
}

#[tauri::command]
pub async fn probe_episode_stream(
    client: State<'_, reqwest::Client>,
    drama_id: String,
    ep_id: String,
    channel: String,
) -> Result<StreamDetails, String> {
    let ffprobe_path = if Path::new("/usr/bin/ffprobe").is_file() {
        Some("/usr/bin/ffprobe")
    } else {
        None
    };
    probe_stream_core(&client, &drama_id, &ep_id, &channel, ffprobe_path).await
}

#[tauri::command]
pub async fn get_episode_preview_stream(
    client: State<'_, reqwest::Client>,
    drama_id: String,
    ep_id: String,
    channel: String,
) -> Result<PreviewStreamInfo, String> {
    let m3u8_url = fetch_episode_m3u8_core(&client, &drama_id, &ep_id, &channel).await?;
    let chinaq = ChinaQExtractor::new();
    let referer = chinaq.build_episode_referer("", &drama_id, &ep_id);

    let proxy_port = crate::engine::proxy::start_streaming_proxy(client.inner().clone())
        .await
        .map_err(|e| e.to_string())?;

    let proxy_url = Some(crate::engine::proxy::build_proxy_url(
        proxy_port, &m3u8_url, &referer,
    ));

    Ok(PreviewStreamInfo {
        m3u8_url,
        referer,
        proxy_url,
    })
}

#[tauri::command]
pub async fn start_download_queue(
    app: tauri::AppHandle,
    client: State<'_, reqwest::Client>,
    req: DownloadQueuePayload,
) -> Result<(), String> {
    run_download_queue(app, client.inner().clone(), req).await
}
