use bingeget::commands::{
    detect_system_environment_core, fetch_episode_m3u8_core, parse_series_metadata_core,
    search_series_core,
};

#[tokio::test]
async fn test_parse_series_metadata_empty_or_invalid_url() {
    let client = reqwest::Client::new();
    let err = parse_series_metadata_core(&client, "").await.unwrap_err();
    assert!(err.contains("URL cannot be empty"));

    let err2 = parse_series_metadata_core(&client, "https://unknown-domain.com/123").await.unwrap_err();
    assert!(err2.contains("Unsupported website URL"));
}

#[tokio::test]
async fn test_fetch_episode_m3u8_validation() {
    let client = reqwest::Client::new();
    let err = fetch_episode_m3u8_core(&client, "", "ep1", "ikzy").await.unwrap_err();
    assert!(err.contains("Drama ID cannot be empty"));

    let err2 = fetch_episode_m3u8_core(&client, "202614235", "", "ikzy").await.unwrap_err();
    assert!(err2.contains("Episode ID cannot be empty"));

    let err3 = fetch_episode_m3u8_core(&client, "202614235", "ep1", "").await.unwrap_err();
    assert!(err3.contains("Channel ID cannot be empty"));
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
}
