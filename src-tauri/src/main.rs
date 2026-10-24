// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bingeget::commands::{fetch_episode_m3u8_core, parse_series_metadata_core};
use bingeget::engine::downloader::{build_download_args, parse_progress_line, DownloadConfig, DownloadTaskSpec};
use bingeget::models::SeriesMetadata;
use serde::Serialize;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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

#[tauri::command]
async fn parse_series_metadata(url: String) -> Result<SeriesMetadata, String> {
    parse_series_metadata_core(&url).await
}

#[tauri::command]
async fn fetch_episode_m3u8(
    drama_id: String,
    ep_id: String,
    channel: String,
) -> Result<String, String> {
    fetch_episode_m3u8_core(&drama_id, &ep_id, &channel).await
}

#[tauri::command]
async fn start_download_task(
    app: tauri::AppHandle,
    spec: DownloadTaskSpec,
    config: Option<DownloadConfig>,
) -> Result<(), String> {
    let config = config.unwrap_or_default();
    let args = build_download_args(&spec, &config);

    tokio::spawn(async move {
        let ep = spec.episode;

        let _ = app.emit("download-status", EpisodeStatusPayload {
            episode: ep,
            status: "Downloading".to_string(),
            error: None,
        });

        let mut child = match Command::new("n-m3u8dl-re")
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
                    error: Some(format!("Failed to start n-m3u8dl-re: {}", e)),
                });
                return;
            }
        };

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
                    error: Some(format!("Process exited with status: {}", status)),
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

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            parse_series_metadata,
            fetch_episode_m3u8,
            start_download_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
