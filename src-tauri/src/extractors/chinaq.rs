use std::collections::HashSet;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::Deserialize;

use crate::extractors::SiteExtractor;
use crate::models::{ChannelOption, EpisodeInfo, SeriesMetadata};

#[derive(Default, Clone)]
pub struct ChinaQExtractor {}

impl ChinaQExtractor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn extract_drama_id(&self, url: &str) -> Result<String> {
        let re = Regex::new(r#"/tv-cn/([a-zA-Z0-9]+)(?:/.*)?"#)?;
        if let Some(cap) = re.captures(url) {
            if let Some(id) = cap.get(1) {
                return Ok(id.as_str().to_string());
            }
        }
        Err(anyhow!("Cannot extract drama ID from URL: {}", url))
    }

    pub fn parse_episodes_from_html(&self, html: &str) -> Result<Vec<EpisodeInfo>> {
        let re = Regex::new(r#"/tv-cn/[^/]+/ep(\d+)\.html"#)?;
        let mut seen = HashSet::new();
        let mut episodes = Vec::new();

        for cap in re.captures_iter(html) {
            if let Some(num_str) = cap.get(1) {
                let num_str = num_str.as_str();
                if let Ok(num) = num_str.parse::<u32>() {
                    if seen.insert(num) {
                        episodes.push(EpisodeInfo {
                            ep_number: num,
                            ep_id: format!("ep{}", num),
                        });
                    }
                }
            }
        }

        episodes.sort_by_key(|e| e.ep_number);
        Ok(episodes)
    }

    pub fn parse_available_channels(&self, qplays_json: &str) -> Result<Vec<ChannelOption>> {
        #[derive(Deserialize)]
        struct VideoPlay {
            src_site: String,
        }

        #[derive(Deserialize)]
        struct QplaysResponse {
            video_plays: Vec<VideoPlay>,
        }

        let resp: QplaysResponse = serde_json::from_str(qplays_json)
            .map_err(|e| anyhow!("Failed to parse qplays JSON: {}", e))?;

        let mut seen = HashSet::new();
        let mut channels = Vec::new();

        for play in resp.video_plays {
            if seen.insert(play.src_site.clone()) {
                let name = match play.src_site.as_str() {
                    "ikzy" => "iKun 資源 (ikzy)".to_string(),
                    "hnzy" => "紅牛 資源 (hnzy)".to_string(),
                    "jszy" => "極速 資源 (jszy)".to_string(),
                    "jyzy" => "金鷹 資源 (jyzy)".to_string(),
                    "mdzy" => "魔都 資源 (mdzy)".to_string(),
                    "lzzy" => "量子公司 (lzzy)".to_string(),
                    other => format!("片源 ({})", other),
                };

                channels.push(ChannelOption {
                    id: play.src_site,
                    name,
                });
            }
        }

        Ok(channels)
    }

    pub fn extract_m3u8_for_channel(&self, qplays_json: &str, channel_id: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct VideoPlay {
            play_data: String,
            src_site: String,
        }

        #[derive(Deserialize)]
        struct QplaysResponse {
            video_plays: Vec<VideoPlay>,
        }

        let resp: QplaysResponse = serde_json::from_str(qplays_json)
            .map_err(|e| anyhow!("Failed to parse qplays JSON: {}", e))?;

        for play in resp.video_plays {
            if play.src_site == channel_id {
                return Ok(play.play_data);
            }
        }

        Err(anyhow!(
            "Channel '{}' not found in available video sources",
            channel_id
        ))
    }

    pub async fn fetch_series_metadata(&self, client: &reqwest::Client, url: &str) -> Result<SeriesMetadata> {
        let drama_id = self.extract_drama_id(url)?;
        let drama_url = format!("https://chinaq.fun/tv-cn/{}/", drama_id);

        let html = client
            .get(&drama_url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .send()
            .await?
            .text()
            .await?;

        let episodes = self.parse_episodes_from_html(&html)?;
        if episodes.is_empty() {
            return Err(anyhow!("No episodes found on page: {}", drama_url));
        }

        // Query ep1 to get initial channel list
        let first_ep_id = &episodes[0].ep_id;
        let qplays_url = format!("https://chinaq.fun/qplays/{}/{}", drama_id, first_ep_id);
        let qplays_json = client
            .get(&qplays_url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .header("Referer", &drama_url)
            .send()
            .await?
            .text()
            .await?;

        let channels = self.parse_available_channels(&qplays_json)?;

        Ok(SeriesMetadata {
            drama_id,
            episodes,
            channels,
        })
    }

    pub async fn fetch_m3u8_for_episode(
        &self,
        client: &reqwest::Client,
        drama_id: &str,
        ep_id: &str,
        channel: &str,
    ) -> Result<String> {
        let qplays_url = format!("https://chinaq.fun/qplays/{}/{}", drama_id, ep_id);
        let referer = format!("https://chinaq.fun/tv-cn/{}/{}.html", drama_id, ep_id);
        let qplays_json = client
            .get(&qplays_url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .header("Referer", &referer)
            .send()
            .await?
            .text()
            .await?;

        self.extract_m3u8_for_channel(&qplays_json, channel)
    }
}

impl SiteExtractor for ChinaQExtractor {
    fn id(&self) -> &'static str {
        "chinaq"
    }

    fn name(&self) -> &'static str {
        "ChinaQ"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("chinaq.fun")
    }
}
