//! Ingestion Configuration

use crate::ingestion::IngestionConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model_dir: PathBuf,
    pub output_base_dir: PathBuf,
    pub fallback_enabled: bool,
    pub default_ocr_langs: Vec<String>,
    pub max_parallel: usize,
    pub timeout_hours: u64,
}

impl From<IngestionConfig> for Config {
    fn from(c: IngestionConfig) -> Self {
        Self {
            model_dir: c.model_dir,
            output_base_dir: c.output_base_dir,
            fallback_enabled: c.fallback_enabled,
            default_ocr_langs: c.default_ocr_langs,
            max_parallel: c.max_parallel,
            timeout_hours: c.timeout_hours,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_dir: PathBuf::from(std::env::var("MINERU_MODEL_DIR").unwrap_or_else(|_| "/models/mineru".to_string())),
            output_base_dir: PathBuf::from(std::env::var("ALESYS_INGESTION_OUTPUT").unwrap_or_else(|_| "/tmp/alesys-ingestion".to_string())),
            fallback_enabled: true,
            default_ocr_langs: vec!["en".to_string(), "es".to_string()],
            max_parallel: 1,
            timeout_hours: 20,
        }
    }
}