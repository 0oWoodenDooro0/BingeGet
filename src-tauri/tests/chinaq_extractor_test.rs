use bingeget::extractors::chinaq::ChinaQExtractor;
use bingeget::extractors::SiteExtractor;

#[test]
fn test_chinaq_can_handle_urls() {
    let extractor = ChinaQExtractor::new();
    assert!(extractor.can_handle("https://chinaq.fun/tv-cn/202614235/"));
    assert!(extractor.can_handle("https://chinaq.fun/tv-cn/202614235/ep1.html"));
    assert!(extractor.can_handle("http://chinaq.fun/tv-cn/12345/"));
    assert!(!extractor.can_handle("https://example.com/movie/123"));
}

#[test]
fn test_chinaq_extract_drama_id() {
    let extractor = ChinaQExtractor::new();
    assert_eq!(
        extractor.extract_drama_id("https://chinaq.fun/tv-cn/202614235/").unwrap(),
        "202614235"
    );
    assert_eq!(
        extractor.extract_drama_id("https://chinaq.fun/tv-cn/202614235/ep12.html").unwrap(),
        "202614235"
    );
    assert!(extractor.extract_drama_id("https://chinaq.fun/invalid-url").is_err());
}

#[test]
fn test_chinaq_parse_episodes_from_html() {
    let extractor = ChinaQExtractor::new();
    let sample_html = r#"
        <div class="category clearfix">
            <h2><a href="/tv-cn/202614235/ep3.html">花開錦繡 第3集</a></h2>
            <h2><a href="/tv-cn/202614235/ep1.html">花開錦繡 第1集</a></h2>
            <h2><a href="/tv-cn/202614235/ep2.html">花開錦繡 第2集</a></h2>
        </div>
    "#;

    let episodes = extractor.parse_episodes_from_html(sample_html).expect("Should parse episodes");
    assert_eq!(episodes.len(), 3);
    // Should be sorted ascending by episode number
    assert_eq!(episodes[0].ep_number, 1);
    assert_eq!(episodes[0].ep_id, "ep1");
    assert_eq!(episodes[1].ep_number, 2);
    assert_eq!(episodes[1].ep_id, "ep2");
    assert_eq!(episodes[2].ep_number, 3);
    assert_eq!(episodes[2].ep_id, "ep3");
}

#[test]
fn test_chinaq_extract_channels_from_qplays() {
    let extractor = ChinaQExtractor::new();
    let sample_qplays_json = r#"{
        "video_plays": [
            {
                "play_data": "https://hn.bfvvs.com/play/eVO78xMa/index.m3u8",
                "src_site": "hnzy",
                "language": null
            },
            {
                "play_data": "https://vv.jisuzyv.com/play/e1wGmlRb/index.m3u8",
                "src_site": "jszy",
                "language": null
            },
            {
                "play_data": "https://bfikuncdn.com/20260809/XpZySbA2/index.m3u8",
                "src_site": "ikzy",
                "language": null
            }
        ]
    }"#;

    let channels = extractor.parse_available_channels(sample_qplays_json).unwrap();
    assert_eq!(channels.len(), 3);
    assert_eq!(channels[0].id, "hnzy");
    assert_eq!(channels[1].id, "jszy");
    assert_eq!(channels[2].id, "ikzy");
}

#[test]
fn test_chinaq_extract_m3u8_with_channel_lock() {
    let extractor = ChinaQExtractor::new();
    let sample_qplays_json = r#"{
        "video_plays": [
            {
                "play_data": "https://hn.bfvvs.com/play/eVO78xMa/index.m3u8",
                "src_site": "hnzy",
                "language": null
            },
            {
                "play_data": "https://bfikuncdn.com/20260809/XpZySbA2/index.m3u8",
                "src_site": "ikzy",
                "language": null
            }
        ]
    }"#;

    // Lock to ikzy
    let m3u8_ikzy = extractor.extract_m3u8_for_channel(sample_qplays_json, "ikzy");
    assert_eq!(m3u8_ikzy.unwrap(), "https://bfikuncdn.com/20260809/XpZySbA2/index.m3u8");

    // Lock to hnzy
    let m3u8_hnzy = extractor.extract_m3u8_for_channel(sample_qplays_json, "hnzy");
    assert_eq!(m3u8_hnzy.unwrap(), "https://hn.bfvvs.com/play/eVO78xMa/index.m3u8");

    // Missing channel
    let m3u8_missing = extractor.extract_m3u8_for_channel(sample_qplays_json, "nonexistent");
    assert!(m3u8_missing.is_err());
}

#[tokio::test]
async fn test_chinaq_live_metadata_and_m3u8_fetch() {
    let extractor = ChinaQExtractor::new();
    let client = reqwest::Client::new();
    let url = "https://chinaq.fun/tv-cn/202614235/";

    let metadata = extractor
        .fetch_series_metadata(&client, url)
        .await
        .expect("Live series metadata should succeed");

    assert_eq!(metadata.drama_id, "202614235");
    assert!(!metadata.episodes.is_empty(), "Should parse episodes");
    assert!(!metadata.channels.is_empty(), "Should parse available channels");

    // Try fetching ep1 m3u8 with the first available channel
    let target_channel = &metadata.channels[0].id;
    let m3u8_url = extractor
        .fetch_m3u8_for_episode(&client, &metadata.drama_id, &metadata.episodes[0].ep_id, target_channel)
        .await
        .expect("Should fetch live m3u8 url");

    assert!(m3u8_url.ends_with(".m3u8") || m3u8_url.contains("m3u8"));
}
