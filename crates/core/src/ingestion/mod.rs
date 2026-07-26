//! Document Ingestion Module - PDF to Markdown + Images pipeline
//! Phase 29: Ingesta Documental

pub mod models;
pub mod mineru_wrapper;
pub mod pymupdf_fallback;
pub mod organizer;
pub mod pdf_processor;
pub mod config;
pub mod plugin;
pub mod progress;

#[cfg(test)]
pub mod tests;

pub use models::{
    IngestionJob, IngestionResult, IngestionStats, IngestionProgress, IngestionStage,
    ProcessingMethod, Chapter, ImageRef, BoundingBox, IngestionError, IngestionMode,
};
pub use pdf_processor::PDFProcessor;
pub use organizer::Organizer;
pub use mineru_wrapper::MinerUWrapper;
pub use plugin::IngestionPlugin;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    pub model_dir: PathBuf,
    pub output_base_dir: PathBuf,
    pub fallback_enabled: bool,
    pub default_ocr_langs: Vec<String>,
    pub max_parallel: usize,
    pub timeout_hours: u64,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            model_dir: PathBuf::from("/models/mineru"),
            output_base_dir: PathBuf::from("/tmp/alesys-ingestion"),
            fallback_enabled: true,
            default_ocr_langs: vec!["en".to_string(), "es".to_string()],
            max_parallel: 1,
            timeout_hours: 20,
        }
    }
}

impl IngestionConfig {
    pub fn from_env() -> Self {
        Self {
            model_dir: PathBuf::from(std::env::var("MINERU_MODEL_DIR").unwrap_or_else(|_| "/models/mineru".to_string())),
            output_base_dir: PathBuf::from(std::env::var("ALESYS_INGESTION_OUTPUT").unwrap_or_else(|_| "/tmp/alesys-ingestion".to_string())),
            fallback_enabled: std::env::var("ALESYS_INGESTION_FALLBACK")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            default_ocr_langs: std::env::var("ALESYS_INGESTION_OCR_LANGS")
                .ok()
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|| vec!["en".to_string(), "es".to_string()]),
            max_parallel: std::env::var("ALESYS_INGESTION_PARALLEL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            timeout_hours: std::env::var("ALESYS_INGESTION_TIMEOUT_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
        }
    }
}