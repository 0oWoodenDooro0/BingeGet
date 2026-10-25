use bingeget::engine::probe::{parse_ffprobe_json_output, parse_m3u8_master_tags};
use bingeget::engine::proxy::{build_proxy_client, rewrite_m3u8};

#[test]
fn test_parse_m3u8_master_tags_success() {
    let sample_master_playlist = r#"#EXTM3U
#EXT-X-STREAM-INF:PROGRAM-ID=1,BANDWIDTH=3198000,RESOLUTION=1920x1080
/20261009/bHXyTRZf/3198kb/hls/index.m3u8
"#;

    let details = parse_m3u8_master_tags(sample_master_playlist)
        .expect("Should parse master playlist tags");

    assert_eq!(details.resolution, "1920x1080 (1080p)");
    assert_eq!(details.bitrate, "3,198 kbps");
    assert!(details.video_codec.contains("H.264"));
    assert!(details.audio_codec.contains("AAC"));
}

#[test]
fn test_parse_m3u8_master_tags_non_master() {
    let sample_media_playlist = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:6
#EXTINF:6.000,
segment0.ts
#EXTINF:6.000,
segment1.ts
#EXT-X-ENDLIST
"#;

    let details = parse_m3u8_master_tags(sample_media_playlist);
    assert!(details.is_none(), "Regular media playlist should return None for master tags");
}

#[test]
fn test_parse_m3u8_master_tags_with_bogus_bandwidth_and_uri_hint() {
    let sample = r#"#EXTM3U
#EXT-X-STREAM-INF:PROGRAM-ID=1,BANDWIDTH=128,RESOLUTION=1280x720
1000k_720/hls/index.m3u8
"#;

    let details = parse_m3u8_master_tags(sample)
        .expect("Should parse master playlist tags");

    assert_eq!(details.resolution, "1280x720 (720p)");
    // Should NOT be 0 kbps; should resolve to 1,000 kbps from URI hint or fallback
    assert_ne!(details.bitrate, "0 kbps");
    assert_eq!(details.bitrate, "1,000 kbps");
}

#[test]
fn test_parse_m3u8_master_tags_with_bogus_bandwidth_and_resolution_fallback() {
    let sample = r#"#EXTM3U
#EXT-X-STREAM-INF:PROGRAM-ID=1,BANDWIDTH=128,RESOLUTION=1920x1080
subplaylist/video.m3u8
"#;

    let details = parse_m3u8_master_tags(sample)
        .expect("Should parse master playlist tags");

    assert_eq!(details.resolution, "1920x1080 (1080p)");
    assert_eq!(details.bitrate, "2,500 kbps");
}

#[test]
fn test_parse_ffprobe_json_output() {
    let sample_ffprobe_json = r#"{
        "streams": [
            {
                "index": 0,
                "codec_name": "h264",
                "codec_type": "video",
                "width": 1280,
                "height": 720,
                "r_frame_rate": "30/1",
                "bit_rate": "2500000"
            },
            {
                "index": 1,
                "codec_name": "aac",
                "codec_type": "audio"
            }
        ],
        "format": {
            "duration": "142.500000"
        }
    }"#;

    let details = parse_ffprobe_json_output(sample_ffprobe_json)
        .expect("Should parse valid ffprobe json");

    assert_eq!(details.resolution, "1280x720 (720p)");
    assert_eq!(details.bitrate, "2,500 kbps");
    assert_eq!(details.video_codec, "h264");
    assert_eq!(details.audio_codec, "aac");
    assert_eq!(details.fps, Some(30));
    assert_eq!(details.duration_secs, Some(142.5));
}

#[test]
fn test_probe_with_ffprobe_arguments_include_hls_image_segment_bypass_and_headers() {
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!("test_ffprobe_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let mock_ffprobe = dir.join("mock_ffprobe.sh");
    let recorded_args = dir.join("args.txt");
    let _ = fs::remove_file(&recorded_args);

    let script = format!(
        r#"#!/bin/sh
for arg in "$@"; do
    printf '%s\0' "$arg" >> "{}"
done
cat << 'EOF'
{{
    "streams": [
        {{
            "codec_name": "h264",
            "codec_type": "video",
            "width": 1920,
            "height": 1080,
            "r_frame_rate": "30/1",
            "bit_rate": "3000000"
        }}
    ]
}}
EOF
exit 0
"#,
        recorded_args.display()
    );

    fs::write(&mock_ffprobe, script).unwrap();
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&mock_ffprobe).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_ffprobe, perms).unwrap();
    }

    let stream_url = "https://vod3.wgslsw.com/20240413/cb6LmDpM/index.m3u8";
    let referer = "https://tw.juqing.app/vod/play/id/oupsenzv-8-1.html";

    let result = bingeget::engine::probe::probe_with_ffprobe(
        mock_ffprobe.to_str().unwrap(),
        stream_url,
        Some(referer),
    );

    assert!(result.is_ok(), "probe_with_ffprobe should succeed with mock ffprobe");

    let args_bytes = fs::read(&recorded_args).unwrap();
    let args: Vec<String> = args_bytes
        .split(|&b| b == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect();

    // Clean up
    let _ = fs::remove_dir_all(&dir);

    // Verify -allowed_segment_extensions ALL
    let ext_pos = args.iter().position(|a| a == "-allowed_segment_extensions");
    assert!(ext_pos.is_some(), "Should pass -allowed_segment_extensions");
    assert_eq!(args.get(ext_pos.unwrap() + 1).map(|s| s.as_str()), Some("ALL"));

    // Verify -extension_picky 0
    let picky_pos = args.iter().position(|a| a == "-extension_picky");
    assert!(picky_pos.is_some(), "Should pass -extension_picky");
    assert_eq!(args.get(picky_pos.unwrap() + 1).map(|s| s.as_str()), Some("0"));

    // Verify -headers contains User-Agent and Referer
    let headers_pos = args.iter().position(|a| a == "-headers");
    assert!(headers_pos.is_some(), "Should pass -headers");
    let headers_val = args.get(headers_pos.unwrap() + 1).expect("Headers value present");
    assert!(headers_val.contains("User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"));
    assert!(headers_val.contains("Referer: https://tw.juqing.app/vod/play/id/oupsenzv-8-1.html"));
}

#[test]
fn test_rewrite_m3u8_for_streaming_proxy() {
    let sample_m3u8 = r#"#EXTM3U
#EXT-X-VERSION:3
#EXTINF:6.000,
seg1.ts
#EXTINF:6.000,
https://other.cdn/seg2.ts
#EXT-X-ENDLIST
"#;

    let rewritten = rewrite_m3u8(
        sample_m3u8,
        "https://cdn.example.com/live/index.m3u8",
        "https://chinaq.fun/drama-1.html",
        8899,
    );

    assert!(rewritten.contains("http://127.0.0.1:8899/proxy?url=https%3A%2F%2Fcdn.example.com%2Flive%2Fseg1.ts"));
    assert!(rewritten.contains("http://127.0.0.1:8899/proxy?url=https%3A%2F%2Fother.cdn%2Fseg2.ts"));
    assert!(rewritten.contains("referer=https%3A%2F%2Fchinaq.fun%2Fdrama-1.html"));
}

#[test]
fn test_rewrite_m3u8_with_encryption_key_and_map_tags() {
    let sample_m3u8 = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXT-X-KEY:METHOD=AES-128,URI="enc.key",IV=0x00000000000000000000000000000000
#EXT-X-MAP:URI="init.mp4"
#EXTINF:10.000000,
segment1.ts
#EXT-X-ENDLIST
"#;

    let rewritten = rewrite_m3u8(
        sample_m3u8,
        "https://hd.kuktxu.com/play/eXDM31ve/index.m3u8",
        "https://chinaq.fun/tv-cn/202639642/ep1.html",
        37099,
    );

    // The encryption key URI must be rewritten to route through our local proxy
    assert!(
        rewritten.contains("URI=\"http://127.0.0.1:37099/proxy?url=https%3A%2F%2Fhd.kuktxu.com%2Fplay%2FeXDM31ve%2Fenc.key&referer=https%3A%2F%2Fchinaq.fun%2Ftv-cn%2F202639642%2Fep1.html\""),
        "Encryption key URI must be rewritten to proxy URL, got:\n{}",
        rewritten
    );

    // The map URI must also be rewritten to route through proxy
    assert!(
        rewritten.contains("URI=\"http://127.0.0.1:37099/proxy?url=https%3A%2F%2Fhd.kuktxu.com%2Fplay%2FeXDM31ve%2Finit.mp4&referer=https%3A%2F%2Fchinaq.fun%2Ftv-cn%2F202639642%2Fep1.html\""),
        "Map URI must be rewritten to proxy URL, got:\n{}",
        rewritten
    );

    // Media segment must also be rewritten
    assert!(
        rewritten.contains("http://127.0.0.1:37099/proxy?url=https%3A%2F%2Fhd.kuktxu.com%2Fplay%2FeXDM31ve%2Fsegment1.ts"),
        "Segment must be rewritten to proxy URL"
    );
}

#[test]
fn test_proxy_client_builder() {
    let client = build_proxy_client();
    // Successfully built reqwest client with timeouts
    let _ = client;
}

#[tokio::test]
async fn test_proxy_timeout_fails_fast_with_504() {
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Spawn a mock hanging upstream server
    let hanging_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let hanging_addr = hanging_listener.local_addr().unwrap();

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = hanging_listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            // Sleep longer than client timeout without sending response
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    // Create client with short timeout (100ms)
    let fast_timeout_client = reqwest::Client::builder()
        .timeout(Duration::from_millis(100))
        .build()
        .unwrap();

    // Create a mock stream connection for proxy handle_connection
    let proxy_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();

    tokio::spawn(async move {
        if let Ok((socket, _)) = proxy_listener.accept().await {
            bingeget::engine::proxy::handle_connection(socket, fast_timeout_client, proxy_addr.port()).await;
        }
    });

    let mut client_stream = tokio::net::TcpStream::connect(proxy_addr).await.unwrap();
    let req = format!(
        "GET /proxy?url=http%3A%2F%2F{}%2Fstream.m3u8 HTTP/1.1\r\nHost: {}\r\n\r\n",
        hanging_addr, proxy_addr
    );
    client_stream.write_all(req.as_bytes()).await.unwrap();

    let mut resp_buf = [0u8; 1024];
    let n = client_stream.read(&mut resp_buf).await.unwrap();
    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);

    assert!(
        resp_str.contains("504 Gateway Timeout"),
        "Proxy should respond with 504 Gateway Timeout on upstream timeout, got: {}",
        resp_str
    );
}

#[tokio::test]
async fn test_proxy_unreachable_upstream_fails_fast_with_502() {
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let fast_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(100))
        .build()
        .unwrap();

    let proxy_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();

    tokio::spawn(async move {
        if let Ok((socket, _)) = proxy_listener.accept().await {
            bingeget::engine::proxy::handle_connection(socket, fast_client, proxy_addr.port()).await;
        }
    });

    // Request an unused local port to trigger connection error
    let mut client_stream = tokio::net::TcpStream::connect(proxy_addr).await.unwrap();
    let req = format!(
        "GET /proxy?url=http%3A%2F%2F127.0.0.1%3A59999%2Fstream.m3u8 HTTP/1.1\r\nHost: {}\r\n\r\n",
        proxy_addr
    );
    client_stream.write_all(req.as_bytes()).await.unwrap();

    let mut resp_buf = [0u8; 1024];
    let n = client_stream.read(&mut resp_buf).await.unwrap();
    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);

    assert!(
        resp_str.contains("502 Bad Gateway"),
        "Proxy should respond with 502 Bad Gateway on connection error, got: {}",
        resp_str
    );
}
