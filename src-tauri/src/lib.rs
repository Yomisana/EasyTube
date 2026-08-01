use std::collections::HashMap;
use std::sync::Arc;

use easytube_core::provider::{BinaryProvider, ProviderType};
use easytube_core::state::{DownloadManager, DownloadSettings};
use tokio::sync::RwLock;

mod clipboard;
mod commands;

use clipboard::ClipboardMonitor;
use commands::RunningJob;

pub struct AppState {
    pub manager: Arc<DownloadManager>,
    pub running: Arc<RwLock<HashMap<String, RunningJob>>>,
    pub clipboard: Arc<ClipboardMonitor>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let ytdlp = BinaryProvider::find_ytdlp(&ProviderType::System)
            .or_else(|| BinaryProvider::find_ytdlp(&ProviderType::Downloaded))
            .unwrap_or_else(|| std::path::PathBuf::from("yt-dlp"));

        let ffmpeg = BinaryProvider::find_ffmpeg(&ProviderType::System)
            .or_else(|| BinaryProvider::find_ffmpeg(&ProviderType::Downloaded));

        let settings = DownloadSettings::default();
        Self {
            manager: Arc::new(DownloadManager::new(settings, ytdlp, ffmpeg)),
            running: Arc::new(RwLock::new(HashMap::new())),
            clipboard: Arc::new(ClipboardMonitor::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let _tray = app.tray_by_id("easytube-tray");
            Ok(())
        })
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::probe_url,
            commands::start_download,
            commands::cancel_download,
            commands::get_history,
            commands::get_settings,
            commands::save_settings,
            commands::download_ytdlp,
            commands::check_clipboard,
            commands::set_clipboard_enabled,
            commands::get_clipboard_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
