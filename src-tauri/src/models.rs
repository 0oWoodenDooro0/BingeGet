use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub ep_number: u32,
    pub ep_id: String,
    #[serde(default)]
    pub available_channels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMetadata {
    pub drama_id: String,
    pub episodes: Vec<EpisodeInfo>,
    pub channels: Vec<ChannelOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEnvironment {
    pub nm3u8dl_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub is_ready: bool,
}
