pub mod chinaq;

use async_trait::async_trait;
use crate::models::{SearchResult, SeriesMetadata};

#[async_trait]
pub trait SiteExtractor: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn can_handle(&self, url: &str) -> bool;

    async fn fetch_series_metadata(&self, client: &reqwest::Client, url: &str) -> anyhow::Result<SeriesMetadata>;
    async fn fetch_m3u8_for_episode(&self, client: &reqwest::Client, drama_id: &str, ep_id: &str, channel: &str) -> anyhow::Result<String>;
    async fn search(&self, client: &reqwest::Client, keyword: &str) -> anyhow::Result<Vec<SearchResult>>;
}
