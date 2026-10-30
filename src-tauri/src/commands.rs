use std::path::Path;
use tauri::State;

use crate::engine::probe::probe_stream_core;
pub use crate::engine::queue::{
    cancel_download_task_core, retry_download_task_core, run_download_queue,
    start_download_queue_core,
};
pub use crate::engine::stream::prepare_episode_stream as prepare_episode_stream_core;
use crate::extractors::{global_catalog, Catalog};
use crate::models::{
    DownloadQueuePayload, PreviewStreamInfo, SearchResult, SeriesMetadata, StreamDetails,
    StreamSession, SystemEnvironment,
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
    global_catalog().parse_series_metadata(client, url).await
}

pub async fn fetch_episode_m3u8_core(
    client: &reqwest::Client,
    drama_id: &str,
    ep_id: &str,
    channel: &str,
    source_id: Option<&str>,
) -> Result<String, String> {
    global_catalog()
        .fetch_episode_m3u8(client, drama_id, ep_id, channel, source_id)
        .await
}

pub async fn search_series_with_catalog(
    client: &reqwest::Client,
    keyword: &str,
    catalog: &Catalog,
) -> Result<Vec<SearchResult>, String> {
    catalog.search(client, keyword).await
}

pub async fn search_series_core(
    client: &reqwest::Client,
    keyword: &str,
) -> Result<Vec<SearchResult>, String> {
    global_catalog().search(client, keyword).await
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
pub async fn prepare_episode_stream(
    client: State<'_, reqwest::Client>,
    drama_id: String,
    ep_id: String,
    channel: String,
    source_id: Option<String>,
) -> Result<StreamSession, String> {
    let ffprobe_bin = resolve_binary_path("ffprobe", "/usr/bin/ffprobe");
    prepare_episode_stream_core(
        &client,
        &drama_id,
        &ep_id,
        &channel,
        source_id.as_deref(),
        ffprobe_bin.as_deref(),
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
    let ffprobe_bin = resolve_binary_path("ffprobe", "/usr/bin/ffprobe");
    let session = prepare_episode_stream_core(
        &client,
        &drama_id,
        &ep_id,
        &channel,
        source_id.as_deref(),
        ffprobe_bin.as_deref(),
    )
    .await?;

    Ok(session.to_preview_stream_info())
}

#[tauri::command]
pub async fn start_download_queue(
    app: tauri::AppHandle,
    client: State<'_, reqwest::Client>,
    req: DownloadQueuePayload,
) -> Result<(), String> {
    run_download_queue(app, client.inner().clone(), req).await
}

#[tauri::command]
pub async fn cancel_download_task(episode: u32) -> Result<(), String> {
    cancel_download_task_core(episode).await
}

#[tauri::command]
pub async fn retry_download_task(episode: u32) -> Result<(), String> {
    retry_download_task_core(episode).await
}
