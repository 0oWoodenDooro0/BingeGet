use std::collections::BTreeMap;
use std::sync::LazyLock;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use reqwest::Url;

use crate::extractors::SiteExtractor;
use crate::models::{ChannelOption, EpisodeInfo, SearchResult, SeriesMetadata};

static JUQING_DETAIL_PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^/vod/detail/id/([a-zA-Z0-9_-]+)\.html$"#).expect("Invalid detail path regex")
});

static JUQING_PLAY_PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^/vod/play/id/([a-zA-Z0-9_-]+)-([a-zA-Z0-9_-]+)-([a-zA-Z0-9_-]+)\.html$"#)
        .expect("Invalid play path regex")
});

static PANEL_HEADER_SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<div[^>]*class=["'][^"']*\bplay-name\b[^"']*["'][^>]*>.*?</div>"#)
        .expect("Invalid panel header split regex")
});

static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<[^>]+>"#).expect("Invalid HTML tag regex")
});

static PLAY_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"/vod/play/id/([a-zA-Z0-9_-]+)-([a-zA-Z0-9_-]+)-([a-zA-Z0-9_-]+)\.html"#)
        .expect("Invalid play link regex")
});

static OG_IMAGE_RE1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?(?:property|name)=["']og:image["'][^>]*?content=["']([^"']+)["']"#)
        .expect("Invalid og:image regex 1")
});

static OG_IMAGE_RE2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?content=["']([^"']+)["'][^>]*?(?:property|name)=["']og:image["']"#)
        .expect("Invalid og:image regex 2")
});

static COVER_IMG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<img\s+[^>]*?(?:data-src|src)=["']([^"']+)["'][^>]*?class=["'][^"']*(?:lazyload|cover-img)[^"']*["']"#)
        .expect("Invalid cover img regex")
});

static OG_CATEGORY_RE1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?(?:property|name)=["']og:video:class["'][^>]*?content=["']([^"']+)["']"#)
        .expect("Invalid og:video:class regex 1")
});

static OG_CATEGORY_RE2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?content=["']([^"']+)["'][^>]*?(?:property|name)=["']og:video:class["']"#)
        .expect("Invalid og:video:class regex 2")
});

static GENRE_CATEGORY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?itemprop=["']genre["'][^>]*?content=["']([^"']+)["']"#)
        .expect("Invalid itemprop genre regex")
});

static H1_TITLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<h1[^>]*>(?:<a[^>]*>)?(.*?)(?:</a>)?</h1>"#)
        .expect("Invalid h1 title regex")
});

static OG_TITLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?(?:property|name)=["']og:title["'][^>]*?content=["']([^"']+)["']"#)
        .expect("Invalid og:title regex")
});

static MY_PLAYER_VIDEO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<video\b[^>]*?\bid=["']my-player["'][^>]*>(.*?)</video>"#)
        .expect("Invalid video tag regex")
});

static MY_PLAYER_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<video\b[^>]*?\bid=["']my-player["'][^>]*>"#)
        .expect("Invalid video tag start regex")
});

static SOURCE_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<source\b([^>]*)>"#).expect("Invalid source tag regex")
});

static SRC_ATTR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\bsrc=["']([^"']+)["']"#).expect("Invalid src attr regex")
});

static TYPE_ATTR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\btype=["']([^"']+)["']"#).expect("Invalid type attr regex")
});

static CHANNEL_ATTR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\b(?:data-channel|channel|data-ch|ch|data-from|from|title)=["']([^"']+)["']"#)
        .expect("Invalid channel attr regex")
});

#[derive(Default, Clone)]
pub struct JuqingExtractor {}

impl JuqingExtractor {
    pub fn new() -> Self {
        Self {}
    }

    fn parse_url_object(url: &str) -> Option<Url> {
        let raw = url.trim();
        if raw.is_empty() {
            return None;
        }
        if raw.starts_with("http://") || raw.starts_with("https://") {
            Url::parse(raw).ok()
        } else {
            Url::parse(&format!("https://{}", raw)).ok()
        }
    }

    pub fn extract_drama_id(&self, url: &str) -> Result<String> {
        let parsed = Self::parse_url_object(url)
            .ok_or_else(|| anyhow!("Invalid or empty URL: '{}'", url))?;

        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow!("Missing host in URL: '{}'", url))?
            .to_ascii_lowercase();

        if host != "juqing.app" && !host.ends_with(".juqing.app") {
            return Err(anyhow!("Not a juqing.app domain: '{}'", url));
        }

        let path = parsed.path();
        if let Some(caps) = JUQING_DETAIL_PATH_RE.captures(path) {
            if let Some(id) = caps.get(1) {
                return Ok(id.as_str().to_string());
            }
        }

        if let Some(caps) = JUQING_PLAY_PATH_RE.captures(path) {
            if let Some(id) = caps.get(1) {
                return Ok(id.as_str().to_string());
            }
        }

        Err(anyhow!("Cannot extract drama ID from Juqing URL: '{}'", url))
    }

    pub fn normalize_detail_url(&self, url: &str) -> Result<String> {
        let parsed = Self::parse_url_object(url)
            .ok_or_else(|| anyhow!("Invalid or empty URL: '{}'", url))?;

        let drama_id = self.extract_drama_id(url)?;
        let scheme = parsed.scheme();
        let host = parsed.host_str().unwrap_or("juqing.app");

        Ok(format!("{}://{}/vod/detail/id/{}.html", scheme, host, drama_id))
    }

    pub fn build_episode_referer(&self, channel: &str, drama_id: &str, ep_id: &str) -> String {
        let clean_drama = drama_id.trim();
        let clean_channel = channel.trim();
        let clean_ep = ep_id.trim().trim_start_matches("ep");

        if clean_channel.is_empty() || clean_ep.is_empty() {
            if clean_drama.is_empty() {
                "https://tw.juqing.app/".to_string()
            } else {
                format!("https://tw.juqing.app/vod/detail/id/{}.html", clean_drama)
            }
        } else {
            format!(
                "https://tw.juqing.app/vod/play/id/{}-{}-{}.html",
                clean_drama, clean_channel, clean_ep
            )
        }
    }

    pub fn parse_cover_url(html: &str) -> Option<String> {
        let raw_cover = if let Some(caps) = OG_IMAGE_RE1.captures(html) {
            caps.get(1).map(|m| m.as_str())
        } else if let Some(caps) = OG_IMAGE_RE2.captures(html) {
            caps.get(1).map(|m| m.as_str())
        } else if let Some(caps) = COVER_IMG_RE.captures(html) {
            caps.get(1).map(|m| m.as_str())
        } else {
            None
        };

        raw_cover.map(|c| {
            let trimmed = c.trim();
            if trimmed.starts_with("//") {
                format!("https:{}", trimmed)
            } else if trimmed.starts_with('/') {
                format!("https://juqing.app{}", trimmed)
            } else {
                trimmed.to_string()
            }
        })
    }

    pub fn parse_title(html: &str) -> Option<String> {
        if let Some(caps) = H1_TITLE_RE.captures(html) {
            if let Some(t) = caps.get(1) {
                let text = HTML_TAG_RE.replace_all(t.as_str(), "").trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        if let Some(caps) = OG_TITLE_RE.captures(html) {
            if let Some(t) = caps.get(1) {
                let mut title = t.as_str().trim();
                if let Some((first, _)) = title.split_once("線上看") {
                    title = first.trim();
                }
                title = title.trim_end_matches("- 小鴨影視").trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }

        None
    }

    pub fn parse_category(html: &str) -> Option<String> {
        if let Some(caps) = OG_CATEGORY_RE1.captures(html) {
            if let Some(c) = caps.get(1) {
                let trimmed = c.as_str().trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        if let Some(caps) = OG_CATEGORY_RE2.captures(html) {
            if let Some(c) = caps.get(1) {
                let trimmed = c.as_str().trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        if let Some(caps) = GENRE_CATEGORY_RE.captures(html) {
            if let Some(c) = caps.get(1) {
                let trimmed = c.as_str().trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    pub fn parse_playlist_panels(
        html: &str,
        drama_id: &str,
    ) -> (Vec<ChannelOption>, Vec<EpisodeInfo>) {
        let headers: Vec<regex::Match> = PANEL_HEADER_SPLIT_RE.find_iter(html).collect();
        let mut channels: Vec<ChannelOption> = Vec::new();
        let mut episodes_map: BTreeMap<u32, Vec<String>> = BTreeMap::new();

        if !headers.is_empty() {
            for i in 0..headers.len() {
                let header_text = headers[i].as_str();
                let channel_name = HTML_TAG_RE.replace_all(header_text, "").trim().to_string();

                let body_start = headers[i].end();
                let body_end = if i + 1 < headers.len() {
                    headers[i + 1].start()
                } else {
                    html.len()
                };
                let body = &html[body_start..body_end];

                let mut current_channel_id: Option<String> = None;

                for cap in PLAY_LINK_RE.captures_iter(body) {
                    let d_id = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
                    let ch_id = cap.get(2).map(|m| m.as_str()).unwrap_or_default();
                    let ep_str = cap.get(3).map(|m| m.as_str()).unwrap_or_default();

                    if !drama_id.is_empty() && d_id != drama_id {
                        continue;
                    }

                    if current_channel_id.is_none() {
                        current_channel_id = Some(ch_id.to_string());
                        if !channels.iter().any(|c| c.id == ch_id) {
                            let name = if channel_name.is_empty() {
                                format!("Channel {}", ch_id)
                            } else {
                                channel_name.clone()
                            };
                            channels.push(ChannelOption {
                                id: ch_id.to_string(),
                                name,
                            });
                        }
                    }

                    if let Ok(ep_num) = ep_str.parse::<u32>() {
                        let chs = episodes_map.entry(ep_num).or_default();
                        if !chs.iter().any(|c| c == ch_id) {
                            chs.push(ch_id.to_string());
                        }
                    }
                }
            }
        } else {
            // Fallback if no play-name headers found
            for cap in PLAY_LINK_RE.captures_iter(html) {
                let d_id = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
                let ch_id = cap.get(2).map(|m| m.as_str()).unwrap_or_default();
                let ep_str = cap.get(3).map(|m| m.as_str()).unwrap_or_default();

                if !drama_id.is_empty() && d_id != drama_id {
                    continue;
                }

                if !channels.iter().any(|c| c.id == ch_id) {
                    channels.push(ChannelOption {
                        id: ch_id.to_string(),
                        name: format!("Channel {}", ch_id),
                    });
                }

                if let Ok(ep_num) = ep_str.parse::<u32>() {
                    let chs = episodes_map.entry(ep_num).or_default();
                    if !chs.iter().any(|c| c == ch_id) {
                        chs.push(ch_id.to_string());
                    }
                }
            }
        }

        let episodes = episodes_map
            .into_iter()
            .map(|(ep_number, available_channels)| EpisodeInfo {
                ep_number,
                ep_id: ep_number.to_string(),
                available_channels,
            })
            .collect();

        (channels, episodes)
    }

    pub fn parse_metadata_from_html(&self, html: &str, drama_id: &str) -> Result<SeriesMetadata> {
        let (channels, episodes) = Self::parse_playlist_panels(html, drama_id);
        if episodes.is_empty() {
            return Err(anyhow!(
                "No episodes found on Juqing page for drama ID: {}",
                drama_id
            ));
        }

        let cover_url = Self::parse_cover_url(html);
        let category_slug = Self::parse_category(html);

        Ok(SeriesMetadata {
            drama_id: drama_id.to_string(),
            source_id: "juqing".to_string(),
            category_slug,
            cover_url,
            episodes,
            channels,
        })
    }

    pub fn parse_m3u8_from_html(html: &str, requested_channel: Option<&str>) -> Result<String> {
        let video_content = if let Some(caps) = MY_PLAYER_VIDEO_RE.captures(html) {
            caps.get(1).map(|m| m.as_str()).unwrap_or("")
        } else if let Some(caps) = MY_PLAYER_TAG_RE.captures(html) {
            let start = caps.get(0).unwrap().end();
            let rest = &html[start..];
            if let Some(end) = rest.find("</video>") {
                &rest[..end]
            } else {
                rest
            }
        } else if html.contains("my-player") {
            html
        } else {
            return Err(anyhow!("No <video id=\"my-player\"> tag found in Juqing page"));
        };

        struct SourceEntry {
            src: String,
            mime_type: Option<String>,
            channel: Option<String>,
            raw_tag: String,
        }

        let mut sources = Vec::new();

        for cap in SOURCE_TAG_RE.captures_iter(video_content) {
            let attrs = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let src = SRC_ATTR_RE
                .captures(attrs)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string());
            let mime_type = TYPE_ATTR_RE
                .captures(attrs)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_lowercase());
            let channel = CHANNEL_ATTR_RE
                .captures(attrs)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string());

            if let Some(src_val) = src {
                if !src_val.is_empty() {
                    sources.push(SourceEntry {
                        src: src_val,
                        mime_type,
                        channel,
                        raw_tag: attrs.to_string(),
                    });
                }
            }
        }

        // If no <source> tags, check if <video id="my-player"> has a direct src="..."
        if sources.is_empty() {
            if let Some(cap) = MY_PLAYER_TAG_RE.captures(html) {
                let tag_str = cap.get(0).unwrap().as_str();
                if let Some(src_cap) = SRC_ATTR_RE.captures(tag_str) {
                    if let Some(direct_src) = src_cap.get(1) {
                        let trimmed = direct_src.as_str().trim();
                        if !trimmed.is_empty() {
                            return Ok(Self::normalize_stream_url(trimmed));
                        }
                    }
                }
            }
            return Err(anyhow!("No <source> tags found inside <video id=\"my-player\">"));
        }

        // Prefer mpegurl sources
        let mpegurl_sources: Vec<&SourceEntry> = sources
            .iter()
            .filter(|s| {
                if let Some(ref m) = s.mime_type {
                    if m.contains("mpegurl") {
                        return true;
                    }
                }
                s.src.contains(".m3u8")
            })
            .collect();

        let pool = if !mpegurl_sources.is_empty() {
            mpegurl_sources
        } else {
            sources.iter().collect()
        };

        // Try matching requested channel
        if let Some(req_ch) = requested_channel {
            let req_ch = req_ch.trim();
            if !req_ch.is_empty() {
                // 1. Direct channel attribute match
                if let Some(found) = pool.iter().find(|s| {
                    if let Some(ref c) = s.channel {
                        c == req_ch || c.contains(req_ch)
                    } else {
                        false
                    }
                }) {
                    return Ok(Self::normalize_stream_url(&found.src));
                }

                // 2. Raw tag or src contains channel string
                if let Some(found) = pool
                    .iter()
                    .find(|s| s.raw_tag.contains(req_ch) || s.src.contains(req_ch))
                {
                    return Ok(Self::normalize_stream_url(&found.src));
                }
            }
        }

        // Default to first source
        let chosen = pool.first().ok_or_else(|| anyhow!("No valid stream source found"))?;
        Ok(Self::normalize_stream_url(&chosen.src))
    }

    fn normalize_stream_url(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.starts_with("//") {
            format!("https:{}", trimmed)
        } else if trimmed.starts_with('/') {
            format!("https://tw.juqing.app{}", trimmed)
        } else {
            trimmed.to_string()
        }
    }
}

#[async_trait]
impl SiteExtractor for JuqingExtractor {
    fn id(&self) -> &'static str {
        "juqing"
    }

    fn name(&self) -> &'static str {
        "小鴨影視"
    }

    fn can_handle(&self, url: &str) -> bool {
        let Some(parsed) = Self::parse_url_object(url) else {
            return false;
        };

        let Some(host) = parsed.host_str() else {
            return false;
        };

        let host = host.to_ascii_lowercase();
        if host != "juqing.app" && !host.ends_with(".juqing.app") {
            return false;
        }

        let path = parsed.path();
        JUQING_DETAIL_PATH_RE.is_match(path) || JUQING_PLAY_PATH_RE.is_match(path)
    }

    fn build_episode_referer(&self, category_slug: &str, drama_id: &str, ep_id: &str) -> String {
        self.build_episode_referer(category_slug, drama_id, ep_id)
    }

    async fn fetch_series_metadata(&self, client: &reqwest::Client, url: &str) -> Result<SeriesMetadata> {
        let detail_url = self.normalize_detail_url(url)?;
        let drama_id = self.extract_drama_id(&detail_url)?;

        let html = client
            .get(&detail_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .send()
            .await?
            .text()
            .await?;

        self.parse_metadata_from_html(&html, &drama_id)
    }

    async fn fetch_m3u8_for_episode(
        &self,
        client: &reqwest::Client,
        drama_id: &str,
        ep_id: &str,
        channel: &str,
    ) -> Result<String> {
        let clean_ep = ep_id.trim().trim_start_matches("ep");
        let play_url = if drama_id.starts_with("http://") || drama_id.starts_with("https://") {
            drama_id.to_string()
        } else {
            format!(
                "https://tw.juqing.app/vod/play/id/{}-{}-{}.html",
                drama_id.trim(),
                channel.trim(),
                clean_ep
            )
        };

        let referer = self.build_episode_referer(channel, drama_id, ep_id);

        let html = client
            .get(&play_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .header("Referer", &referer)
            .send()
            .await?
            .text()
            .await?;

        Self::parse_m3u8_from_html(&html, Some(channel))
    }

    async fn search(&self, _client: &reqwest::Client, _keyword: &str) -> Result<Vec<SearchResult>> {
        // Ticket #41 will implement full search
        Ok(Vec::new())
    }
}
