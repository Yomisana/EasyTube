use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{RwLock, Semaphore};
use tracing::info;

use crate::types::*;

const DEFAULT_MAX_CONCURRENT: usize = 5;
const MAX_CONCURRENT_LIMIT: usize = 60;

pub struct DownloadManager {
    jobs: Arc<RwLock<HashMap<String, DownloadJob>>>,
    semaphore: Arc<Semaphore>,
    settings: RwLock<DownloadSettings>,
    output_dir: PathBuf,
    ytdlp_path: PathBuf,
    ffmpeg_path: Option<PathBuf>,
    event_tx: tokio::sync::broadcast::Sender<DownloadEvent>,
}

#[derive(Debug, Clone)]
pub struct DownloadSettings {
    pub max_concurrent: usize,
    pub output_dir: PathBuf,
    pub retry_count: u32,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            max_concurrent: DEFAULT_MAX_CONCURRENT,
            output_dir: dirs::download_dir()
                .or_else(|| dirs::home_dir().map(|p| p.join("Downloads")))
                .unwrap_or_else(|| PathBuf::from(".")),
            retry_count: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum DownloadEvent {
    Progress(DownloadProgress),
    Complete { job_id: String, output_path: Option<String> },
    Failed { job_id: String, error: String },
}

impl DownloadManager {
    pub fn new(
        settings: DownloadSettings,
        ytdlp_path: PathBuf,
        ffmpeg_path: Option<PathBuf>,
    ) -> Self {
        let max = settings.max_concurrent.min(MAX_CONCURRENT_LIMIT).max(1);
        let output_dir = settings.output_dir.clone();
        let (tx, _) = tokio::sync::broadcast::channel(256);
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max)),
            settings: RwLock::new(settings),
            output_dir,
            ytdlp_path,
            ffmpeg_path,
            event_tx: tx,
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<DownloadEvent> {
        self.event_tx.subscribe()
    }

    pub async fn set_max_concurrent(&self, n: usize) {
        let n = n.min(MAX_CONCURRENT_LIMIT).max(1);
        self.settings.write().await.max_concurrent = n;
        info!(n, "max concurrent updated");
    }

    pub async fn set_output_dir(&self, dir: PathBuf) {
        self.settings.write().await.output_dir = dir;
    }

    pub async fn max_concurrent(&self) -> usize {
        self.settings.read().await.max_concurrent
    }

    pub fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    pub fn ytdlp_path(&self) -> &PathBuf {
        &self.ytdlp_path
    }

    pub fn ffmpeg_path(&self) -> Option<&PathBuf> {
        self.ffmpeg_path.as_ref()
    }

    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    pub async fn retry_count(&self) -> u32 {
        self.settings.read().await.retry_count
    }

    pub async fn probe(&self, url: &str) -> Result<VideoInfo, String> {
        info!(url, "probing");
        let output = tokio::process::Command::new(&self.ytdlp_path)
            .args(["-J", "--no-playlist", "--ignore-config", url])
            .output()
            .await
            .map_err(|e| format!("spawn yt-dlp: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Unsupported URL") || stderr.contains("not a valid URL") {
                return Err(format!("INVALID_URL: {}", url));
            }
            return Err(format!("PROBE_FAILED: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let raw: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("parse probe: {}", e))?;

        let title = raw["title"].as_str().unwrap_or(url).to_string();
        let thumbnail = raw["thumbnail"].as_str().map(|s| s.to_string());
        let duration = raw["duration"].as_i64().map(|d| format_duration(d as u64));

        let formats: Vec<FormatInfo> = raw["formats"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| {
                        let id = f["format_id"].as_str()?.to_string();
                        let ext = f["ext"].as_str().unwrap_or("?").to_string();
                        let resolution = f["resolution"].as_str().map(|s| s.to_string());
                        let note = format_note(f);
                        let filesize = f["filesize"].as_i64();
                        Some(FormatInfo { id, ext, resolution, filesize, note })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(VideoInfo {
            title,
            url: url.to_string(),
            thumbnail,
            formats,
            duration,
        })
    }

    pub async fn create_download(
        &self,
        url: String,
        format_id: Option<String>,
        output_dir: Option<PathBuf>,
    ) -> Result<String, String> {
        let default_dir = self.settings.read().await.output_dir.clone();
        let out = output_dir.unwrap_or(default_dir);

        let job_id = uuid::Uuid::new_v4().to_string();
        let job = DownloadJob {
            id: job_id.clone(),
            url,
            title: None,
            thumbnail: None,
            format_id,
            output_dir: Some(out),
            status: JobStatus::Pending,
            progress: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.jobs.write().await.insert(job_id.clone(), job);
        info!(job_id, "job created");
        Ok(job_id)
    }

    pub async fn get_job(&self, job_id: &str) -> Option<DownloadJob> {
        self.jobs.read().await.get(job_id).cloned()
    }

    pub async fn all_jobs(&self) -> Vec<DownloadJob> {
        self.jobs.read().await.values().cloned().collect()
    }

    pub async fn update_status(&self, job_id: &str, status: JobStatus) {
        if let Some(job) = self.jobs.write().await.get_mut(job_id) {
            job.status = status;
        }
    }

    pub async fn update_title(&self, job_id: &str, title: &str) {
        if let Some(job) = self.jobs.write().await.get_mut(job_id) {
            job.title = Some(title.to_string());
        }
    }

    pub fn emit(&self, event: DownloadEvent) {
        let _ = self.event_tx.send(event);
    }
}

fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 { format!("{}:{:02}:{:02}", h, m, s) } else { format!("{}:{:02}", m, s) }
}

fn format_note(f: &serde_json::Value) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(res) = f["resolution"].as_str() { parts.push(res.to_string()); }
    if let Some(fps) = f["fps"].as_f64() { parts.push(format!("{:.0}fps", fps)); }
    if let Some(abr) = f["abr"].as_f64() { parts.push(format!("{:.0}kbps", abr)); }
    if let Some(vc) = f["vcodec"].as_str() { if vc != "none" { parts.push(vc.to_string()); } }
    if parts.is_empty() { None } else { Some(parts.join(" | ")) }
}
