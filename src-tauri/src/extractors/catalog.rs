use std::sync::{Arc, LazyLock};

use crate::extractors::chinaq::ChinaQExtractor;
use crate::extractors::juqing::JuqingExtractor;
use crate::extractors::SiteExtractor;
use crate::models::{SearchResult, SeriesMetadata};

#[derive(Clone)]
pub struct Catalog {
    extractors: Vec<Arc<dyn SiteExtractor>>,
}

impl Default for Catalog {
    fn default() -> Self {
        Self::default_catalog()
    }
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    pub fn default_catalog() -> Self {
        let mut catalog = Self::new();
        catalog.register(Arc::new(ChinaQExtractor::new()));
        catalog.register(Arc::new(JuqingExtractor::new()));
        catalog
    }

    pub fn default_registry() -> Self {
        Self::default_catalog()
    }

    pub fn register(&mut self, extractor: Arc<dyn SiteExtractor>) {
        self.extractors.push(extractor);
    }

    pub fn get_by_url(&self, url: &str) -> Option<Arc<dyn SiteExtractor>> {
        self.extractors.iter().find(|e| e.can_handle(url)).cloned()
    }

    pub fn get_by_source_id(&self, source_id: &str) -> Option<Arc<dyn SiteExtractor>> {
        self.extractors.iter().find(|e| e.id() == source_id).cloned()
    }

    pub fn all(&self) -> Vec<Arc<dyn SiteExtractor>> {
        self.extractors.clone()
    }

    pub async fn search(
        &self,
        client: &reqwest::Client,
        keyword: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let kw = keyword.trim();
        if kw.is_empty() {
            return Err("Keyword cannot be empty".to_string());
        }

        let extractors = self.all();
        let mut handles = Vec::new();

        for extractor in extractors {
            let client_clone = client.clone();
            let kw_string = kw.to_string();
            handles.push(tokio::spawn(async move {
                let name = extractor.name();
                let res = extractor.search(&client_clone, &kw_string).await;
                (name, res)
            }));
        }

        let mut all_results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((_name, Ok(results))) => {
                    all_results.extend(results);
                }
                Ok((name, Err(e))) => {
                    eprintln!("Search error on {}: {}", name, e);
                }
                Err(e) => {
                    eprintln!("Search task join error: {}", e);
                }
            }
        }

        Ok(all_results)
    }

    pub async fn parse_series_metadata(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<SeriesMetadata, String> {
        let url = url.trim();
        if url.is_empty() {
            return Err("URL cannot be empty".to_string());
        }

        if let Some(extractor) = self.get_by_url(url) {
            return extractor
                .fetch_series_metadata(client, url)
                .await
                .map_err(|e| e.to_string());
        }

        Err(format!("Unsupported website URL: {}", url))
    }

    pub fn build_stream_referer(
        &self,
        source_id: Option<&str>,
        drama_id: &str,
        ep_id: &str,
        channel: &str,
        category_slug: Option<&str>,
    ) -> Result<String, String> {
        let sid = source_id.unwrap_or("chinaq");
        let extractor = self
            .get_by_source_id(sid)
            .ok_or_else(|| format!("Extractor not found for source: {}", sid))?;
        Ok(extractor.build_stream_referer(drama_id, ep_id, channel, category_slug))
    }

    pub fn build_episode_referer(
        &self,
        source_id: Option<&str>,
        category_slug: &str,
        drama_id: &str,
        ep_id: &str,
    ) -> Result<String, String> {
        let sid = source_id.unwrap_or("chinaq");
        let extractor = self
            .get_by_source_id(sid)
            .ok_or_else(|| format!("Extractor not found for source: {}", sid))?;
        Ok(extractor.build_episode_referer(category_slug, drama_id, ep_id))
    }

    pub async fn fetch_episode_m3u8(
        &self,
        client: &reqwest::Client,
        drama_id: &str,
        ep_id: &str,
        channel: &str,
        source_id: Option<&str>,
    ) -> Result<String, String> {
        if drama_id.trim().is_empty() {
            return Err("Drama ID cannot be empty".to_string());
        }
        if ep_id.trim().is_empty() {
            return Err("Episode ID cannot be empty".to_string());
        }
        if channel.trim().is_empty() {
            return Err("Channel ID cannot be empty".to_string());
        }

        let sid = source_id.unwrap_or("chinaq");
        let extractor = self
            .get_by_source_id(sid)
            .ok_or_else(|| format!("Extractor not found for source: {}", sid))?;

        extractor
            .fetch_m3u8_for_episode(client, drama_id, ep_id, channel)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn resolve_stream_with_referer(
        &self,
        client: &reqwest::Client,
        drama_id: &str,
        ep_id: &str,
        channel: &str,
        source_id: Option<&str>,
    ) -> Result<(String, String), String> {
        let m3u8_url = self
            .fetch_episode_m3u8(client, drama_id, ep_id, channel, source_id)
            .await?;
        let referer = self.build_stream_referer(source_id, drama_id, ep_id, channel, None)?;
        Ok((m3u8_url, referer))
    }
}

static GLOBAL_CATALOG: LazyLock<Catalog> = LazyLock::new(Catalog::default_catalog);

pub fn global_catalog() -> &'static Catalog {
    &GLOBAL_CATALOG
}
