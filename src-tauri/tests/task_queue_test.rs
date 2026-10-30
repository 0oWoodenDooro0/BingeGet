use bingeget::engine::downloader::DownloadTaskSpec;
use bingeget::engine::queue::{ActiveTaskHandle, QueueTaskState, TaskQueue};

#[test]
fn test_task_queue_concurrency_limiting() {
    let mut queue = TaskQueue::new(2); // Max 2 concurrent tasks

    // Add 5 episode tasks
    for ep in 1..=5 {
        queue.enqueue(DownloadTaskSpec {
            m3u8_url: format!("https://example.com/ep{}.m3u8", ep),
            custom_title: "繁花".to_string(),
            season: 1,
            episode: ep,
            save_dir: "/media/TV".to_string(),
            referer: "".to_string(),
        });
    }

    assert_eq!(queue.len(), 5);

    // Initial ready batch should return exactly 2 tasks
    let next1 = queue.get_next_tasks_to_start();
    assert_eq!(next1.len(), 2);
    assert_eq!(next1[0].episode, 1);
    assert_eq!(next1[1].episode, 2);

    // After starting those two, active count is 2. Next request should return empty.
    queue.set_task_state(1, QueueTaskState::Downloading);
    queue.set_task_state(2, QueueTaskState::Downloading);

    let next2 = queue.get_next_tasks_to_start();
    assert!(next2.is_empty(), "Should not start more than max_concurrent tasks");

    // Complete episode 1
    queue.set_task_state(1, QueueTaskState::Completed);

    // Now one slot is free, next request should return episode 3
    let next3 = queue.get_next_tasks_to_start();
    assert_eq!(next3.len(), 1);
    assert_eq!(next3[0].episode, 3);
}

#[test]
fn test_task_queue_cancel_and_retry() {
    let mut queue = TaskQueue::new(2);

    queue.enqueue(DownloadTaskSpec {
        m3u8_url: "https://example.com/ep1.m3u8".to_string(),
        custom_title: "繁花".to_string(),
        season: 1,
        episode: 1,
        save_dir: "/media/TV".to_string(),
        referer: "".to_string(),
    });

    queue.set_task_state(1, QueueTaskState::Downloading);
    queue.set_task_state(1, QueueTaskState::Failed("Network timeout".to_string()));

    let task = queue.get_task(1).expect("Task 1 must exist");
    assert_eq!(task.state, QueueTaskState::Failed("Network timeout".to_string()));

    // Retry
    queue.retry(1);
    let task_after_retry = queue.get_task(1).unwrap();
    assert_eq!(task_after_retry.state, QueueTaskState::Queued);
}

#[tokio::test]
async fn test_task_queue_active_task_tracking_and_process_kill() {
    let mut queue = TaskQueue::new(2);

    queue.enqueue(DownloadTaskSpec {
        m3u8_url: "https://example.com/ep1.m3u8".to_string(),
        custom_title: "繁花".to_string(),
        season: 1,
        episode: 1,
        save_dir: "/tmp".to_string(),
        referer: "".to_string(),
    });

    // Spawn a long-running child process
    let mut cmd = tokio::process::Command::new("sleep");
    cmd.arg("60");
    cmd.kill_on_drop(true);
    let mut child = cmd.spawn().expect("failed to spawn sleep process");
    let pid = child.id().expect("child has pid");

    let (kill_tx, mut kill_rx) = tokio::sync::oneshot::channel::<()>();

    let join_handle = tokio::spawn(async move {
        tokio::select! {
            _ = &mut kill_rx => {
                let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).output();
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {}
        }
    });

    let active_handle = ActiveTaskHandle {
        abort_handle: join_handle.abort_handle(),
        child_pid: Some(pid),
        kill_sender: Some(kill_tx),
    };

    queue.set_task_state(1, QueueTaskState::Downloading);
    queue.register_active_handle(1, active_handle);

    assert!(queue.is_active(1));
    assert_eq!(queue.active_count(), 1);

    // Cancel task 1
    let cancelled = queue.cancel(1);
    assert!(cancelled);
    assert!(!queue.is_active(1));
    assert_eq!(queue.get_task(1).unwrap().state, QueueTaskState::Cancelled);

    // Give a brief moment for the process kill to execute
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Verify process is killed
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let exit_status = child.try_wait().expect("try_wait succeeds");
    assert!(exit_status.is_some(), "Child process should have exited");
}

#[tokio::test]
async fn test_task_queue_concurrency_throttling_with_dispatch() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use bingeget::engine::queue::{start_download_queue_with_queue, cancel_download_task_with_queue, retry_download_task_with_queue, SharedTaskQueue};
    use bingeget::models::DownloadQueuePayload;

    let queue: SharedTaskQueue = Arc::new(Mutex::new(TaskQueue::new(2)));
    let client = reqwest::Client::new();

    let payload = DownloadQueuePayload {
        drama_id: "test-drama".to_string(),
        channel_id: "ikzy".to_string(),
        episodes: vec![1, 2, 3, 4],
        custom_title: "繁花".to_string(),
        season: 1,
        save_dir: "/tmp".to_string(),
        threads: 4,
        max_concurrent: 2,
        binary_path: Some("/bin/echo".to_string()),
        category_slug: Some("tv-cn".to_string()),
        source_id: Some("chinaq".to_string()),
    };

    let start_res = start_download_queue_with_queue(&queue, None, client, payload).await;
    assert!(start_res.is_ok());

    {
        let q = queue.lock().await;
        assert_eq!(q.len(), 4);
        assert_eq!(q.max_concurrent(), 2);
    }

    // Cancel task 3 (which was queued)
    let cancel_res = cancel_download_task_with_queue(&queue, 3).await;
    assert!(cancel_res.is_ok());

    {
        let q = queue.lock().await;
        assert_eq!(q.get_task(3).unwrap().state, QueueTaskState::Cancelled);
    }

    // Retry task 3
    let retry_res = retry_download_task_with_queue(&queue, 3).await;
    assert!(retry_res.is_ok());

    {
        let q = queue.lock().await;
        let task_3_state = &q.get_task(3).unwrap().state;
        assert!(matches!(task_3_state, QueueTaskState::Queued | QueueTaskState::Downloading | QueueTaskState::Completed | QueueTaskState::Failed(_)));
    }
}
