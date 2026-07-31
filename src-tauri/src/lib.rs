use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub formats: Vec<FormatInfo>,
    pub duration: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormatInfo {
    pub id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub job_id: String,
    pub stage: String,
    pub percent: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
}

#[tauri::command]
fn probe_url(url: String) -> Result<VideoInfo, String> {
    Ok(VideoInfo {
        title: "placeholder".into(),
        url,
        thumbnail: None,
        formats: vec![],
        duration: None,
    })
}

#[tauri::command]
fn start_download(url: String, format_id: Option<String>) -> Result<String, String> {
    let _ = (url, format_id);
    Ok("job-001".into())
}

#[tauri::command]
fn cancel_download(job_id: String) -> Result<(), String> {
    let _ = job_id;
    Ok(())
}

#[tauri::command]
fn get_history() -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
fn get_settings() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "clipboard_monitor": false,
        "download_dir": "",
        "language": "zh-TW",
        "theme": "system"
    }))
}

#[tauri::command]
fn save_settings(settings: serde_json::Value) -> Result<(), String> {
    let _ = settings;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                if let Some(tray) = app.tray_by_id("easytube-tray") {
                    let _ = tray.set_show_menu_on_left_click(false);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            probe_url,
            start_download,
            cancel_download,
            get_history,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
