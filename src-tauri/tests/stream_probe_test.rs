use bingeget::engine::probe::{parse_ffprobe_json_output, parse_m3u8_master_tags};
use bingeget::engine::proxy::rewrite_m3u8;

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
