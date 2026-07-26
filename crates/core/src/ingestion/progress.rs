use crate::ingestion::models::{IngestionProgress, IngestionStage, IngestionMode, Result};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

pub struct ProgressTracker {
    tx: Sender<IngestionProgress>,
    job_id: Uuid,
    mode: IngestionMode,
}

impl ProgressTracker {
    pub fn new(tx: Sender<IngestionProgress>, job_id: Uuid, mode: IngestionMode) -> Self {
        Self { tx, job_id, mode }
    }

    pub async fn send_stage(&self, stage: IngestionStage, pct: f32, message: impl Into<String>) {
        let _ = self
            .tx
            .send(IngestionProgress {
                job_id: self.job_id,
                stage,
                mode: self.mode.clone(),
                message: message.into(),
                progress_pct: pct,
                current_page: None,
                total_pages: None,
                database_indexed: None,
            })
            .await;
    }

    pub async fn starting(&self) {
        self.send_stage(IngestionStage::Starting, 0.0, "Starting ingestion").await;
    }

    pub async fn detecting_gpu(&self) {
        self.send_stage(IngestionStage::DetectingGpu, 5.0, "Detecting GPU").await;
    }

    pub async fn downloading_models(&self) {
        self.send_stage(IngestionStage::DownloadingModels, 10.0, "Downloading models")
            .await;
    }

    pub async fn running_mineru(&self, message: impl Into<String>) {
        self.send_stage(IngestionStage::RunningMinerU, 30.0, message)
            .await;
    }

    pub async fn running_fallback(&self, message: impl Into<String>) {
        self.send_stage(IngestionStage::RunningFallback, 30.0, message)
            .await;
    }

    pub async fn organizing(&self) {
        self.send_stage(IngestionStage::OrganizingOutput, 80.0, "Organizing output")
            .await;
    }

    pub async fn indexing_graphrag(&self) {
        self.send_stage(IngestionStage::IndexingGraphRAG, 95.0, "Indexing in GraphRAG")
            .await;
    }

    pub async fn completed(&self) {
        self.send_stage(IngestionStage::Completed, 100.0, "Completed")
            .await;
    }

    pub async fn failed(&self, error: impl Into<String>) {
        self.send_stage(IngestionStage::Failed, 0.0, error).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_progress_tracker_lifecycle() {
        let (tx, mut rx) = mpsc::channel(8);
        let tracker = ProgressTracker::new(tx, Uuid::new_v4(), IngestionMode::FilesOnly);

        tracker.starting().await;
        tracker.detecting_gpu().await;
        tracker.running_mineru("Processing page 1").await;
        tracker.organizing().await;
        tracker.completed().await;

        let stages: Vec<IngestionStage> = vec![
            IngestionStage::Starting,
            IngestionStage::DetectingGpu,
            IngestionStage::RunningMinerU,
            IngestionStage::OrganizingOutput,
            IngestionStage::Completed,
        ];

        for expected in stages {
            let received = rx.recv().await.unwrap();
            assert_eq!(received.stage, expected);
        }
    }

    #[tokio::test]
    async fn test_progress_tracker_failed() {
        let (tx, mut rx) = mpsc::channel(8);
        let tracker = ProgressTracker::new(tx, Uuid::new_v4(), IngestionMode::Full);

        tracker.failed("Something went wrong").await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.stage, IngestionStage::Failed);
        assert_eq!(received.message, "Something went wrong");
    }
}
