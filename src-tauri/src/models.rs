use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct EpisodeInfo {
    pub ep_number: u32,
    pub ep_id: String,
    #[serde(default)]
    pub available_channels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ChannelOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct SeriesMetadata {
    pub drama_id: String,
    #[serde(default)]
    pub category_slug: Option<String>,
    pub episodes: Vec<EpisodeInfo>,
    pub channels: Vec<ChannelOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct SearchResult {
    pub title: String,
    pub year: Option<String>,
    pub category: Option<String>,
    pub url: String,
    pub source_id: String,
    pub source_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct SystemEnvironment {
    pub nm3u8dl_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub is_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DownloadQueuePayload {
    pub drama_id: String,
    pub channel_id: String,
    pub episodes: Vec<u32>,
    pub custom_title: String,
    pub season: u32,
    pub save_dir: String,
    pub threads: u32,
    pub max_concurrent: usize,
    pub binary_path: Option<String>,
    #[serde(default)]
    pub category_slug: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ProgressPayload {
    pub episode: u32,
    pub percent: f32,
    pub speed: String,
    pub eta: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct EpisodeStatusPayload {
    pub episode: u32,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DownloadTaskSpec {
    pub m3u8_url: String,
    pub custom_title: String,
    pub season: u32,
    pub episode: u32,
    pub save_dir: String,
    pub referer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DownloadProgress {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
}
