// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use bingeget::commands::{
    detect_system_environment_core, fetch_episode_m3u8_core, parse_series_metadata_core,
};
use bingeget::engine::downloader::{build_download_args, parse_progress_line, DownloadConfig, DownloadTaskSpec};
use bingeget::models::{SeriesMetadata, SystemEnvironment};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Semaphore;

#[derive(Clone, Serialize)]
struct ProgressPayload {
    episode: u32,
    percent: f32,
    speed: String,
    eta: String,
}

#[derive(Clone, Serialize)]
struct EpisodeStatusPayload {
    episode: u32,
    status: String,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadQueuePayload {
    pub drama_id: String,
    pub channel_id: String,
    pub episodes: Vec<u32>,
    pub custom_title: String,
    pub season: u32,
    pub save_dir: String,
    pub threads: u32,
    pub max_concurrent: usize,
    pub binary_path: Option<String>,
}

#[tauri::command]
fn get_system_environment() -> SystemEnvironment {
    detect_system_environment_core()
}

#[tauri::command]
async fn parse_series_metadata(
    client: State<'_, reqwest::Client>,
    url: String,
) -> Result<SeriesMetadata, String> {
    parse_series_metadata_core(&client, &url).await
}

#[tauri::command]
async fn fetch_episode_m3u8(
    client: State<'_, reqwest::Client>,
    drama_id: String,
    ep_id: String,
    channel: String,
) -> Result<String, String> {
    fetch_episode_m3u8_core(&client, &drama_id, &ep_id, &channel).await
}

#[tauri::command]
async fn start_download_queue(
    app: tauri::AppHandle,
    client: State<'_, reqwest::Client>,
    req: DownloadQueuePayload,
) -> Result<(), String> {
    let client = client.inner().clone();
    let max_concurrent = if req.max_concurrent == 0 { 2 } else { req.max_concurrent };
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let drama_id = req.drama_id;
    let channel_id = req.channel_id;
    let custom_title = req.custom_title;
    let season = req.season;
    let save_dir = req.save_dir;
    let threads = req.threads;
    let binary_path = req.binary_path;

    for ep in req.episodes {
        let app = app.clone();
        let client = client.clone();
        let sem = semaphore.clone();

        let drama_id = drama_id.clone();
        let channel_id = channel_id.clone();
        let custom_title = custom_title.clone();
        let save_dir = save_dir.clone();
        let binary_path = binary_path.clone();

        tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => return,
            };

            let ep_id = format!("ep{}", ep);
            let _ = app.emit("download-status", EpisodeStatusPayload {
                episode: ep,
                status: "Downloading".to_string(),
                error: None,
            });

            // 1. Fetch live m3u8 URL for this episode
            let m3u8_url = match fetch_episode_m3u8_core(&client, &drama_id, &ep_id, &channel_id).await {
                Ok(url) => url,
                Err(e) => {
                    let _ = app.emit("download-status", EpisodeStatusPayload {
                        episode: ep,
                        status: "Failed".to_string(),
                        error: Some(format!("Failed to fetch m3u8: {}", e)),
                    });
                    return;
                }
            };

            let spec = DownloadTaskSpec {
                m3u8_url,
                custom_title,
                season,
                episode: ep,
                save_dir,
                referer: format!("https://chinaq.fun/tv-cn/{}/{}.html", drama_id, ep_id),
            };

            let config = DownloadConfig {
                threads,
                tmp_dir: "/tmp/bingeget_chunks".to_string(),
                retry_count: 5,
                binary_path: binary_path.clone(),
            };

            let args = build_download_args(&spec, &config);
            let bin = binary_path.unwrap_or_else(|| "n-m3u8dl-re".to_string());

            let mut child = match Command::new(&bin)
                .args(&args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = app.emit("download-status", EpisodeStatusPayload {
                        episode: ep,
                        status: "Failed".to_string(),
                        error: Some(format!("Failed to start {}: {}", bin, e)),
                    });
                    return;
                }
            };

            // Prevent process deadlock: actively drain stderr in a separate task
            if let Some(stderr) = child.stderr.take() {
                tokio::spawn(async move {
                    let mut reader = BufReader::new(stderr);
                    let mut buf = [0u8; 1024];
                    while let Ok(n) = reader.read(&mut buf).await {
                        if n == 0 {
                            break;
                        }
                    }
                });
            }

            // Stream stdout for live progress events
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Some(prog) = parse_progress_line(&line) {
                        let _ = app.emit("download-progress", ProgressPayload {
                            episode: ep,
                            percent: prog.percent,
                            speed: prog.speed,
                            eta: prog.eta,
                        });
                    }
                }
            }

            match child.wait().await {
                Ok(status) if status.success() => {
                    let _ = app.emit("download-status", EpisodeStatusPayload {
                        episode: ep,
                        status: "Completed".to_string(),
                        error: None,
                    });
                }
                Ok(status) => {
                    let _ = app.emit("download-status", EpisodeStatusPayload {
                        episode: ep,
                        status: "Failed".to_string(),
                        error: Some(format!("Exited with status: {}", status)),
                    });
                }
                Err(e) => {
                    let _ = app.emit("download-status", EpisodeStatusPayload {
                        episode: ep,
                        status: "Failed".to_string(),
                        error: Some(format!("Wait failed: {}", e)),
                    });
                }
            }
        });
    }

    Ok(())
}

fn main() {
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to build HTTP client");

    tauri::Builder::default()
        .manage(client)
        .invoke_handler(tauri::generate_handler![
            get_system_environment,
            parse_series_metadata,
            fetch_episode_m3u8,
            start_download_queue,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
