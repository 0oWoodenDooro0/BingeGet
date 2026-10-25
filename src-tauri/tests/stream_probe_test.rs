use bingeget::engine::probe::{parse_m3u8_master_tags, parse_ffprobe_json_output};

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
    assert_eq!(details.has_master_playlist, true);
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
                "bit_rate": "1750000"
            },
            {
                "index": 1,
                "codec_name": "aac",
                "codec_type": "audio",
                "bit_rate": "128000"
            }
        ]
    }"#;

    let details = parse_ffprobe_json_output(sample_ffprobe_json)
        .expect("Should parse ffprobe json output");

    assert_eq!(details.resolution, "1280x720 (720p)");
    assert_eq!(details.bitrate, "1,750 kbps");
    assert_eq!(details.fps, Some(30));
    assert_eq!(details.has_master_playlist, false);
    assert_eq!(details.video_codec, "h264");
    assert_eq!(details.audio_codec, "aac");
}
