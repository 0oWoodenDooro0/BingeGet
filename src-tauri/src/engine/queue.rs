use serde::{Deserialize, Serialize};
use crate::engine::downloader::{DownloadProgress, DownloadTaskSpec};

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
            .filter(|t| matches!(t.state, QueueTaskState::Downloading | QueueTaskState::Progress(_) | QueueTaskState::Muxing))
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
