use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::{DownloadJob, DownloadProgress, JobStatus};

pub struct DownloadManager {
    jobs: Arc<RwLock<HashMap<String, DownloadJob>>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_job(&self, url: &str, format_id: Option<&str>) -> String {
        let job_id = uuid::Uuid::new_v4().to_string();
        let job = DownloadJob {
            id: job_id.clone(),
            url: url.to_string(),
            title: None,
            format_id: format_id.map(|s| s.to_string()),
            output_dir: None,
            status: JobStatus::Pending,
            progress: None,
        };
        self.jobs.write().await.insert(job_id.clone(), job);
        job_id
    }

    pub async fn get_progress(&self, job_id: &str) -> Option<DownloadProgress> {
        self.jobs.read().await.get(job_id)?.progress.clone()
    }

    pub async fn cancel(&self, job_id: &str) {
        if let Some(job) = self.jobs.write().await.get_mut(job_id) {
            job.status = JobStatus::Cancelled;
        }
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}
