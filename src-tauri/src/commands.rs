use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use easytube_core::provider::{BinaryProvider, ProviderType};
use easytube_core::types::*;

use crate::AppState;

pub struct RunningJob {
    pub job: DownloadJob,
    pub child: Arc<tokio::sync::Mutex<Option<Child>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub job_id: String,
    pub stage: String,
    pub percent: f64,
    pub speed_bytes: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

#[tauri::command]
pub async fn probe_url(state: State<'_, AppState>, url: String) -> Result<VideoInfo, String> {
    state.manager.probe(&url).await
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
    format_id: Option<String>,
) -> Result<String, String> {
    let job_id = state
        .manager
        .create_download(url.clone(), format_id.clone(), None)
        .await?;
    debug!(job_id, "download job created");

    let manager = state.manager.clone();
    let running = state.running.clone();
    let id = job_id.clone();
    let app_clone = app.clone();

    tokio::spawn(async move {
        let _permit = manager.semaphore().acquire_owned().await.unwrap();
        run_download_job(&app_clone, &manager, &running, &id).await;
    });

    Ok(job_id)
}

async fn run_download_job(
    app: &AppHandle,
    manager: &easytube_core::state::DownloadManager,
    running: &Arc<RwLock<HashMap<String, RunningJob>>>,
    job_id: &str,
) {
    manager.update_status(job_id, JobStatus::Probe).await;
    emit_stage(app, job_id, "probing", 0.0);

    let job = match manager.get_job(job_id).await {
        Some(j) => j,
        None => {
            error!(job_id, "job disappeared");
            return;
        }
    };

    let info = manager.probe(&job.url).await;
    match &info {
        Ok(v) => {
            manager.update_title(job_id, &v.title).await;
            manager.update_status(job_id, JobStatus::Downloading).await;
            info!(job_id, title = v.title, "probe ok");
        }
        Err(e) => {
            error!(job_id, error = %e, "probe failed");
            manager.update_status(job_id, JobStatus::Failed(e.clone())).await;
            let _ = app.emit("download-failed", serde_json::json!({
                "job_id": job_id,
                "error": e,
            }));
            return;
        }
    }

    emit_stage(app, job_id, "downloading", 0.0);

    let fmt = job.format_id.as_deref().unwrap_or("best");
    let out_dir = job
        .output_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| manager.output_dir().to_string_lossy().to_string());

    let mut cmd = tokio::process::Command::new(manager.ytdlp_path());
    cmd.args([
        "-o",
        &format!("{}/%(title)s.%(ext)s", out_dir),
        "-f",
        fmt,
        "--no-playlist",
        "--ignore-config",
        &job.url,
    ]);

    if let Some(ff) = manager.ffmpeg_path() {
        cmd.arg("--ffmpeg-location");
        cmd.arg(ff.to_string_lossy().to_string());
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            error!(job_id, error = %e, "spawn failed");
            manager.update_status(job_id, JobStatus::Failed(e.to_string())).await;
            let _ = app.emit("download-failed", serde_json::json!({
                "job_id": job_id,
                "error": e.to_string(),
            }));
            return;
        }
    };

    let child_lock = Arc::new(tokio::sync::Mutex::new(None));
    {
        running.write().await.insert(
            job_id.to_string(),
            RunningJob {
                job: manager.get_job(job_id).await.unwrap(),
                child: child_lock.clone(),
            },
        );
    }

    // Stream stderr for progress
    let stderr = child.stderr.take();
    if let Some(stderr) = stderr {
        let job_id = job_id.to_string();
        let app = app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(pct) = extract_percent(&line) {
                    let _ = app.emit("download-progress", ProgressPayload {
                        job_id: job_id.clone(),
                        stage: "downloading".into(),
                        percent: pct,
                        speed_bytes: None,
                        eta_seconds: None,
                        downloaded_bytes: None,
                        total_bytes: None,
                    });
                }
            }
        });
    }

    *child_lock.lock().await = Some(child);
    let mut child = child_lock.lock().await.take().unwrap();

    let output = match child.wait_with_output().await {
        Ok(o) => o,
        Err(e) => {
            error!(job_id, error = %e, "process error");
            manager.update_status(job_id, JobStatus::Failed(e.to_string())).await;
            let _ = app.emit("download-failed", serde_json::json!({
                "job_id": job_id,
                "error": e.to_string(),
            }));
            running.write().await.remove(job_id);
            return;
        }
    };

    running.write().await.remove(job_id);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let output_file = stdout.lines().last().map(|s| s.to_string());
        manager.update_status(job_id, JobStatus::Done).await;
        emit_stage(app, job_id, "done", 100.0);
        let _ = app.emit("download-complete", serde_json::json!({
            "job_id": job_id,
            "output_file": output_file,
        }));
        info!(job_id, "download done");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let err = stderr.lines().last().unwrap_or("unknown").to_string();
        manager.update_status(job_id, JobStatus::Failed(err.clone())).await;
        let _ = app.emit("download-failed", serde_json::json!({
            "job_id": job_id,
            "error": err,
        }));
        error!(job_id, "download failed");
    }
}

fn emit_stage(app: &AppHandle, job_id: &str, stage: &str, percent: f64) {
    let _ = app.emit("download-progress", ProgressPayload {
        job_id: job_id.to_string(),
        stage: stage.to_string(),
        percent,
        speed_bytes: None,
        eta_seconds: None,
        downloaded_bytes: None,
        total_bytes: None,
    });
}

fn extract_percent(line: &str) -> Option<f64> {
    if !line.contains('%') { return None; }
    let pct = line.split('%').next()?.trim().rsplit(' ').next()?.trim().parse::<f64>().ok()?;
    if (0.0..=100.0).contains(&pct) { Some(pct) } else { None }
}

#[tauri::command]
pub async fn cancel_download(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    info!(job_id, "cancelling");
    state.manager.update_status(&job_id, JobStatus::Cancelled).await;

    if let Some(mut entry) = state.running.write().await.remove(&job_id) {
        if let Some(mut child) = entry.child.lock().await.take() {
            if let Err(e) = child.kill().await {
                warn!(job_id, error = %e, "kill failed");
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_history() -> Result<Vec<DownloadJob>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn get_settings() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "clipboard_monitor": false,
        "download_dir": "",
        "language": "zh-TW",
        "theme": "system",
        "concurrent_downloads": 5,
        "max_concurrent": 60
    }))
}

#[tauri::command]
pub async fn save_settings(settings: serde_json::Value) -> Result<(), String> {
    let _ = settings;
    Ok(())
}

#[tauri::command]
pub async fn download_ytdlp() -> Result<String, String> {
    let url = ytdlp_download_url();
    let path = BinaryProvider::download_ytdlp(&url).await?;
    info!(path = %path.display(), "yt-dlp downloaded");
    Ok(path.to_string_lossy().to_string())
}

fn ytdlp_download_url() -> String {
    let version = "2026.07.04";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { format!("https://github.com/yt-dlp/yt-dlp/releases/download/{version}/yt-dlp_macos", version = version) }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { format!("https://github.com/yt-dlp/yt-dlp/releases/download/{version}/yt-dlp_macos", version = version) }
    #[cfg(target_os = "linux")]
    { format!("https://github.com/yt-dlp/yt-dlp/releases/download/{version}/yt-dlp_linux", version = version) }
    #[cfg(target_os = "windows")]
    { format!("https://github.com/yt-dlp/yt-dlp/releases/download/{version}/yt-dlp.exe", version = version) }
}
