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
    Regex::new(r#"(?i)\b(?:data-channel|channel|data-ch|ch|data-from|from|title|label)=["']([^"']+)["']"#)
        .expect("Invalid channel attr regex")
});

static SEARCH_ARTICLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<article\b[^>]*>(.*?)</article>"#).expect("Invalid search article regex")
});

static SEARCH_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)href=["']([^"']*/vod/detail/id/[a-zA-Z0-9_-]+\.html)["']"#)
        .expect("Invalid search link regex")
});

static H4_TITLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<h4[^>]*class=["'][^"']*\btitle\b[^"']*["'][^>]*>(.*?)</h4>"#)
        .expect("Invalid h4 title regex")
});

static META_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?(?:itemprop=["']name["']|name=["']name["'])[^>]*?content=["']([^"']+)["']"#)
        .expect("Invalid meta name regex")
});

static COVER_TITLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<a\s+[^>]*?class=["'][^"']*\bcover-img\b[^"']*["'][^>]*?title=["']([^"']+)["']"#)
        .expect("Invalid cover title regex")
});

static META_YEAR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?(?:itemprop=["']dateCreated["']|itemprop=["']copyrightYear["'])[^>]*?content=["'](\d{4})["']"#)
        .expect("Invalid meta year regex")
});

static META_DATE_PUB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?itemprop=["']datePublished["'][^>]*?content=["'](\d{4})"#)
        .expect("Invalid meta date published regex")
});

static ARTICLE_IMG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<img\b([^>]*)/?>"#).expect("Invalid article img regex")
});

static DATA_SRC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\bdata-src=["']([^"']+)["']"#).expect("Invalid data-src regex")
});

static SRC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\bsrc=["']([^"']+)["']"#).expect("Invalid src regex")
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

    pub fn build_stream_referer(
        &self,
        drama_id: &str,
        ep_id: &str,
        channel: &str,
        _category_slug: Option<&str>,
    ) -> String {
        self.build_episode_referer(channel, drama_id, ep_id)
    }

    pub fn category_label_from_slug(slug: &str) -> Option<&'static str> {
        let clean = slug.trim().trim_start_matches("tv-");
        match clean {
            "cn" => Some("陸劇"),
            "kr" => Some("韓劇"),
            "tw" => Some("台劇"),
            "jp" => Some("日劇"),
            "hk" => Some("港劇"),
            "th" => Some("泰劇"),
            "anime" | "acg" => Some("動漫"),
            "movie" => Some("電影"),
            _ => None,
        }
    }

    pub fn slug_from_category(label: &str) -> Option<&'static str> {
        match label.trim() {
            "陸劇" | "國產劇" | "大陸劇" => Some("tv-cn"),
            "韓劇" => Some("tv-kr"),
            "台劇" | "臺灣劇" => Some("tv-tw"),
            "日劇" => Some("tv-jp"),
            "港劇" | "香港劇" => Some("tv-hk"),
            "泰劇" => Some("tv-th"),
            "動漫" => Some("anime"),
            "電影" => Some("movie"),
            _ => None,
        }
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

    pub fn parse_title(html: &str) -> Option<String> {
        if let Some(caps) = H1_TITLE_RE.captures(html) {
            if let Some(m) = caps.get(1) {
                let clean = HTML_TAG_RE.replace_all(m.as_str(), "").trim().to_string();
                if !clean.is_empty() {
                    return Some(clean);
                }
            }
        }

        if let Some(caps) = OG_TITLE_RE.captures(html) {
            if let Some(m) = caps.get(1) {
                let clean = m.as_str().trim();
                let title = clean.split_once("線上看").map(|(t, _)| t.trim()).unwrap_or(clean);
                let title = title.trim_end_matches("- 小鴨影視").trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }

        None
    }

    pub fn parse_cover_url(html: &str) -> Option<String> {
        if let Some(caps) = OG_IMAGE_RE1.captures(html) {
            if let Some(m) = caps.get(1) {
                let url = m.as_str().trim();
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }

        if let Some(caps) = OG_IMAGE_RE2.captures(html) {
            if let Some(m) = caps.get(1) {
                let url = m.as_str().trim();
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }

        if let Some(caps) = COVER_IMG_RE.captures(html) {
            if let Some(m) = caps.get(1) {
                let url = m.as_str().trim();
                if !url.is_empty() && !url.contains("loading.") {
                    return Some(url.to_string());
                }
            }
        }

        None
    }

    pub fn parse_category(html: &str) -> Option<String> {
        if let Some(caps) = OG_CATEGORY_RE1.captures(html) {
            if let Some(m) = caps.get(1) {
                let cat = m.as_str().trim();
                if !cat.is_empty() {
                    return Some(cat.to_string());
                }
            }
        }

        if let Some(caps) = OG_CATEGORY_RE2.captures(html) {
            if let Some(m) = caps.get(1) {
                let cat = m.as_str().trim();
                if !cat.is_empty() {
                    return Some(cat.to_string());
                }
            }
        }

        if let Some(caps) = GENRE_CATEGORY_RE.captures(html) {
            if let Some(m) = caps.get(1) {
                let cat = m.as_str().trim();
                if !cat.is_empty() {
                    return Some(cat.to_string());
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
            #[allow(dead_code)]
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
                let target_label = format!("線路{}", req_ch);
                let req_digits: String = req_ch.chars().filter(|c| c.is_ascii_digit()).collect();

                if let Some(found) = pool.iter().find(|s| {
                    if let Some(ref c) = s.channel {
                        let c_trimmed = c.trim();
                        // 1. Direct match (e.g. "8" == "8" or "線路8" == "線路8")
                        if c_trimmed == req_ch {
                            return true;
                        }
                        // 2. Match with "線路" prefix
                        if c_trimmed == target_label || c_trimmed.strip_prefix("線路") == Some(req_ch) {
                            return true;
                        }
                        // 3. Match numeric ID (e.g. "線路8" -> "8" == "8")
                        if !req_digits.is_empty() {
                            let c_digits: String = c_trimmed.chars().filter(|ch| ch.is_ascii_digit()).collect();
                            if !c_digits.is_empty() && c_digits == req_digits {
                                return true;
                            }
                        }
                        false
                    } else {
                        false
                    }
                }) {
                    return Ok(Self::normalize_stream_url(&found.src));
                }

                return Err(anyhow!(
                    "Requested channel '{}' not found in stream sources",
                    req_ch
                ));
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

    pub fn parse_article_title(block: &str) -> Option<String> {
        let raw_title = if let Some(caps) = H4_TITLE_RE.captures(block) {
            caps.get(1).map(|m| {
                HTML_TAG_RE.replace_all(m.as_str(), "").trim().to_string()
            })
        } else if let Some(caps) = META_NAME_RE.captures(block) {
            caps.get(1).map(|m| m.as_str().trim().to_string())
        } else if let Some(caps) = COVER_TITLE_RE.captures(block) {
            caps.get(1).map(|m| m.as_str().trim().to_string())
        } else {
            None
        };

        raw_title.and_then(|t| {
            let mut clean = t.as_str().trim();
            if let Some((first, _)) = clean.split_once("線上看") {
                clean = first.trim();
            }
            let title = clean.trim_end_matches("- 小鴨影視").trim();
            if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            }
        })
    }

    pub fn parse_article_year(block: &str) -> Option<String> {
        if let Some(caps) = META_YEAR_RE.captures(block) {
            caps.get(1).map(|m| m.as_str().trim().to_string())
        } else if let Some(caps) = META_DATE_PUB_RE.captures(block) {
            caps.get(1).map(|m| m.as_str().trim().to_string())
        } else {
            None
        }
    }

    pub fn parse_article_category(block: &str) -> Option<String> {
        Self::parse_category(block)
    }

    pub fn parse_article_cover_url(block: &str) -> Option<String> {
        for img_cap in ARTICLE_IMG_RE.captures_iter(block) {
            let attrs = img_cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let mut raw_url: Option<&str> = None;

            if let Some(data_src) = DATA_SRC_RE.captures(attrs).and_then(|c| c.get(1)) {
                let s = data_src.as_str().trim();
                if !s.is_empty() && !s.contains("loading.") {
                    raw_url = Some(s);
                }
            }

            if raw_url.is_none() {
                if let Some(src) = SRC_RE.captures(attrs).and_then(|c| c.get(1)) {
                    let s = src.as_str().trim();
                    if !s.is_empty() && !s.contains("loading.") {
                        raw_url = Some(s);
                    }
                }
            }

            if let Some(url_str) = raw_url {
                let trimmed = url_str.trim();
                let full = if trimmed.starts_with("//") {
                    format!("https:{}", trimmed)
                } else if trimmed.starts_with('/') {
                    format!("https://tw.juqing.app{}", trimmed)
                } else {
                    trimmed.to_string()
                };
                return Some(full);
            }
        }

        Self::parse_cover_url(block)
    }

    pub fn parse_search_results(&self, html: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for cap in SEARCH_ARTICLE_RE.captures_iter(html) {
            let article_block = cap.get(1).map(|m| m.as_str()).unwrap_or_default();

            let link_match = SEARCH_LINK_RE.captures(article_block);
            let raw_url = match link_match {
                Some(caps) => caps.get(1).map(|m| m.as_str().trim().to_string()),
                None => continue,
            };
            let Some(raw_url) = raw_url else { continue };

            let full_url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
                raw_url
            } else if raw_url.starts_with("//") {
                format!("https:{}", raw_url)
            } else if raw_url.starts_with('/') {
                format!("https://tw.juqing.app{}", raw_url)
            } else {
                format!("https://tw.juqing.app/{}", raw_url)
            };

            let title = match Self::parse_article_title(article_block) {
                Some(t) if !t.is_empty() => t,
                _ => continue,
            };

            let year = Self::parse_article_year(article_block);
            let category = Self::parse_article_category(article_block);
            let cover_url = Self::parse_article_cover_url(article_block);

            results.push(SearchResult {
                title,
                year,
                category,
                url: full_url,
                source_id: self.id().to_string(),
                source_name: self.name().to_string(),
                cover_url,
            });
        }

        Ok(results)
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

    fn build_stream_referer(
        &self,
        drama_id: &str,
        ep_id: &str,
        channel: &str,
        category_slug: Option<&str>,
    ) -> String {
        self.build_stream_referer(drama_id, ep_id, channel, category_slug)
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
            .error_for_status()?
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

        let referer = self.build_stream_referer(drama_id, ep_id, channel, None);

        let html = client
            .get(&play_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .header("Referer", &referer)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        Self::parse_m3u8_from_html(&html, Some(channel))
    }

    async fn search(&self, client: &reqwest::Client, keyword: &str) -> Result<Vec<SearchResult>> {
        let kw = keyword.trim();
        if kw.is_empty() {
            return Ok(Vec::new());
        }

        let search_url = Url::parse_with_params(
            "https://tw.juqing.app/vod/search/-------------.html",
            &[("wd", kw)],
        )?;

        let html = client
            .get(search_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        self.parse_search_results(&html)
    }
}
