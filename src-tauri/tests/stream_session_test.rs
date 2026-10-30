use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use bingeget::engine::stream::prepare_episode_stream;
use bingeget::models::StreamSession;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_prepare_episode_stream_validation() {
    let client = reqwest::Client::new();

    let err1 = prepare_episode_stream(&client, "", "1", "1", None, None)
        .await
        .unwrap_err();
    assert!(err1.contains("Drama ID cannot be empty"));

    let err2 = prepare_episode_stream(&client, "drama1", "", "1", None, None)
        .await
        .unwrap_err();
    assert!(err2.contains("Episode ID cannot be empty"));

    let err3 = prepare_episode_stream(&client, "drama1", "1", "", None, None)
        .await
        .unwrap_err();
    assert!(err3.contains("Channel ID cannot be empty"));

    let err4 = prepare_episode_stream(&client, "drama1", "1", "1", Some("unknown_source"), None)
        .await
        .unwrap_err();
    assert!(err4.contains("Extractor not found for source: unknown_source"));
}

#[tokio::test]
async fn test_prepare_episode_stream_atomic_single_fetch_and_probe() {
    let m3u8_fetch_count = Arc::new(AtomicUsize::new(0));
    let count_clone = m3u8_fetch_count.clone();

    let sample_m3u8 = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=3198000,RESOLUTION=1920x1080\nsub.m3u8\n";

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_task = tokio::spawn(async move {
        // 1. Play page request from Juqing extractor
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let play_html = format!(
                r#"<video id="my-player"><source src="http://{}/stream.m3u8" channel="7" type="application/x-mpegURL"></video>"#,
                addr
            );
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                play_html.len(),
                play_html
            );
            let _ = socket.write_all(resp.as_bytes()).await;
        }

        // 2. Playlist request (should happen ONLY ONCE)
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                let req_str = String::from_utf8_lossy(&buf[..n]);
                if req_str.contains("/stream.m3u8") {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\n\r\n{}",
                        sample_m3u8.len(),
                        sample_m3u8
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                }
            } else {
                break;
            }
        }
    });

    let client = reqwest::Client::new();
    let mock_drama_url = format!("http://{}", addr);

    // Call atomic prepare_episode_stream
    let session: StreamSession = prepare_episode_stream(
        &client,
        &mock_drama_url,
        "1",
        "7",
        Some("juqing"),
        None,
    )
    .await
    .expect("prepare_episode_stream should succeed");

    assert_eq!(session.stream_details.resolution, "1920x1080 (1080p)");
    assert_eq!(session.stream_details.bitrate, "3,198 kbps");
    assert!(session.proxy_url.is_some());
    assert_eq!(m3u8_fetch_count.load(Ordering::SeqCst), 1);

    // Now verify that requesting the local proxy_url uses the cached playlist without contacting upstream
    let proxy_url = session.proxy_url.as_ref().unwrap();
    let proxy_resp = client.get(proxy_url).send().await.expect("Proxy fetch should succeed");
    assert_eq!(proxy_resp.status(), 200);
    let proxy_body = proxy_resp.text().await.expect("Read proxy body");
    assert!(proxy_body.contains("/proxy?url="));

    // Upstream fetch count must STILL be 1!
    assert_eq!(m3u8_fetch_count.load(Ordering::SeqCst), 1);

    server_task.abort();
}
