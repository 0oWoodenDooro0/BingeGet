use bingeget::extractors::chinaq::ChinaQExtractor;
use bingeget::extractors::SiteExtractor;

#[test]
fn test_chinaq_can_handle_urls() {
    let extractor = ChinaQExtractor::new();
    assert!(extractor.can_handle("https://chinaq.fun/tv-cn/202614235/"));
    assert!(extractor.can_handle("https://chinaq.fun/tv-kr/202639642/"));
    assert!(extractor.can_handle("https://chinaq.fun/tv-tw/12345/"));
    assert!(extractor.can_handle("https://chinaq.fun/tv-jp/99999/ep1.html"));
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
    assert_eq!(
        extractor.extract_drama_id("https://chinaq.fun/tv-kr/202639642/").unwrap(),
        "202639642"
    );
    assert_eq!(
        extractor.extract_drama_id("https://chinaq.fun/tv-tw/12345/").unwrap(),
        "12345"
    );
    assert_eq!(
        extractor.extract_drama_id("https://chinaq.fun/tv-jp/99999/ep3.html").unwrap(),
        "99999"
    );
    assert!(extractor.extract_drama_id("https://chinaq.fun/invalid-url").is_err());
}

#[test]
fn test_chinaq_category_url_matrix() {
    let extractor = ChinaQExtractor::new();
    let categories = ["tv-cn", "tv-kr", "tv-tw", "tv-jp", "tv-hk", "tv-th"];

    for cat in categories {
        let drama_id = "20269999";
        let base_url_trailing = format!("https://chinaq.fun/{}/{}/", cat, drama_id);
        let base_url_no_trailing = format!("https://chinaq.fun/{}/{}", cat, drama_id);
        let ep_url = format!("https://chinaq.fun/{}/{}/ep5.html", cat, drama_id);

        // 1. can_handle check
        assert!(extractor.can_handle(&base_url_trailing), "Should handle {}", base_url_trailing);
        assert!(extractor.can_handle(&base_url_no_trailing), "Should handle {}", base_url_no_trailing);
        assert!(extractor.can_handle(&ep_url), "Should handle {}", ep_url);

        // 2. extract_category_and_drama_id check
        let (extracted_cat, extracted_id) = extractor
            .extract_category_and_drama_id(&base_url_trailing)
            .unwrap_or_else(|_| panic!("Failed to extract from {}", base_url_trailing));
        assert_eq!(extracted_cat, cat);
        assert_eq!(extracted_id, drama_id);

        let (extracted_cat_ep, extracted_id_ep) = extractor
            .extract_category_and_drama_id(&ep_url)
            .unwrap_or_else(|_| panic!("Failed to extract from {}", ep_url));
        assert_eq!(extracted_cat_ep, cat);
        assert_eq!(extracted_id_ep, drama_id);

        // 3. build_episode_referer check
        let built_referer = extractor.build_episode_referer(cat, drama_id, "ep5");
        assert_eq!(built_referer, ep_url);
    }
}

#[test]
fn test_chinaq_category_label_mappings() {
    assert_eq!(ChinaQExtractor::category_label_from_code("cn").as_deref(), Some("陸劇"));
    assert_eq!(ChinaQExtractor::category_label_from_code("kr").as_deref(), Some("韓劇"));
    assert_eq!(ChinaQExtractor::category_label_from_code("tw").as_deref(), Some("台劇"));
    assert_eq!(ChinaQExtractor::category_label_from_code("jp").as_deref(), Some("日劇"));
    assert_eq!(ChinaQExtractor::category_label_from_code("hk").as_deref(), Some("港劇"));
    assert_eq!(ChinaQExtractor::category_label_from_code("th").as_deref(), Some("泰劇"));
    assert_eq!(ChinaQExtractor::category_label_from_code("anime").as_deref(), Some("動漫"));
    assert_eq!(ChinaQExtractor::category_label_from_code("movie").as_deref(), Some("電影"));
    assert_eq!(ChinaQExtractor::category_label_from_code("").as_deref(), None);
}

#[test]
fn test_chinaq_parse_episodes_from_html() {
    let extractor = ChinaQExtractor::new();
    let sample_html = r#"
        <div class="category clearfix">
            <h2><a href="/tv-kr/202639642/ep3.html">Doctor X 第3集</a></h2>
            <h2><a href="/tv-kr/202639642/ep1.html">Doctor X 第1集</a></h2>
            <h2><a href="/tv-kr/202639642/ep2.html">Doctor X 第2集</a></h2>
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

#[test]
fn test_chinaq_parse_search_results() {
    let extractor = ChinaQExtractor::new();
    let sample_all_html = r#"
        <ul class="drama_list">
            <li cat="cn">
                <a href="/tv-cn/202312271/">
                    繁花
                    (2023)
                </a>
            </li>
            <li cat="kr">
                <a href="/tv-kr/202625953/">
                    殭屍校園 第二季 지금 우리 학교는
                    (2026)
                </a>
            </li>
            <li cat="tw">
                <a href="/tv-tw/202209121/">
                    繁星點點
                </a>
            </li>
            <li cat="jp">
                <a href="/tv-jp/202639642/">
                    Doctor X
                    (2021)
                </a>
            </li>
            <li cat="kr">
                <a href="/tv-kr/202106171/">
                    機智醫生生活
                    (2020)
                </a>
            </li>
        </ul>
    "#;

    // 1. Exact match keyword "繁花"
    let results = extractor.parse_search_results(sample_all_html, "繁花").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "繁花");
    assert_eq!(results[0].year.as_deref(), Some("2023"));
    assert_eq!(results[0].category.as_deref(), Some("陸劇"));
    assert_eq!(results[0].url, "https://chinaq.fun/tv-cn/202312271/");
    assert_eq!(results[0].source_id, "chinaq");
    assert_eq!(results[0].source_name, "ChinaQ");

    // 2. Simplified Chinese query on Traditional title: "机智医生" -> "機智醫生生活"
    let s2t_results = extractor.parse_search_results(sample_all_html, "机智医生").unwrap();
    assert_eq!(s2t_results.len(), 1);
    assert_eq!(s2t_results[0].title, "機智醫生生活");

    // 3. Full-width ASCII query: "Ｄｏｃｔｏｒ" -> "Doctor X"
    let fullwidth_results = extractor.parse_search_results(sample_all_html, "Ｄｏｃｔｏｒ").unwrap();
    assert_eq!(fullwidth_results.len(), 1);
    assert_eq!(fullwidth_results[0].title, "Doctor X");

    // 4. Case-insensitive English: "doctor x" -> "Doctor X"
    let lower_results = extractor.parse_search_results(sample_all_html, "doctor x").unwrap();
    assert_eq!(lower_results.len(), 1);
    assert_eq!(lower_results[0].title, "Doctor X");

    // 5. Partial match "繁" matches both 繁花 and 繁星點點
    let multi_results = extractor.parse_search_results(sample_all_html, "繁").unwrap();
    assert_eq!(multi_results.len(), 2);

    // 6. No match
    let empty_results = extractor.parse_search_results(sample_all_html, "不存在的劇名").unwrap();
    assert!(empty_results.is_empty());
}

#[tokio::test]
async fn test_chinaq_live_metadata_and_m3u8_fetch() {
    let extractor = ChinaQExtractor::new();
    let client = reqwest::Client::new();
    let url = "https://chinaq.fun/tv-kr/202639642/";

    let metadata = extractor
        .fetch_series_metadata(&client, url)
        .await
        .expect("Live series metadata should succeed for tv-kr");

    assert_eq!(metadata.drama_id, "202639642");
    assert!(!metadata.episodes.is_empty(), "Should parse episodes");
    assert!(!metadata.channels.is_empty(), "Should parse available channels");
}
