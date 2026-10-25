use std::path::Path;
use tauri::State;

use crate::engine::probe::probe_stream_core;
use crate::engine::queue::run_download_queue;
pub use crate::engine::queue::start_download_queue_core;
use crate::extractors::global_registry;
use crate::extractors::ExtractorRegistry;
use crate::models::{
    DownloadQueuePayload, PreviewStreamInfo, SearchResult, SeriesMetadata, StreamDetails,
    SystemEnvironment,
};

pub fn resolve_binary_path(name: &str, preferred: &str) -> Option<String> {
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
}

pub fn detect_system_environment_core() -> SystemEnvironment {
    let nm3u8dl_path = resolve_binary_path("n-m3u8dl-re", "/usr/bin/n-m3u8dl-re");
    let ffmpeg_path = resolve_binary_path("ffmpeg", "/usr/bin/ffmpeg");
    let is_ready = nm3u8dl_path.is_some() && ffmpeg_path.is_some();

    #[cfg(target_os = "linux")]
    let gstreamer_ready = {
        let has_gst = resolve_binary_path("gst-inspect-1.0", "/usr/bin/gst-inspect-1.0").is_some();
        if has_gst {
            let output = std::process::Command::new("gst-inspect-1.0")
                .arg("playbin")
                .output();
            Some(output.map(|o| o.status.success()).unwrap_or(false))
        } else {
            Some(false)
        }
    };

    #[cfg(not(target_os = "linux"))]
    let gstreamer_ready: Option<bool> = None;

    SystemEnvironment {
        nm3u8dl_path,
        ffmpeg_path,
        is_ready,
        gstreamer_ready,
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

    let registry = global_registry();
    if let Some(extractor) = registry.get_by_url(url) {
        return extractor
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
    source_id: Option<&str>,
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

    let sid = source_id.unwrap_or("chinaq");
    let registry = global_registry();
    let extractor = registry
        .get_by_source_id(sid)
        .ok_or_else(|| format!("Extractor not found for source: {}", sid))?;

    extractor
        .fetch_m3u8_for_episode(client, drama_id, ep_id, channel)
        .await
        .map_err(|e| e.to_string())
}

pub async fn search_series_with_registry(
    client: &reqwest::Client,
    keyword: &str,
    registry: &ExtractorRegistry,
) -> Result<Vec<SearchResult>, String> {
    let kw = keyword.trim();
    if kw.is_empty() {
        return Err("Keyword cannot be empty".to_string());
    }

    let extractors = registry.all();
    let mut handles = Vec::new();

    for extractor in extractors {
        let client_clone = client.clone();
        let kw_string = kw.to_string();
        handles.push(tokio::spawn(async move {
            let name = extractor.name();
            let res = extractor.search(&client_clone, &kw_string).await;
            (name, res)
        }));
    }

    let mut all_results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((_name, Ok(results))) => {
                all_results.extend(results);
            }
            Ok((name, Err(e))) => {
                eprintln!("Search error on {}: {}", name, e);
            }
            Err(e) => {
                eprintln!("Search task join error: {}", e);
            }
        }
    }

    Ok(all_results)
}

pub async fn search_series_core(
    client: &reqwest::Client,
    keyword: &str,
) -> Result<Vec<SearchResult>, String> {
    search_series_with_registry(client, keyword, global_registry()).await
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
    source_id: Option<String>,
) -> Result<String, String> {
    fetch_episode_m3u8_core(&client, &drama_id, &ep_id, &channel, source_id.as_deref()).await
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
    source_id: Option<String>,
) -> Result<StreamDetails, String> {
    let ffprobe_bin = resolve_binary_path("ffprobe", "/usr/bin/ffprobe");
    probe_stream_core(
        &client,
        &drama_id,
        &ep_id,
        &channel,
        ffprobe_bin.as_deref(),
        source_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn get_episode_preview_stream(
    client: State<'_, reqwest::Client>,
    drama_id: String,
    ep_id: String,
    channel: String,
    source_id: Option<String>,
) -> Result<PreviewStreamInfo, String> {
    let m3u8_url = fetch_episode_m3u8_core(&client, &drama_id, &ep_id, &channel, source_id.as_deref()).await?;
    let sid = source_id.as_deref().unwrap_or("chinaq");
    let registry = global_registry();
    let extractor = registry
        .get_by_source_id(sid)
        .ok_or_else(|| format!("Extractor not found for source: {}", sid))?;
    let referer = if sid == "juqing" {
        extractor.build_episode_referer(&channel, &drama_id, &ep_id)
    } else {
        extractor.build_episode_referer("", &drama_id, &ep_id)
    };

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
