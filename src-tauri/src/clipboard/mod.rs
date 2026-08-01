use std::sync::Arc;

use easytube_core::filter::{run_filter_pipeline, FilterPipeline, FilterResult};
use tokio::sync::RwLock;
use tracing::{debug, info};

pub struct ClipboardMonitor {
    enabled: Arc<RwLock<bool>>,
    interval_ms: Arc<RwLock<u64>>,
    pipeline: Arc<RwLock<FilterPipeline>>,
    allowlist: Arc<RwLock<Vec<String>>>,
    blocklist: Arc<RwLock<Vec<String>>>,
    tracking_blacklist: Arc<RwLock<Vec<String>>>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        let default_allowlist = vec![
            "youtube.com".into(),
            "youtu.be".into(),
            "bilibili.com".into(),
            "nicovideo.jp".into(),
            "twitch.tv".into(),
        ];
        let default_tracking = vec![
            "utm_source".into(),
            "utm_medium".into(),
            "utm_campaign".into(),
            "utm_term".into(),
            "utm_content".into(),
            "fbclid".into(),
            "gclid".into(),
            "igshid".into(),
            "ref".into(),
        ];
        Self {
            enabled: Arc::new(RwLock::new(false)),
            interval_ms: Arc::new(RwLock::new(1000)),
            pipeline: Arc::new(RwLock::new(FilterPipeline::new(30))),
            allowlist: Arc::new(RwLock::new(default_allowlist)),
            blocklist: Arc::new(RwLock::new(vec![])),
            tracking_blacklist: Arc::new(RwLock::new(default_tracking)),
        }
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
        info!(enabled, "clipboard monitor");
    }

    pub async fn set_interval_ms(&self, ms: u64) {
        *self.interval_ms.write().await = ms;
    }

    pub async fn set_allowlist(&self, list: Vec<String>) {
        *self.allowlist.write().await = list;
    }

    pub async fn set_blocklist(&self, list: Vec<String>) {
        *self.blocklist.write().await = list;
    }

    pub async fn set_tracking_blacklist(&self, list: Vec<String>) {
        *self.tracking_blacklist.write().await = list;
    }

    pub async fn check_clipboard_text(&self, text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }

        let mut pipeline = self.pipeline.write().await;
        let allowlist = self.allowlist.read().await;
        let blocklist = self.blocklist.read().await;
        let tracking_blacklist = self.tracking_blacklist.read().await;

        debug!("checking clipboard: {:.100}", text);

        match run_filter_pipeline(
            text,
            &mut pipeline,
            &allowlist,
            &blocklist,
            &tracking_blacklist,
            &[],
        ) {
            FilterResult::Valid { url, .. } => {
                info!(url, "clipboard URL detected");
                Some(url)
            }
            FilterResult::Ignored(reason) => {
                debug!(reason, "clipboard text ignored");
                None
            }
        }
    }
}
