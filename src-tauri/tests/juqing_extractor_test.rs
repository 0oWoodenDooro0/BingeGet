use bingeget::extractors::juqing::JuqingExtractor;
use bingeget::extractors::SiteExtractor;

#[test]
fn test_juqing_can_handle_urls() {
    let extractor = JuqingExtractor::new();

    // Valid detail URLs
    assert!(extractor.can_handle("https://juqing.app/vod/detail/id/gb27z2th.html"));
    assert!(extractor.can_handle("https://www.juqing.app/vod/detail/id/12345.html"));
    assert!(extractor.can_handle("https://taiwan.juqing.app/vod/detail/id/gb27z2th.html"));
    assert!(extractor.can_handle("http://juqing.app/vod/detail/id/9rm707ak.html"));
    assert!(extractor.can_handle("https://juqing.app/vod/detail/id/gb27z2th.html?src=home"));

    // Valid play URLs
    assert!(extractor.can_handle("https://juqing.app/vod/play/id/gb27z2th-1-1.html"));
    assert!(extractor.can_handle("https://hk.juqing.app/vod/play/id/12345-7-2.html"));
    assert!(extractor.can_handle("https://juqing.app/vod/play/id/gb27z2th-6-36.html"));

    // Invalid or foreign domains
    assert!(!extractor.can_handle("https://example.com/vod/detail/id/gb27z2th.html"));
    assert!(!extractor.can_handle("https://chinaq.fun/tv-cn/202614235/"));
    assert!(!extractor.can_handle("https://notjuqing.app/vod/detail/id/123.html"));
    assert!(!extractor.can_handle("https://juqing.app.malicious.com/vod/detail/id/123.html"));
    assert!(!extractor.can_handle("https://juqing.app/vod/other/id/123.html"));
    assert!(!extractor.can_handle("https://juqing.app/vod/detail/id/"));
    assert!(!extractor.can_handle("https://juqing.app/vod/play/id/123-1.html"));
    assert!(!extractor.can_handle("https://juqing.app/"));
    assert!(!extractor.can_handle(""));
}

#[test]
fn test_juqing_extract_drama_id_and_normalize_url() {
    let extractor = JuqingExtractor::new();

    // Detail URL
    assert_eq!(
        extractor
            .extract_drama_id("https://juqing.app/vod/detail/id/gb27z2th.html")
            .unwrap(),
        "gb27z2th"
    );

    // Play URL
    assert_eq!(
        extractor
            .extract_drama_id("https://juqing.app/vod/play/id/gb27z2th-1-1.html")
            .unwrap(),
        "gb27z2th"
    );

    assert_eq!(
        extractor
            .extract_drama_id("https://taiwan.juqing.app/vod/play/id/3s748zkk-7-15.html")
            .unwrap(),
        "3s748zkk"
    );

    // Invalid URL error
    assert!(extractor.extract_drama_id("https://juqing.app/invalid/url").is_err());

    // Normalize play URL to detail URL
    assert_eq!(
        extractor
            .normalize_detail_url("https://juqing.app/vod/play/id/gb27z2th-1-1.html")
            .unwrap(),
        "https://juqing.app/vod/detail/id/gb27z2th.html"
    );

    assert_eq!(
        extractor
            .normalize_detail_url("https://taiwan.juqing.app/vod/detail/id/gb27z2th.html")
            .unwrap(),
        "https://taiwan.juqing.app/vod/detail/id/gb27z2th.html"
    );
}

#[test]
fn test_juqing_build_episode_referer() {
    let extractor = JuqingExtractor::new();

    let play_referer = extractor.build_episode_referer("陸劇", "gb27z2th", "1");
    assert!(
        play_referer == "https://juqing.app/vod/play/id/gb27z2th-1-1.html"
            || play_referer == "https://juqing.app/vod/detail/id/gb27z2th.html"
    );

    let detail_referer = extractor.build_episode_referer("", "gb27z2th", "");
    assert_eq!(detail_referer, "https://juqing.app/vod/detail/id/gb27z2th.html");
}

#[test]
fn test_juqing_extractor_trait_properties() {
    let extractor = JuqingExtractor::new();
    assert_eq!(extractor.id(), "juqing");
    assert_eq!(extractor.name(), "小鴨影視");
}

#[test]
fn test_juqing_parse_metadata_offline_fixture() {
    let extractor = JuqingExtractor::new();
    let mock_html = r#"
<!DOCTYPE html>
<html lang="zh-Hant-TW">
<head>
    <title>慶餘年第二季線上看 - 小鴨影視</title>
    <meta property="og:title" content="慶餘年第二季線上看 - 小鴨影視">
    <meta property="og:image" content="https://image.juqing.app/vod/1c/01c92e9c3dd9bec34e0c7dfa022e2f24.webp">
    <meta property="og:video:class" content="陸劇">
</head>
<body>
    <div class="shoutu-media-bd">
        <h1><a href="/vod/detail/id/gb27z2th.html" title="慶餘年第二季線上看">慶餘年第二季</a></h1>
    </div>

    <!-- Tab panel 1 -->
    <div class="panel-hd play-name"><span class="play-from">1080P雲2</span></div>
    <div class="panel-bd play-list-link">
        <ul class="shoutu-playlist sort-list playlist">
            <li><a title="慶餘年第二季第01集線上看" href="/vod/play/id/gb27z2th-7-1.html">第01集</a></li>
            <li><a title="慶餘年第二季第02集線上看" href="/vod/play/id/gb27z2th-7-2.html">第02集</a></li>
            <li><a title="慶餘年第二季第03集線上看" href="/vod/play/id/gb27z2th-7-3.html">第03集</a></li>
        </ul>
    </div>

    <!-- Tab panel 2 -->
    <div class="panel-hd play-name">極速快播</div>
    <div class="panel-bd play-list-link">
        <ul class="shoutu-playlist sort-list playlist">
            <li><a title="慶餘年第二季第01集線上看" href="/vod/play/id/gb27z2th-9-1.html">第01集</a></li>
            <li><a title="慶餘年第二季第02集線上看" href="/vod/play/id/gb27z2th-9-2.html">第02集</a></li>
        </ul>
    </div>
</body>
</html>
"#;

    let metadata = extractor
        .parse_metadata_from_html(mock_html, "gb27z2th")
        .expect("Should parse metadata successfully");

    assert_eq!(metadata.drama_id, "gb27z2th");
    assert_eq!(metadata.source_id, "juqing");
    assert_eq!(metadata.category_slug, Some("陸劇".to_string()));
    assert_eq!(
        metadata.cover_url,
        Some("https://image.juqing.app/vod/1c/01c92e9c3dd9bec34e0c7dfa022e2f24.webp".to_string())
    );

    // Channels
    assert_eq!(metadata.channels.len(), 2);
    assert_eq!(metadata.channels[0].id, "7");
    assert_eq!(metadata.channels[0].name, "1080P雲2");
    assert_eq!(metadata.channels[1].id, "9");
    assert_eq!(metadata.channels[1].name, "極速快播");

    // Episodes
    assert_eq!(metadata.episodes.len(), 3);

    // Episode 1
    assert_eq!(metadata.episodes[0].ep_number, 1);
    assert_eq!(metadata.episodes[0].ep_id, "1");
    assert_eq!(metadata.episodes[0].available_channels, vec!["7", "9"]);

    // Episode 2
    assert_eq!(metadata.episodes[1].ep_number, 2);
    assert_eq!(metadata.episodes[1].ep_id, "2");
    assert_eq!(metadata.episodes[1].available_channels, vec!["7", "9"]);

    // Episode 3
    assert_eq!(metadata.episodes[2].ep_number, 3);
    assert_eq!(metadata.episodes[2].ep_id, "3");
    assert_eq!(metadata.episodes[2].available_channels, vec!["7"]);
}
