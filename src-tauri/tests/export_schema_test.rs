use bingeget::models::*;
use ts_rs::{Config, TS};

#[test]
fn test_export_all_typescript_schemas() {
    let cfg = Config::default();
    EpisodeInfo::export(&cfg).expect("failed to export EpisodeInfo");
    ChannelOption::export(&cfg).expect("failed to export ChannelOption");
    SeriesMetadata::export(&cfg).expect("failed to export SeriesMetadata");
    SearchResult::export(&cfg).expect("failed to export SearchResult");
    SystemEnvironment::export(&cfg).expect("failed to export SystemEnvironment");
    DownloadQueuePayload::export(&cfg).expect("failed to export DownloadQueuePayload");
    ProgressPayload::export(&cfg).expect("failed to export ProgressPayload");
    EpisodeStatusPayload::export(&cfg).expect("failed to export EpisodeStatusPayload");
    DownloadTaskSpec::export(&cfg).expect("failed to export DownloadTaskSpec");
    DownloadProgress::export(&cfg).expect("failed to export DownloadProgress");
    StreamDetails::export(&cfg).expect("failed to export StreamDetails");
}
