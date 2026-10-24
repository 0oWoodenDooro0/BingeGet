use bingeget::commands::{parse_series_metadata_core, fetch_episode_m3u8_core};

#[tokio::test]
async fn test_parse_series_metadata_empty_or_invalid_url() {
    let err = parse_series_metadata_core("").await.unwrap_err();
    assert!(err.contains("URL cannot be empty"));

    let err2 = parse_series_metadata_core("https://unknown-domain.com/123").await.unwrap_err();
    assert!(err2.contains("Unsupported website URL"));
}

#[tokio::test]
async fn test_fetch_episode_m3u8_validation() {
    let err = fetch_episode_m3u8_core("", "ep1", "ikzy").await.unwrap_err();
    assert!(err.contains("Drama ID cannot be empty"));

    let err2 = fetch_episode_m3u8_core("202614235", "", "ikzy").await.unwrap_err();
    assert!(err2.contains("Episode ID cannot be empty"));

    let err3 = fetch_episode_m3u8_core("202614235", "ep1", "").await.unwrap_err();
    assert!(err3.contains("Channel ID cannot be empty"));
}
