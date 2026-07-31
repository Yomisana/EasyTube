use easytube_core::state::{DownloadManager, DownloadSettings};
use std::path::PathBuf;

#[tokio::test]
#[ignore = "requires yt-dlp on PATH and network"]
async fn test_real_probe_public_video() {
    let ytdlp = which::which("yt-dlp").unwrap_or_else(|_| PathBuf::from("yt-dlp"));
    let manager = DownloadManager::new(DownloadSettings::default(), ytdlp, None);

    let info = manager
        .probe("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
        .await
        .expect("probe should succeed");

    assert!(!info.title.is_empty(), "title should not be empty");
    assert!(!info.formats.is_empty(), "should have format options");
    println!("Probed: {} ({} formats)", info.title, info.formats.len());
}

#[tokio::test]
#[ignore = "requires yt-dlp on PATH"]
async fn test_probe_invalid_url() {
    let ytdlp = which::which("yt-dlp").unwrap_or_else(|_| PathBuf::from("yt-dlp"));
    let manager = DownloadManager::new(DownloadSettings::default(), ytdlp, None);

    let result = manager.probe("not-a-valid-url").await;
    assert!(result.is_err());
}

#[test]
fn test_download_settings_defaults() {
    let s = DownloadSettings::default();
    assert!(s.max_concurrent == 5);
    assert!(s.retry_count == 3);
}
