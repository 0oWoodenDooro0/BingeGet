use std::sync::Arc;
use async_trait::async_trait;
use bingeget::commands::{
    detect_system_environment_core, fetch_episode_m3u8_core, parse_series_metadata_core,
    search_series_core,
};
use bingeget::extractors::registry::{global_registry, ExtractorRegistry};
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

#[test]
fn test_extractor_registry_lookup_by_url_and_source_id() {
    let registry = global_registry();

    let extractor_by_url = registry.get_by_url("https://chinaq.fun/tv-cn/202614235/");
    assert!(extractor_by_url.is_some(), "Default registry must resolve chinaq url");
    assert_eq!(extractor_by_url.unwrap().id(), "chinaq");

    let extractor_by_id = registry.get_by_source_id("chinaq");
    assert!(extractor_by_id.is_some(), "Default registry must resolve source_id chinaq");
    assert_eq!(extractor_by_id.unwrap().name(), "ChinaQ");

    let juqing_by_url = registry.get_by_url("https://juqing.app/vod/detail/id/gb27z2th.html");
    assert!(juqing_by_url.is_some(), "Default registry must resolve juqing url");
    assert_eq!(juqing_by_url.unwrap().id(), "juqing");

    let juqing_by_id = registry.get_by_source_id("juqing");
    assert!(juqing_by_id.is_some(), "Default registry must resolve source_id juqing");
    assert_eq!(juqing_by_id.unwrap().name(), "小鴨影視");

    assert!(registry.get_by_source_id("unknown_site").is_none());
    assert!(registry.get_by_url("https://unknown.com/video/1").is_none());
}

#[test]
fn test_custom_extractor_registry_registration() {
    let mut registry = ExtractorRegistry::new();
    assert_eq!(registry.all().len(), 0);

    let dummy = Arc::new(DummyExtractor {
        id: "dummy",
        name: "Dummy Source",
        domain: "dummy.tv",
    });

    registry.register(dummy.clone());
    assert_eq!(registry.all().len(), 1);

    let found_by_url = registry.get_by_url("https://dummy.tv/watch/42").expect("Should find dummy by URL");
    assert_eq!(found_by_url.id(), "dummy");
    assert_eq!(found_by_url.name(), "Dummy Source");

    let found_by_id = registry.get_by_source_id("dummy").expect("Should find dummy by ID");
    assert_eq!(found_by_id.id(), "dummy");

    assert!(registry.get_by_source_id("chinaq").is_none());
    assert!(registry.get_by_source_id("juqing").is_none());
}

#[tokio::test]
async fn test_parse_series_metadata_empty_or_invalid_url() {
    let client = reqwest::Client::new();
    let err = parse_series_metadata_core(&client, "").await.unwrap_err();
    assert!(err.contains("URL cannot be empty"));

    let err2 = parse_series_metadata_core(&client, "https://unknown-domain.com/123").await.unwrap_err();
    assert!(err2.contains("Unsupported website URL"));
}

#[tokio::test]
async fn test_parse_series_metadata_core_resolves_juqing_url() {
    let client = reqwest::Client::new();
    let res = parse_series_metadata_core(&client, "https://juqing.app/vod/detail/id/gb27z2th.html").await;
    match res {
        Ok(metadata) => {
            assert_eq!(metadata.source_id, "juqing");
            assert_eq!(metadata.drama_id, "gb27z2th");
            assert!(!metadata.channels.is_empty());
            assert!(!metadata.episodes.is_empty());
        }
        Err(err) => {
            assert!(!err.contains("Unsupported website URL"));
        }
    }
}

#[tokio::test]
async fn test_parse_series_metadata_core_resolves_juqing_play_url() {
    let client = reqwest::Client::new();
    let res = parse_series_metadata_core(&client, "https://juqing.app/vod/play/id/gb27z2th-1-1.html").await;
    match res {
        Ok(metadata) => {
            assert_eq!(metadata.source_id, "juqing");
            assert_eq!(metadata.drama_id, "gb27z2th");
        }
        Err(err) => {
            assert!(!err.contains("Unsupported website URL"));
        }
    }
}

#[tokio::test]
async fn test_fetch_episode_m3u8_validation() {
    let client = reqwest::Client::new();
    let err = fetch_episode_m3u8_core(&client, "", "ep1", "ikzy", None).await.unwrap_err();
    assert!(err.contains("Drama ID cannot be empty"));

    let err2 = fetch_episode_m3u8_core(&client, "202614235", "", "ikzy", None).await.unwrap_err();
    assert!(err2.contains("Episode ID cannot be empty"));

    let err3 = fetch_episode_m3u8_core(&client, "202614235", "ep1", "", None).await.unwrap_err();
    assert!(err3.contains("Channel ID cannot be empty"));
}

#[tokio::test]
async fn test_fetch_episode_m3u8_source_id_dispatch() {
    let client = reqwest::Client::new();
    let err = fetch_episode_m3u8_core(&client, "202614235", "ep1", "ikzy", Some("unknown_source"))
        .await
        .unwrap_err();
    assert!(err.contains("Extractor not found for source: unknown_source"));
}

#[tokio::test]
async fn test_search_series_validation() {
    let client = reqwest::Client::new();
    let err = search_series_core(&client, "").await.unwrap_err();
    assert!(err.contains("Keyword cannot be empty"));

    let err2 = search_series_core(&client, "   ").await.unwrap_err();
    assert!(err2.contains("Keyword cannot be empty"));
}

#[test]
fn test_detect_system_environment() {
    let env = detect_system_environment_core();
    // System should find /usr/bin/n-m3u8dl-re and /usr/bin/ffmpeg on this Linux machine
    assert!(env.nm3u8dl_path.is_some());
    assert!(env.ffmpeg_path.is_some());
    assert!(env.is_ready);
    #[cfg(target_os = "linux")]
    assert!(env.gstreamer_ready.is_some());
}
