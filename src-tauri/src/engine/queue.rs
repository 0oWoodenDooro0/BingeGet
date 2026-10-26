use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{Mutex, oneshot};

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

#[derive(Debug)]
pub struct ActiveTaskHandle {
    pub abort_handle: tokio::task::AbortHandle,
    pub child_pid: Option<u32>,
    pub kill_sender: Option<oneshot::Sender<()>>,
}

#[derive(Clone)]
pub struct QueueExecutionContext {
    pub app: Option<tauri::AppHandle>,
    pub client: reqwest::Client,
    pub drama_id: String,
    pub channel_id: String,
    pub category_slug: String,
    pub source_id: String,
    pub custom_title: String,
    pub season: u32,
    pub save_dir: String,
    pub threads: u32,
    pub binary_path: Option<String>,
}

pub struct TaskQueue {
    tasks: Vec<QueueItem>,
    max_concurrent: usize,
    active_handles: HashMap<u32, ActiveTaskHandle>,
    execution_ctx: Option<QueueExecutionContext>,
}

impl TaskQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            tasks: Vec::new(),
            max_concurrent: if max_concurrent == 0 { 2 } else { max_concurrent },
            active_handles: HashMap::new(),
            execution_ctx: None,
        }
    }

    pub fn enqueue(&mut self, spec: DownloadTaskSpec) {
        if let Some(existing) = self.tasks.iter_mut().find(|t| t.spec.episode == spec.episode) {
            existing.spec = spec;
            existing.state = QueueTaskState::Queued;
        } else {
            let next_id = (self.tasks.len() + 1) as u32;
            self.tasks.push(QueueItem {
                id: next_id,
                spec,
                state: QueueTaskState::Queued,
            });
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = if max == 0 { 2 } else { max };
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

    pub fn is_active(&self, episode: u32) -> bool {
        self.active_handles.contains_key(&episode)
    }

    pub fn register_active_handle(&mut self, episode: u32, handle: ActiveTaskHandle) {
        self.active_handles.insert(episode, handle);
    }

    pub fn update_active_child_pid(&mut self, episode: u32, pid: u32) {
        if let Some(handle) = self.active_handles.get_mut(&episode) {
            handle.child_pid = Some(pid);
        }
    }

    pub fn unregister_active_handle(&mut self, episode: u32) -> Option<ActiveTaskHandle> {
        self.active_handles.remove(&episode)
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

    pub fn all_tasks(&self) -> &[QueueItem] {
        &self.tasks
    }

    pub fn cancel(&mut self, episode: u32) -> bool {
        let mut found = false;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.spec.episode == episode) {
            task.state = QueueTaskState::Cancelled;
            found = true;
        }

        if let Some(handle) = self.active_handles.remove(&episode) {
            found = true;
            if let Some(tx) = handle.kill_sender {
                let _ = tx.send(());
            }
            handle.abort_handle.abort();
            #[cfg(unix)]
            if let Some(pid) = handle.child_pid {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
        }

        found
    }

    pub fn retry(&mut self, episode: u32) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.spec.episode == episode) {
            if matches!(task.state, QueueTaskState::Failed(_) | QueueTaskState::Cancelled) {
                task.state = QueueTaskState::Queued;
                return true;
            }
        }
        false
    }

    pub fn clear(&mut self) {
        for (_, handle) in self.active_handles.drain() {
            if let Some(tx) = handle.kill_sender {
                let _ = tx.send(());
            }
            handle.abort_handle.abort();
            #[cfg(unix)]
            if let Some(pid) = handle.child_pid {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
        }
        self.tasks.clear();
        self.execution_ctx = None;
    }

    pub fn set_execution_ctx(&mut self, ctx: QueueExecutionContext) {
        self.execution_ctx = Some(ctx);
    }

    pub fn execution_ctx(&self) -> Option<&QueueExecutionContext> {
        self.execution_ctx.as_ref()
    }
}

pub type SharedTaskQueue = Arc<Mutex<TaskQueue>>;

static GLOBAL_QUEUE: LazyLock<SharedTaskQueue> = LazyLock::new(|| {
    Arc::new(Mutex::new(TaskQueue::new(2)))
});

pub fn global_task_queue() -> &'static SharedTaskQueue {
    &GLOBAL_QUEUE
}

fn emit_progress_if_matched(app: &Option<tauri::AppHandle>, line: &str, ep: u32) {
    if let Some(app) = app {
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
}

async fn complete_task(queue: &SharedTaskQueue, app: &Option<tauri::AppHandle>, ep: u32) {
    {
        let mut q = queue.lock().await;
        q.unregister_active_handle(ep);
        q.set_task_state(ep, QueueTaskState::Completed);
    }
    if let Some(app) = app {
        let _ = app.emit(
            "download-status",
            EpisodeStatusPayload {
                episode: ep,
                status: "Completed".to_string(),
                error: None,
            },
        );
    }
    let q_clone = queue.clone();
    tokio::spawn(async move {
        dispatch_queue(&q_clone).await;
    });
}

async fn fail_task(queue: &SharedTaskQueue, app: &Option<tauri::AppHandle>, ep: u32, err: String) {
    {
        let mut q = queue.lock().await;
        q.unregister_active_handle(ep);
        q.set_task_state(ep, QueueTaskState::Failed(err.clone()));
    }
    if let Some(app) = app {
        let _ = app.emit(
            "download-status",
            EpisodeStatusPayload {
                episode: ep,
                status: "Failed".to_string(),
                error: Some(err),
            },
        );
    }
    let q_clone = queue.clone();
    tokio::spawn(async move {
        dispatch_queue(&q_clone).await;
    });
}

async fn cleanup_cancelled_task(queue: &SharedTaskQueue, ep: u32) {
    {
        let mut q = queue.lock().await;
        q.unregister_active_handle(ep);
        q.set_task_state(ep, QueueTaskState::Cancelled);
    }
    let q_clone = queue.clone();
    tokio::spawn(async move {
        dispatch_queue(&q_clone).await;
    });
}

async fn run_single_task(
    queue: SharedTaskQueue,
    ctx: QueueExecutionContext,
    spec: DownloadTaskSpec,
    mut kill_rx: oneshot::Receiver<()>,
) {
    let ep = spec.episode;
    let ep_str = format!("ep{}", ep);
    let app = ctx.app.clone();

    let registry = global_registry();
    let extractor = match registry.get_by_source_id(&ctx.source_id) {
        Some(e) => e,
        None => {
            let err = format!("Extractor not found for source: {}", ctx.source_id);
            fail_task(&queue, &app, ep, err).await;
            return;
        }
    };

    let referer = extractor.build_stream_referer(
        &ctx.drama_id,
        &ep_str,
        &ctx.channel_id,
        Some(&ctx.category_slug),
    );

    // 1. Fetch live m3u8 URL for this episode
    let fetch_fut = fetch_episode_m3u8_core(
        &ctx.client,
        &ctx.drama_id,
        &ep_str,
        &ctx.channel_id,
        Some(&ctx.source_id),
    );

    let m3u8_url = tokio::select! {
        _ = &mut kill_rx => {
            cleanup_cancelled_task(&queue, ep).await;
            return;
        }
        res = fetch_fut => {
            match res {
                Ok(url) => url,
                Err(e) => {
                    fail_task(&queue, &app, ep, format!("Failed to fetch m3u8: {}", e)).await;
                    return;
                }
            }
        }
    };

    let task_spec = DownloadTaskSpec {
        m3u8_url,
        custom_title: ctx.custom_title.clone(),
        season: ctx.season,
        episode: ep,
        save_dir: ctx.save_dir.clone(),
        referer,
    };

    let config = DownloadConfig {
        threads: ctx.threads,
        tmp_dir: "/tmp/bingeget_chunks".to_string(),
        retry_count: 5,
        binary_path: ctx.binary_path.clone(),
    };

    let args = build_download_args(&task_spec, &config);
    let bin = ctx.binary_path.clone().unwrap_or_else(|| "n-m3u8dl-re".to_string());

    let mut cmd = Command::new(&bin);
    cmd.args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            fail_task(&queue, &app, ep, format!("Failed to start {}: {}", bin, e)).await;
            return;
        }
    };

    let child_pid = child.id();
    if let Some(pid) = child_pid {
        let mut q = queue.lock().await;
        q.update_active_child_pid(ep, pid);
    }

    // Drain stderr in separate task
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
    let stdout_handle = child.stdout.take().map(|stdout| {
        let app_clone = app.clone();
        tokio::spawn(async move {
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
                            emit_progress_if_matched(&app_clone, &line_str, ep);
                            line_buf.clear();
                        }
                    } else {
                        line_buf.push(b);
                    }
                }
            }
            if !line_buf.is_empty() {
                let line_str = String::from_utf8_lossy(&line_buf);
                emit_progress_if_matched(&app_clone, &line_str, ep);
            }
        })
    });

    tokio::select! {
        _ = &mut kill_rx => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            if let Some(h) = stdout_handle {
                h.abort();
            }
            #[cfg(unix)]
            if let Some(pid) = child_pid {
                let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).output();
            }
            cleanup_cancelled_task(&queue, ep).await;
        }
        wait_res = child.wait() => {
            if let Some(h) = stdout_handle {
                let _ = h.await;
            }
            match wait_res {
                Ok(status) => {
                    if status.success() {
                        complete_task(&queue, &app, ep).await;
                    } else {
                        fail_task(&queue, &app, ep, format!("Process exited with status: {}", status)).await;
                    }
                }
                Err(e) => {
                    fail_task(&queue, &app, ep, format!("Failed waiting for process: {}", e)).await;
                }
            }
        }
    }
}

pub fn dispatch_queue<'a>(queue: &'a SharedTaskQueue) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    let queue = queue.clone();
    Box::pin(async move {
    let mut tasks_to_spawn = Vec::new();
    let ctx_opt;

    {
        let mut q = queue.lock().await;
        ctx_opt = q.execution_ctx().cloned();
        if let Some(_ctx) = &ctx_opt {
            let next_specs = q.get_next_tasks_to_start();
            for spec in next_specs {
                q.set_task_state(spec.episode, QueueTaskState::Downloading);
                tasks_to_spawn.push(spec);
            }
        }
    }

    let Some(ctx) = ctx_opt else { return; };
    if tasks_to_spawn.is_empty() { return; }

    for spec in tasks_to_spawn {
        let ep = spec.episode;
        if let Some(app) = &ctx.app {
            let _ = app.emit(
                "download-status",
                EpisodeStatusPayload {
                    episode: ep,
                    status: "Downloading".to_string(),
                    error: None,
                },
            );
        }

        let (kill_tx, kill_rx) = oneshot::channel::<()>();
        let queue_clone = queue.clone();
        let ctx_clone = ctx.clone();
        let spec_clone = spec.clone();

        let join_handle = tokio::spawn(async move {
            run_single_task(queue_clone, ctx_clone, spec_clone, kill_rx).await;
        });

        {
            let mut q = queue.lock().await;
            q.register_active_handle(
                ep,
                ActiveTaskHandle {
                    abort_handle: join_handle.abort_handle(),
                    child_pid: None,
                    kill_sender: Some(kill_tx),
                },
            );
        }
    }
    })
}

pub async fn start_download_queue_with_queue(
    queue: &SharedTaskQueue,
    app: Option<tauri::AppHandle>,
    client: reqwest::Client,
    req: DownloadQueuePayload,
) -> Result<(), String> {
    let max_concurrent = if req.max_concurrent == 0 { 2 } else { req.max_concurrent };
    let category_slug = req.category_slug.unwrap_or_else(|| "tv-cn".to_string());
    let source_id = req.source_id.unwrap_or_else(|| "chinaq".to_string());

    let ctx = QueueExecutionContext {
        app,
        client,
        drama_id: req.drama_id,
        channel_id: req.channel_id,
        category_slug,
        source_id,
        custom_title: req.custom_title.clone(),
        season: req.season,
        save_dir: req.save_dir.clone(),
        threads: req.threads,
        binary_path: req.binary_path,
    };

    {
        let mut q = queue.lock().await;
        q.set_max_concurrent(max_concurrent);
        q.set_execution_ctx(ctx);

        for ep in req.episodes {
            let spec = DownloadTaskSpec {
                m3u8_url: String::new(),
                custom_title: req.custom_title.clone(),
                season: req.season,
                episode: ep,
                save_dir: req.save_dir.clone(),
                referer: String::new(),
            };
            q.enqueue(spec);
        }
    }

    dispatch_queue(queue).await;
    Ok(())
}

pub async fn cancel_download_task_with_queue(
    queue: &SharedTaskQueue,
    episode: u32,
) -> Result<(), String> {
    let mut q = queue.lock().await;
    let cancelled = q.cancel(episode);
    if !cancelled {
        return Err(format!("Task for episode {} not found", episode));
    }
    if let Some(ctx) = q.execution_ctx() {
        if let Some(app) = &ctx.app {
            let _ = app.emit(
                "download-status",
                EpisodeStatusPayload {
                    episode,
                    status: "Cancelled".to_string(),
                    error: None,
                },
            );
        }
    }
    drop(q);
    dispatch_queue(queue).await;
    Ok(())
}

pub async fn retry_download_task_with_queue(
    queue: &SharedTaskQueue,
    episode: u32,
) -> Result<(), String> {
    let mut q = queue.lock().await;
    let retried = q.retry(episode);
    if !retried {
        return Err(format!(
            "Task for episode {} not found or not in failed/cancelled state",
            episode
        ));
    }
    if let Some(ctx) = q.execution_ctx() {
        if let Some(app) = &ctx.app {
            let _ = app.emit(
                "download-status",
                EpisodeStatusPayload {
                    episode,
                    status: "Queued".to_string(),
                    error: None,
                },
            );
        }
    }
    drop(q);
    dispatch_queue(queue).await;
    Ok(())
}

pub async fn cancel_download_task_core(episode: u32) -> Result<(), String> {
    cancel_download_task_with_queue(global_task_queue(), episode).await
}

pub async fn retry_download_task_core(episode: u32) -> Result<(), String> {
    retry_download_task_with_queue(global_task_queue(), episode).await
}

pub async fn run_download_queue(
    app: tauri::AppHandle,
    client: reqwest::Client,
    req: DownloadQueuePayload,
) -> Result<(), String> {
    start_download_queue_with_queue(global_task_queue(), Some(app), client, req).await
}

pub use run_download_queue as start_download_queue_core;
