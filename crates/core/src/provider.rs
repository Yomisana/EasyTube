use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum BinaryProviderType {
    Downloaded,
    System,
    Custom(PathBuf),
}

pub struct BinaryProvider;

impl BinaryProvider {
    pub async fn resolve_ytdlp(&self, _provider_type: &BinaryProviderType) -> Result<PathBuf, String> {
        Err("not implemented".into())
    }

    pub async fn resolve_ffmpeg(&self, _provider_type: &BinaryProviderType) -> Result<PathBuf, String> {
        Err("not implemented".into())
    }
}
