use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionJob {
    pub id: Uuid,
    pub pdf_path: PathBuf,
    pub topic: String,
    pub session_id: Option<Uuid>,
    pub mode: IngestionMode,
    pub force_fallback: bool,
    pub ocr_languages: Vec<String>,
    pub extract_formulas: bool,
    pub extract_tables: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum IngestionMode {
    #[default]
    Full,
    FilesOnly,
}

impl Default for IngestionJob {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            pdf_path: PathBuf::new(),
            topic: String::new(),
            session_id: None,
            mode: IngestionMode::FilesOnly,
            force_fallback: false,
            ocr_languages: vec!["en".to_string(), "es".to_string()],
            extract_formulas: true,
            extract_tables: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IngestionResult {
    pub job_id: Uuid,
    pub success: bool,
    pub mode: IngestionMode,
    pub output_dir: PathBuf,
    pub markdown_path: PathBuf,
    pub images_dir: PathBuf,
    pub database_generated: bool,
    pub database_path: Option<PathBuf>,
    pub chapters: Vec<Chapter>,
    pub images: Vec<ImageRef>,
    pub citations: Vec<crate::bibliography::Citation>,
    pub stats: IngestionStats,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: Uuid,
    pub title: String,
    pub level: u8,
    pub start_page: u32,
    pub end_page: u32,
    pub markdown_path: PathBuf,
    pub image_refs: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub markdown_ref: String,
    pub filesystem_path: PathBuf,
    pub ocr_text: Option<String>,
    pub is_formula: bool,
    pub bbox: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IngestionStats {
    pub pages_processed: u32,
    pub chars_extracted: u64,
    pub images_extracted: u32,
    pub formulas_detected: u32,
    pub tables_detected: u32,
    pub processing_time_ms: u64,
    pub method_used: ProcessingMethod,
    pub database_chunks: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingMethod {
    MinerU { gpu: bool, model_version: String },
    PyMuPDF { ocr_enabled: bool },
    Colab { engine: String },
}

impl Default for ProcessingMethod {
    fn default() -> Self {
        ProcessingMethod::PyMuPDF { ocr_enabled: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionProgress {
    pub job_id: Uuid,
    pub stage: IngestionStage,
    pub mode: IngestionMode,
    pub message: String,
    pub progress_pct: f32,
    pub current_page: Option<u32>,
    pub total_pages: Option<u32>,
    pub database_indexed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IngestionStage {
    Starting,
    DetectingGpu,
    DownloadingModels,
    RunningMinerU,
    RunningFallback,
    OrganizingOutput,
    IndexingGraphRAG,
    Completed,
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum IngestionError {
    #[error("PDF not found: {0}")]
    PdfNotFound(PathBuf),
    
    #[error("MinerU execution failed: {0}")]
    MinerUFailed(String),
    
    #[error("Fallback extraction failed: {0}")]
    FallbackFailed(String),
    
    #[error("Organizer error: {0}")]
    OrganizerError(String),
    
    #[error("GraphRAG indexing failed: {0}")]
    GraphRagError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, IngestionError>;