use bingeget::engine::downloader::DownloadTaskSpec;
use bingeget::engine::queue::{QueueTaskState, TaskQueue};

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
