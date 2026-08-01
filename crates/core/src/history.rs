use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub job_id: String,
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub format_id: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub output_file: Option<String>,
    #[serde(default)]
    pub status: String,
    pub downloaded_at: String,
}

fn history_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("easytube")
        .join("history.json")
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        vec![]
    }
}

pub fn add_history_entry(entry: &HistoryEntry) -> Result<(), String> {
    let mut entries = load_history();
    entries.insert(0, entry.clone());
    if entries.len() > 1000 {
        entries.truncate(1000);
    }
    let dir = history_path().parent().unwrap().to_path_buf();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {}", e))?;
    let json = serde_json::to_string_pretty(&entries).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(history_path(), json).map_err(|e| format!("write: {}", e))?;
    Ok(())
}

pub fn clear_history() -> Result<(), String> {
    let path = history_path();
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("remove: {}", e))?;
    }
    Ok(())
}
