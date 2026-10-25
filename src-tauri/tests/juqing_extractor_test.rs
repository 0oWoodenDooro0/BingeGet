use bingeget::engine::downloader::{build_download_args, DownloadConfig, DownloadTaskSpec};
use bingeget::engine::probe::probe_stream_core;
use bingeget::engine::proxy::build_proxy_url;
use bingeget::extractors::juqing::JuqingExtractor;
use bingeget::extractors::SiteExtractor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
fn test_juqing_build_stream_referer() {
    let extractor = JuqingExtractor::new();

    // Uniform build_stream_referer with channel and optional category_slug
    let play_referer = extractor.build_stream_referer("gb27z2th", "1", "1", None);
    assert_eq!(play_referer, "https://tw.juqing.app/vod/play/id/gb27z2th-1-1.html");

    let play_referer_ch7 = extractor.build_stream_referer("gb27z2th", "ep2", "7", Some("tv-cn"));
    assert_eq!(play_referer_ch7, "https://tw.juqing.app/vod/play/id/gb27z2th-7-2.html");

    // Trait object call verification
    let trait_extractor: &dyn SiteExtractor = &extractor;
    let via_trait = trait_extractor.build_stream_referer("gb27z2th", "ep3", "ikzy", Some("tv-kr"));
    assert_eq!(via_trait, "https://tw.juqing.app/vod/play/id/gb27z2th-ikzy-3.html");
}

#[test]
fn test_juqing_regional_category_matrix() {
    let extractor = JuqingExtractor::new();
    let regional_slugs = ["tv-cn", "tv-kr", "tv-tw", "tv-jp", "tv-hk"];

    for slug in regional_slugs {
        // 1. Regional slug to category label mapping and reverse mapping
        let label = JuqingExtractor::category_label_from_slug(slug);
        assert!(label.is_some(), "Should map regional slug {} to label", slug);
        let reverse_slug = JuqingExtractor::slug_from_category(label.unwrap());
        assert_eq!(reverse_slug, Some(slug), "Reverse category mapping should match for {}", slug);

        // 2. Metadata HTML category parsing with regional slug
        let mock_html_slug = format!(
            r#"<!DOCTYPE html><html><head><meta property="og:video:class" content="{}"></head><body></body></html>"#,
            slug
        );
        let parsed_slug = JuqingExtractor::parse_category(&mock_html_slug);
        assert_eq!(parsed_slug, Some(slug.to_string()));

        // 3. Metadata HTML category parsing with regional Chinese label
        let mock_html_label = format!(
            r#"<!DOCTYPE html><html><head><meta property="og:video:class" content="{}"></head><body></body></html>"#,
            label.unwrap()
        );
        let parsed_label = JuqingExtractor::parse_category(&mock_html_label);
        assert_eq!(parsed_label, Some(label.unwrap().to_string()));

        // 4. Uniform build_stream_referer with regional category slug
        let referer = extractor.build_stream_referer("drama999", "1", "ikzy", Some(slug));
        assert_eq!(referer, "https://tw.juqing.app/vod/play/id/drama999-ikzy-1.html");
    }
}

#[test]
fn test_juqing_build_episode_referer() {
    let extractor = JuqingExtractor::new();

    // Channel 1, ep 1
    let play_referer = extractor.build_episode_referer("1", "gb27z2th", "1");
    assert_eq!(play_referer, "https://tw.juqing.app/vod/play/id/gb27z2th-1-1.html");

    // Channel 7, ep ep2 -> stripped to 2
    let play_referer_ch7 = extractor.build_episode_referer("7", "gb27z2th", "ep2");
    assert_eq!(play_referer_ch7, "https://tw.juqing.app/vod/play/id/gb27z2th-7-2.html");

    // Channel missing -> detail URL fallback
    let missing_channel = extractor.build_episode_referer("", "gb27z2th", "1");
    assert_eq!(missing_channel, "https://tw.juqing.app/vod/detail/id/gb27z2th.html");

    // Episode missing -> detail URL fallback
    let missing_ep = extractor.build_episode_referer("1", "gb27z2th", "");
    assert_eq!(missing_ep, "https://tw.juqing.app/vod/detail/id/gb27z2th.html");

    // Both missing -> detail URL
    let both_missing = extractor.build_episode_referer("", "gb27z2th", "");
    assert_eq!(both_missing, "https://tw.juqing.app/vod/detail/id/gb27z2th.html");
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

#[test]
fn test_juqing_parse_m3u8_offline_fixture() {
    let mock_html = r#"
<!DOCTYPE html>
<html>
<head><title>Play Video</title></head>
<body>
    <div class="player-wrapper">
        <video id="my-player" class="video-js" controls preload="auto">
            <source src="https://example.com/stream.m3u8" type="application/x-mpegURL">
        </video>
    </div>
</body>
</html>
"#;

    let stream_url = JuqingExtractor::parse_m3u8_from_html(mock_html, None)
        .expect("Should extract m3u8 url from single source tag");
    assert_eq!(stream_url, "https://example.com/stream.m3u8");
}

#[test]
fn test_juqing_parse_m3u8_multiple_sources_fallback_and_channel_matching() {
    let mock_html = r#"
<!DOCTYPE html>
<html>
<body>
    <video id="my-player">
        <source src="https://example.com/ch7.m3u8" channel="7" type="application/x-mpegURL">
        <source src="https://example.com/ch9.m3u8" channel="9" type="application/x-mpegURL">
    </video>
</body>
</html>
"#;

    // Matching channel 9
    let res9 = JuqingExtractor::parse_m3u8_from_html(mock_html, Some("9"))
        .expect("Should match channel 9");
    assert_eq!(res9, "https://example.com/ch9.m3u8");

    // Matching channel 7
    let res7 = JuqingExtractor::parse_m3u8_from_html(mock_html, Some("7"))
        .expect("Should match channel 7");
    assert_eq!(res7, "https://example.com/ch7.m3u8");

    // Channel Lock Compliance (ADR-0001): unmatched requested channel MUST return an Err
    let unmatched = JuqingExtractor::parse_m3u8_from_html(mock_html, Some("99"));
    assert!(unmatched.is_err(), "Unmatched requested channel must return Err instead of silent fallback");

    // Empty or None requested channel can default to the first source
    let default_empty = JuqingExtractor::parse_m3u8_from_html(mock_html, Some(""))
        .expect("Empty requested channel should default to first source");
    assert_eq!(default_empty, "https://example.com/ch7.m3u8");

    let default_none = JuqingExtractor::parse_m3u8_from_html(mock_html, None)
        .expect("None requested channel should default to first source");
    assert_eq!(default_none, "https://example.com/ch7.m3u8");
}

#[tokio::test]
async fn test_juqing_fetch_m3u8_for_episode_offline_server() {
    let mock_html = r#"
<!DOCTYPE html>
<html>
<body>
    <video id="my-player">
        <source src="https://example.com/juqing-stream.m3u8" channel="7" type="application/x-mpegURL">
    </video>
</body>
</html>
"#;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_task = tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                mock_html.len(),
                mock_html
            );
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    let client = reqwest::Client::new();
    let extractor = JuqingExtractor::new();
    let mock_url = format!("http://{}", addr);

    let stream = extractor
        .fetch_m3u8_for_episode(&client, &mock_url, "1", "7")
        .await
        .expect("fetch_m3u8_for_episode should succeed with offline server");

    assert_eq!(stream, "https://example.com/juqing-stream.m3u8");
    let _ = server_task.await;
}

#[tokio::test]
async fn test_juqing_downstream_probe_and_proxy_and_download_pipeline() {
    let client = reqwest::Client::new();
    let extractor = JuqingExtractor::new();

    // 1. Verify Referer generation for downstream pipelines
    let referer = extractor.build_episode_referer("7", "gb27z2th", "1");
    assert_eq!(referer, "https://tw.juqing.app/vod/play/id/gb27z2th-7-1.html");

    // 2. Verify streaming proxy URL construction with Juqing Referer
    let proxy_url = build_proxy_url(18888, "https://example.com/stream.m3u8", &referer);
    assert!(proxy_url.contains("http://127.0.0.1:18888/proxy"));
    assert!(proxy_url.contains("url=https%3A%2F%2Fexample.com%2Fstream.m3u8"));
    assert!(proxy_url.contains("referer=https%3A%2F%2Ftw.juqing.app%2Fvod%2Fplay%2Fid%2Fgb27z2th-7-1.html"));

    // 3. Verify download args construction for n-m3u8dl-re with Juqing Referer
    let task_spec = DownloadTaskSpec {
        m3u8_url: "https://example.com/stream.m3u8".to_string(),
        custom_title: "慶餘年第二季".to_string(),
        season: 2,
        episode: 1,
        save_dir: "/downloads".to_string(),
        referer: referer.clone(),
    };
    let config = DownloadConfig::default();
    let args = build_download_args(&task_spec, &config);

    assert!(args.contains(&format!("Referer: {}", referer)));
    assert!(args.contains(&"--header".to_string()));

    // 4. Verify probe_stream_core with offline mock server for Juqing
    let mock_m3u8 = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=2500000,RESOLUTION=1920x1080\nchunklist.m3u8\n";
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_task = tokio::spawn(async move {
        // Handle play page request
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

        // Handle m3u8 playlist request
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\n\r\n{}",
                mock_m3u8.len(),
                mock_m3u8
            );
            let _ = socket.write_all(resp.as_bytes()).await;
        }
    });

    let mock_drama_url = format!("http://{}", addr);
    let probe_details = probe_stream_core(
        &client,
        &mock_drama_url,
        "1",
        "7",
        None,
        Some("juqing"),
    )
    .await
    .expect("Should probe Juqing stream successfully");

    assert_eq!(probe_details.resolution, "1920x1080 (1080p)");
    assert_eq!(probe_details.bitrate, "2,500 kbps");
    let _ = server_task.await;
}

#[test]
fn test_juqing_parse_search_results_offline_fixture() {
    let extractor = JuqingExtractor::new();
    let mock_search_html = r#"
<!DOCTYPE html>
<html lang="zh-Hant-TW">
<head><title>慶餘年 - 小鴨影視</title></head>
<body>
<div class="search-result">
    <ul>
        <li class="col9">
            <article itemscope="" itemtype="https://schema.org/Movie">
                <a class="cover-img" itemprop="url" href="https://tw.juqing.app/vod/detail/id/gb27z2th.html" title="慶餘年第二季線上看" rel="bookmark">
                    <img class="lazyload" itemprop="image" data-src="https://image.juqing.app/vod/1c/01c92e9c3dd9bec34e0c7dfa022e2f24.webp" src="/template/xiaoya/img/loading.webp" alt="慶餘年第二季線上看">
                </a>
                <div class="cover-data">
                    <h4 itemprop="headline" class="title">
                        <a href="https://tw.juqing.app/vod/detail/id/gb27z2th.html" title="慶餘年第二季" rel="bookmark">慶餘年第二季</a>
                    </h4>
                    <meta itemprop="name" content="慶餘年第二季">
                    <meta itemprop="dateCreated" content="2024">
                    <meta itemprop="genre" content="劇情,古裝,國產">
                </div>
            </article>
        </li>
        <li class="col9">
            <article itemscope="" itemtype="https://schema.org/Movie">
                <a class="cover-img" itemprop="url" href="/vod/detail/id/rg5u54tc.html" title="慶餘年第一季線上看" rel="bookmark">
                    <img class="cover-img" itemprop="image" src="https://image.juqing.app/vod/8d/98d41fc6640907a78162a36fff239126.webp" alt="慶餘年第一季線上看">
                </a>
                <div class="cover-data">
                    <h4 itemprop="headline" class="title">
                        <a href="/vod/detail/id/rg5u54tc.html" title="慶餘年第一季" rel="bookmark">慶餘年第一季</a>
                    </h4>
                    <meta itemprop="name" content="慶餘年第一季">
                    <meta itemprop="dateCreated" content="2019">
                    <meta itemprop="genre" content="陸劇">
                </div>
            </article>
        </li>
    </ul>
</div>
</body>
</html>
"#;

    let results = extractor
        .parse_search_results(mock_search_html)
        .expect("Should parse search results successfully");

    assert_eq!(results.len(), 2);

    let first = &results[0];
    assert_eq!(first.title, "慶餘年第二季");
    assert_eq!(first.year, Some("2024".to_string()));
    assert_eq!(first.category, Some("劇情,古裝,國產".to_string()));
    assert_eq!(first.url, "https://tw.juqing.app/vod/detail/id/gb27z2th.html");
    assert_eq!(first.source_id, "juqing");
    assert_eq!(first.source_name, "小鴨影視");
    assert_eq!(
        first.cover_url,
        Some("https://image.juqing.app/vod/1c/01c92e9c3dd9bec34e0c7dfa022e2f24.webp".to_string())
    );

    let second = &results[1];
    assert_eq!(second.title, "慶餘年第一季");
    assert_eq!(second.year, Some("2019".to_string()));
    assert_eq!(second.category, Some("陸劇".to_string()));
    assert!(second.url.ends_with("/vod/detail/id/rg5u54tc.html"));
    assert_eq!(second.source_id, "juqing");
    assert_eq!(second.source_name, "小鴨影視");
    assert_eq!(
        second.cover_url,
        Some("https://image.juqing.app/vod/8d/98d41fc6640907a78162a36fff239126.webp".to_string())
    );
}

#[tokio::test]
async fn test_juqing_search_live_or_keyword() {
    let extractor = JuqingExtractor::new();
    let client = reqwest::Client::new();
    let res = extractor.search(&client, "慶餘年").await;
    match res {
        Ok(results) => {
            assert!(!results.is_empty(), "Search for '慶餘年' should return results from juqing");
            for item in &results {
                assert_eq!(item.source_id, "juqing");
                assert_eq!(item.source_name, "小鴨影視");
                assert!(item.url.contains("/vod/detail/id/"));
            }
        }
        Err(e) => {
            eprintln!("Live juqing search network error or timeout: {}", e);
        }
    }
}
