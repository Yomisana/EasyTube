use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub download_dir: String,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default)]
    pub clipboard_monitor: bool,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_max_concurrent() -> usize {
    5
}
fn default_language() -> String {
    "zh-TW".into()
}
fn default_theme() -> String {
    "system".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_dir: String::new(),
            max_concurrent: 5,
            clipboard_monitor: false,
            language: "zh-TW".into(),
            theme: "system".into(),
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("easytube")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_settings() -> AppSettings {
    let path = config_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        AppSettings::default()
    }
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create config dir: {}", e))?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(config_path(), json).map_err(|e| format!("write config: {}", e))?;
    Ok(())
}

pub fn get_setting(key: &str) -> String {
    let s = load_settings();
    match key {
        "download_dir" => s.download_dir,
        "max_concurrent" => s.max_concurrent.to_string(),
        "clipboard_monitor" => s.clipboard_monitor.to_string(),
        "language" => s.language,
        "theme" => s.theme,
        _ => format!("Unknown setting: {}", key),
    }
}

pub fn set_setting(key: &str, value: &str) -> Result<(), String> {
    let mut s = load_settings();
    match key {
        "download_dir" => s.download_dir = value.to_string(),
        "max_concurrent" => {
            s.max_concurrent = value
                .parse::<usize>()
                .map_err(|e| format!("invalid number: {}", e))?
                .clamp(1, 60);
        }
        "clipboard_monitor" => {
            s.clipboard_monitor = value
                .parse::<bool>()
                .map_err(|e| format!("invalid boolean: {}", e))?;
        }
        "language" => s.language = value.to_string(),
        "theme" => s.theme = value.to_string(),
        _ => return Err(format!("Unknown setting: {}", key)),
    }
    save_settings(&s)?;
    Ok(())
}
