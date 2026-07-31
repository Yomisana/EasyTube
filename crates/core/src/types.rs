use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub formats: Vec<FormatInfo>,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadJob {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub format_id: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub status: JobStatus,
    pub progress: Option<DownloadProgress>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Probe,
    Downloading,
    Verifying,
    Done,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub job_id: String,
    pub stage: String,
    pub percent: f64,
    pub speed_bytes: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

impl DownloadProgress {
    pub fn preparing(id: &str) -> Self {
        Self {
            job_id: id.to_string(),
            stage: "preparing".into(),
            percent: 0.0,
            speed_bytes: None,
            eta_seconds: None,
            downloaded_bytes: None,
            total_bytes: None,
        }
    }

    pub fn done(id: &str) -> Self {
        Self {
            job_id: id.to_string(),
            stage: "done".into(),
            percent: 100.0,
            speed_bytes: None,
            eta_seconds: None,
            downloaded_bytes: None,
            total_bytes: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub clipboard_monitor: bool,
    pub download_dir: Option<String>,
    pub language: String,
    pub theme: String,
    pub concurrent_downloads: u32,
    pub max_concurrent: u32,
    pub filter_allowlist: Vec<String>,
    pub tracking_params_blacklist: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            clipboard_monitor: false,
            download_dir: None,
            language: "zh-TW".into(),
            theme: "system".into(),
            concurrent_downloads: 5,
            max_concurrent: 60,
            filter_allowlist: vec![
                "youtube.com".into(),
                "youtu.be".into(),
                "bilibili.com".into(),
                "nicovideo.jp".into(),
                "twitch.tv".into(),
            ],
            tracking_params_blacklist: vec![
                "utm_source".into(),
                "utm_medium".into(),
                "utm_campaign".into(),
                "utm_term".into(),
                "utm_content".into(),
                "fbclid".into(),
                "gclid".into(),
                "igshid".into(),
                "ref".into(),
            ],
        }
    }
}
