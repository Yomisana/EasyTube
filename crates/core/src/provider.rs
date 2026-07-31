use std::path::{Path, PathBuf};
use std::process::Command as SyncCommand;

use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub enum ProviderType {
    Downloaded,
    System,
    Custom(PathBuf),
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub url_template: Option<String>,
    pub pinned_version: Option<String>,
}

pub struct BinaryProvider;

impl BinaryProvider {
    pub fn find_ytdlp(preferred: &ProviderType) -> Option<PathBuf> {
        match preferred {
            ProviderType::Custom(p) if p.exists() => {
                info!(path = %p.display(), "using custom yt-dlp");
                Some(p.clone())
            }
            ProviderType::System => {
                match which::which("yt-dlp") {
                    Ok(p) => {
                        info!(path = %p.display(), "found system yt-dlp");
                        Some(p)
                    }
                    Err(_) => {
                        debug!("yt-dlp not found on PATH");
                        None
                    }
                }
            }
            _ => {
                let path = app_data_ytdlp();
                if path.exists() {
                    info!(path = %path.display(), "found downloaded yt-dlp");
                    Some(path)
                } else {
                    None
                }
            }
        }
    }

    pub fn find_ffmpeg(preferred: &ProviderType) -> Option<PathBuf> {
        match preferred {
            ProviderType::Custom(p) if p.exists() => Some(p.clone()),
            ProviderType::System => {
                match which::which("ffmpeg") {
                    Ok(p) => {
                        info!(path = %p.display(), "found system ffmpeg");
                        Some(p)
                    }
                    Err(_) => {
                        debug!("ffmpeg not found on PATH");
                        None
                    }
                }
            }
            _ => {
                let path = app_data_ffmpeg();
                if path.exists() { Some(path) } else { None }
            }
        }
    }

    pub async fn download_ytdlp(url: &str) -> Result<PathBuf, String> {
        let dest = app_data_ytdlp();
        if dest.exists() {
            debug!("yt-dlp already downloaded at {}", dest.display());
            return Ok(dest);
        }

        info!(url, "downloading yt-dlp");
        let bytes = download_file(url).await?;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create dir: {}", e))?;
        }

        tokio::fs::write(&dest, &bytes)
            .await
            .map_err(|e| format!("failed to write yt-dlp: {}", e))?;

        set_executable(&dest)?;
        strip_quarantine(&dest);

        info!(path = %dest.display(), "yt-dlp downloaded and ready");
        Ok(dest)
    }

    pub async fn download_ffmpeg(url: &str) -> Result<PathBuf, String> {
        let dest = app_data_ffmpeg();
        if dest.exists() {
            debug!("ffmpeg already downloaded at {}", dest.display());
            return Ok(dest);
        }

        info!(url, "downloading ffmpeg");
        let bytes = download_file(url).await?;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create dir: {}", e))?;
        }

        tokio::fs::write(&dest, &bytes)
            .await
            .map_err(|e| format!("failed to write ffmpeg: {}", e))?;

        set_executable(&dest)?;
        strip_quarantine(&dest);

        info!(path = %dest.display(), "ffmpeg downloaded and ready");
        Ok(dest)
    }
}

fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("easytube")
}

fn app_data_ytdlp() -> PathBuf {
    let name = if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };
    app_data_dir().join(name)
}

fn app_data_ffmpeg() -> PathBuf {
    let name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    app_data_dir().join(name)
}

async fn download_file(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .user_agent("EasyTube/0.1")
        .build()
        .map_err(|e| format!("failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "download returned HTTP {}",
            response.status()
        ));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("download read failed: {}", e))
}

fn set_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(|e| format!("metadata: {}", e))?
            .permissions();
        perms.set_mode(perms.mode() | 0o111);
        std::fs::set_permissions(path, perms)
            .map_err(|e| format!("chmod: {}", e))?;
    }
    Ok(())
}

fn strip_quarantine(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let path_str = path.to_string_lossy();
        match SyncCommand::new("xattr")
            .args(["-d", "com.apple.quarantine", &path_str])
            .output()
        {
            Ok(o) if o.status.success() => debug!("removed quarantine xattr"),
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if !stderr.contains("No such xattr") {
                    warn!(?stderr, "xattr removal reported");
                }
            }
            Err(e) => warn!(?e, "xattr command failed"),
        }

        match SyncCommand::new("codesign")
            .args(["-s", "-", &path_str])
            .output()
        {
            Ok(o) if o.status.success() => debug!("ad-hoc codesigned binary"),
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                warn!(?stderr, "codesign reported");
            }
            Err(e) => warn!(?e, "codesign command failed"),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_dir_returns_path() {
        let dir = app_data_dir();
        assert!(dir.ends_with("easytube"));
    }

    #[test]
    fn test_ytdlp_binary_name() {
        let path = app_data_ytdlp();
        let name = path.file_name().unwrap().to_str().unwrap();
        if cfg!(target_os = "windows") {
            assert_eq!(name, "yt-dlp.exe");
        } else {
            assert_eq!(name, "yt-dlp");
        }
    }
}
