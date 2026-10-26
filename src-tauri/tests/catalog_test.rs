use std::sync::Arc;
use async_trait::async_trait;
use bingeget::extractors::catalog::{global_catalog, Catalog};
use bingeget::extractors::SiteExtractor;
use bingeget::models::{SearchResult, SeriesMetadata};

struct DummyExtractor {
    id: &'static str,
    name: &'static str,
    domain: &'static str,
}

#[async_trait]
impl SiteExtractor for DummyExtractor {
    fn id(&self) -> &'static str {
        self.id
    }
    fn name(&self) -> &'static str {
        self.name
    }
    fn can_handle(&self, url: &str) -> bool {
        url.contains(self.domain)
    }
    fn build_episode_referer(&self, category_slug: &str, drama_id: &str, ep_id: &str) -> String {
        format!("https://{}/{}/{}/{}", self.domain, category_slug, drama_id, ep_id)
    }
    fn build_stream_referer(&self, drama_id: &str, ep_id: &str, _channel: &str, category_slug: Option<&str>) -> String {
        format!("https://{}/{}/{}/{}", self.domain, category_slug.unwrap_or("tv-cn"), drama_id, ep_id)
    }
    async fn fetch_series_metadata(&self, _client: &reqwest::Client, _url: &str) -> anyhow::Result<SeriesMetadata> {
        Ok(SeriesMetadata {
            drama_id: "dummy123".to_string(),
            source_id: self.id.to_string(),
            category_slug: Some("test".to_string()),
            cover_url: None,
            episodes: Vec::new(),
            channels: Vec::new(),
        })
    }
    async fn fetch_m3u8_for_episode(&self, _client: &reqwest::Client, _drama_id: &str, _ep_id: &str, _channel: &str) -> anyhow::Result<String> {
        Ok("https://dummy.stream/index.m3u8".to_string())
    }
    async fn search(&self, _client: &reqwest::Client, _keyword: &str) -> anyhow::Result<Vec<SearchResult>> {
        Ok(vec![SearchResult {
            title: "Dummy Title".to_string(),
            year: Some("2024".to_string()),
            category: Some("Drama".to_string()),
            url: format!("https://{}/drama/1", self.domain),
            source_id: self.id.to_string(),
            source_name: self.name.to_string(),
            cover_url: None,
        }])
    }
}

struct FailingExtractor;

#[async_trait]
impl SiteExtractor for FailingExtractor {
    fn id(&self) -> &'static str {
        "failing"
    }
    fn name(&self) -> &'static str {
        "Failing Source"
    }
    fn can_handle(&self, _url: &str) -> bool {
        false
    }
    fn build_episode_referer(&self, _category_slug: &str, _drama_id: &str, _ep_id: &str) -> String {
        String::new()
    }
    fn build_stream_referer(&self, _drama_id: &str, _ep_id: &str, _channel: &str, _category_slug: Option<&str>) -> String {
        String::new()
    }
    async fn fetch_series_metadata(&self, _client: &reqwest::Client, _url: &str) -> anyhow::Result<SeriesMetadata> {
        Err(anyhow::anyhow!("Metadata failed"))
    }
    async fn fetch_m3u8_for_episode(&self, _client: &reqwest::Client, _drama_id: &str, _ep_id: &str, _channel: &str) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("M3u8 failed"))
    }
    async fn search(&self, _client: &reqwest::Client, _keyword: &str) -> anyhow::Result<Vec<SearchResult>> {
        Err(anyhow::anyhow!("Network timeout simulated"))
    }
}

#[test]
fn test_catalog_default_and_lookup() {
    let catalog = global_catalog();

    let chinaq_extractor = catalog.get_by_source_id("chinaq");
    assert!(chinaq_extractor.is_some());
    assert_eq!(chinaq_extractor.unwrap().name(), "ChinaQ");

    let juqing_extractor = catalog.get_by_source_id("juqing");
    assert!(juqing_extractor.is_some());
    assert_eq!(juqing_extractor.unwrap().name(), "小鴨影視");

    let chinaq_by_url = catalog.get_by_url("https://chinaq.fun/tv-cn/123/");
    assert!(chinaq_by_url.is_some());

    let juqing_by_url = catalog.get_by_url("https://juqing.app/vod/detail/id/123.html");
    assert!(juqing_by_url.is_some());

    assert!(catalog.get_by_source_id("non_existent").is_none());
    assert!(catalog.get_by_url("https://unknown-domain.com/test").is_none());
}

#[tokio::test]
async fn test_catalog_search_validation() {
    let catalog = Catalog::new();
    let client = reqwest::Client::new();

    let err1 = catalog.search(&client, "").await.unwrap_err();
    assert!(err1.contains("Keyword cannot be empty"));

    let err2 = catalog.search(&client, "   ").await.unwrap_err();
    assert!(err2.contains("Keyword cannot be empty"));
}

#[tokio::test]
async fn test_catalog_search_fan_out_and_error_isolation() {
    let mut catalog = Catalog::new();
    catalog.register(Arc::new(DummyExtractor {
        id: "dummy",
        name: "Dummy Source",
        domain: "dummy.tv",
    }));
    catalog.register(Arc::new(FailingExtractor));

    let client = reqwest::Client::new();
    let res = catalog.search(&client, "test").await;
    assert!(res.is_ok(), "search should succeed even if one extractor fails");
    let results = res.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Dummy Title");
    assert_eq!(results[0].source_id, "dummy");
}

#[tokio::test]
async fn test_catalog_parse_series_metadata_validation_and_dispatch() {
    let mut catalog = Catalog::new();
    catalog.register(Arc::new(DummyExtractor {
        id: "dummy",
        name: "Dummy Source",
        domain: "dummy.tv",
    }));

    let client = reqwest::Client::new();

    let err_empty = catalog.parse_series_metadata(&client, "").await.unwrap_err();
    assert!(err_empty.contains("URL cannot be empty"));

    let err_whitespace = catalog.parse_series_metadata(&client, "   ").await.unwrap_err();
    assert!(err_whitespace.contains("URL cannot be empty"));

    let err_unsupported = catalog.parse_series_metadata(&client, "https://unknown.com/series").await.unwrap_err();
    assert!(err_unsupported.contains("Unsupported website URL"));

    let meta = catalog.parse_series_metadata(&client, "https://dummy.tv/series/100").await.unwrap();
    assert_eq!(meta.drama_id, "dummy123");
    assert_eq!(meta.source_id, "dummy");
}

#[tokio::test]
async fn test_catalog_fetch_episode_m3u8_validation_and_dispatch() {
    let mut catalog = Catalog::new();
    catalog.register(Arc::new(DummyExtractor {
        id: "dummy",
        name: "Dummy Source",
        domain: "dummy.tv",
    }));

    let client = reqwest::Client::new();

    let err_drama = catalog.fetch_episode_m3u8(&client, "", "ep1", "ch1", Some("dummy")).await.unwrap_err();
    assert!(err_drama.contains("Drama ID cannot be empty"));

    let err_ep = catalog.fetch_episode_m3u8(&client, "drama1", " ", "ch1", Some("dummy")).await.unwrap_err();
    assert!(err_ep.contains("Episode ID cannot be empty"));

    let err_ch = catalog.fetch_episode_m3u8(&client, "drama1", "ep1", "", Some("dummy")).await.unwrap_err();
    assert!(err_ch.contains("Channel ID cannot be empty"));

    let err_source = catalog.fetch_episode_m3u8(&client, "drama1", "ep1", "ch1", Some("unknown")).await.unwrap_err();
    assert!(err_source.contains("Extractor not found for source: unknown"));

    let stream = catalog.fetch_episode_m3u8(&client, "drama1", "ep1", "ch1", Some("dummy")).await.unwrap();
    assert_eq!(stream, "https://dummy.stream/index.m3u8");
}

#[test]
fn test_catalog_referer_synthesis() {
    let mut catalog = Catalog::new();
    catalog.register(Arc::new(DummyExtractor {
        id: "dummy",
        name: "Dummy Source",
        domain: "dummy.tv",
    }));

    let stream_ref = catalog.build_stream_referer(Some("dummy"), "drama1", "ep1", "ch1", Some("tv-tw")).unwrap();
    assert_eq!(stream_ref, "https://dummy.tv/tv-tw/drama1/ep1");

    let ep_ref = catalog.build_episode_referer(Some("dummy"), "tv-tw", "drama1", "ep1").unwrap();
    assert_eq!(ep_ref, "https://dummy.tv/tv-tw/drama1/ep1");

    let err = catalog.build_stream_referer(Some("missing"), "d", "e", "c", None).unwrap_err();
    assert!(err.contains("Extractor not found for source: missing"));
}

#[tokio::test]
async fn test_catalog_resolve_stream_with_referer() {
    let mut catalog = Catalog::new();
    catalog.register(Arc::new(DummyExtractor {
        id: "dummy",
        name: "Dummy Source",
        domain: "dummy.tv",
    }));

    let client = reqwest::Client::new();
    let (url, referer) = catalog
        .resolve_stream_with_referer(&client, "drama1", "ep1", "ch1", Some("dummy"))
        .await
        .unwrap();

    assert_eq!(url, "https://dummy.stream/index.m3u8");
    assert_eq!(referer, "https://dummy.tv/tv-cn/drama1/ep1");
}
