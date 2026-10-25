use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Semaphore;

use crate::commands::fetch_episode_m3u8_core;
use crate::engine::downloader::{
    build_download_args, parse_progress_line, DownloadConfig, DownloadProgress, DownloadTaskSpec,
};
use crate::extractors::global_registry;
use crate::models::{DownloadQueuePayload, EpisodeStatusPayload, ProgressPayload};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueueTaskState {
    Queued,
    Downloading,
    Progress(DownloadProgress),
    Muxing,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: u32,
    pub spec: DownloadTaskSpec,
    pub state: QueueTaskState,
}

pub struct TaskQueue {
    tasks: Vec<QueueItem>,
    max_concurrent: usize,
}

impl TaskQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            tasks: Vec::new(),
            max_concurrent,
        }
    }

    pub fn enqueue(&mut self, spec: DownloadTaskSpec) {
        let next_id = (self.tasks.len() + 1) as u32;
        self.tasks.push(QueueItem {
            id: next_id,
            spec,
            state: QueueTaskState::Queued,
        });
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn active_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| {
                matches!(
                    t.state,
                    QueueTaskState::Downloading
                        | QueueTaskState::Progress(_)
                        | QueueTaskState::Muxing
                )
            })
            .count()
    }

    pub fn get_next_tasks_to_start(&self) -> Vec<DownloadTaskSpec> {
        let active = self.active_count();
        if active >= self.max_concurrent {
            return Vec::new();
        }

        let slots_available = self.max_concurrent - active;
        self.tasks
            .iter()
            .filter(|t| t.state == QueueTaskState::Queued)
            .take(slots_available)
            .map(|t| t.spec.clone())
            .collect()
    }

    pub fn set_task_state(&mut self, episode: u32, state: QueueTaskState) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.spec.episode == episode) {
            task.state = state;
        }
    }

    pub fn get_task(&self, episode: u32) -> Option<&QueueItem> {
        self.tasks.iter().find(|t| t.spec.episode == episode)
    }

    pub fn retry(&mut self, episode: u32) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.spec.episode == episode) {
            if matches!(task.state, QueueTaskState::Failed(_) | QueueTaskState::Cancelled) {
                task.state = QueueTaskState::Queued;
            }
        }
    }

    pub fn cancel(&mut self, episode: u32) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.spec.episode == episode) {
            task.state = QueueTaskState::Cancelled;
        }
    }

    pub fn all_tasks(&self) -> &[QueueItem] {
        &self.tasks
    }
}

fn emit_progress_if_matched(app: &tauri::AppHandle, line: &str, ep: u32) {
    if let Some(prog) = parse_progress_line(line) {
        let _ = app.emit(
            "download-progress",
            ProgressPayload {
                episode: ep,
                percent: prog.percent,
                speed: prog.speed,
                eta: prog.eta,
            },
        );
    }
}

pub async fn run_download_queue(
    app: tauri::AppHandle,
    client: reqwest::Client,
    req: DownloadQueuePayload,
) -> Result<(), String> {
    let max_concurrent = if req.max_concurrent == 0 { 2 } else { req.max_concurrent };
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let drama_id = req.drama_id;
    let channel_id = req.channel_id;
    let custom_title = req.custom_title;
    let season = req.season;
    let save_dir = req.save_dir;
    let threads = req.threads;
    let binary_path = req.binary_path;
    let category_slug = req.category_slug.unwrap_or_else(|| "tv-cn".to_string());
    let source_id = req.source_id.unwrap_or_else(|| "chinaq".to_string());

    let registry = global_registry();
    let extractor = registry
        .get_by_source_id(&source_id)
        .ok_or_else(|| format!("Extractor not found for source: {}", source_id))?;

    for ep in req.episodes {
        let app = app.clone();
        let client = client.clone();
        let sem = semaphore.clone();

        let drama_id = drama_id.clone();
        let channel_id = channel_id.clone();
        let custom_title = custom_title.clone();
        let save_dir = save_dir.clone();
        let binary_path = binary_path.clone();
        let category_slug = category_slug.clone();
        let source_id = source_id.clone();
        let referer = extractor.build_episode_referer(&category_slug, &drama_id, &format!("ep{}", ep));

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
            let m3u8_url = match fetch_episode_m3u8_core(&client, &drama_id, &ep_id, &channel_id, Some(&source_id)).await {
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
                referer,
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

            // Stream stdout for live progress events (split by \r or \n)
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout);
                let mut buf = [0u8; 1024];
                let mut line_buf = Vec::new();
                while let Ok(n) = reader.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    for &b in &buf[..n] {
                        if b == b'\r' || b == b'\n' {
                            if !line_buf.is_empty() {
                                let line_str = String::from_utf8_lossy(&line_buf);
                                emit_progress_if_matched(&app, &line_str, ep);
                                line_buf.clear();
                            }
                        } else {
                            line_buf.push(b);
                        }
                    }
                }
                if !line_buf.is_empty() {
                    let line_str = String::from_utf8_lossy(&line_buf);
                    emit_progress_if_matched(&app, &line_str, ep);
                }
            }

            let status = match child.wait().await {
                Ok(s) => s,
                Err(e) => {
                    let _ = app.emit("download-status", EpisodeStatusPayload {
                        episode: ep,
                        status: "Failed".to_string(),
                        error: Some(format!("Failed waiting for process: {}", e)),
                    });
                    return;
                }
            };

            if status.success() {
                let _ = app.emit("download-status", EpisodeStatusPayload {
                    episode: ep,
                    status: "Completed".to_string(),
                    error: None,
                });
            } else {
                let _ = app.emit("download-status", EpisodeStatusPayload {
                    episode: ep,
                    status: "Failed".to_string(),
                    error: Some(format!("Process exited with status: {}", status)),
                });
            }
        });
    }

    Ok(())
}

pub use run_download_queue as start_download_queue_core;
