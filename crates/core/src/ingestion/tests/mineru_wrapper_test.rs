use crate::ingestion::mineru_wrapper::{MinerUInfo, MinerUOutput, MinerUWrapper};
use crate::ingestion::{IngestionConfig, IngestionJob, IngestionMode};
use std::path::PathBuf;
use uuid::Uuid;

fn make_job() -> IngestionJob {
    IngestionJob {
        id: Uuid::new_v4(),
        pdf_path: PathBuf::from("/tmp/test.pdf"),
        topic: "test".to_string(),
        session_id: None,
        mode: IngestionMode::FilesOnly,
        force_fallback: false,
        ocr_languages: vec!["en".to_string()],
        extract_formulas: true,
        extract_tables: true,
    }
}

fn make_config() -> IngestionConfig {
    IngestionConfig {
        model_dir: PathBuf::from("/tmp/test-models"),
        output_base_dir: PathBuf::from("/tmp/test-output"),
        fallback_enabled: true,
        default_ocr_langs: vec!["en".to_string()],
        max_parallel: 1,
        timeout_hours: 20,
    }
}

#[test]
fn test_mineru_wrapper_new() {
    let wrapper = MinerUWrapper::new(PathBuf::from("/tmp/models"), true);
    assert_eq!(wrapper.model_dir, PathBuf::from("/tmp/models"));
    assert!(wrapper.use_gpu);
}

#[test]
fn test_mineru_info_struct() {
    let info = MinerUInfo {
        version: "1.0".to_string(),
        gpu_available: true,
        models_exist: true,
        model_dir: PathBuf::from("/tmp/models"),
    };
    assert!(info.gpu_available);
    assert!(info.models_exist);
}

#[test]
fn test_mineru_output_struct() {
    let output = MinerUOutput {
        job_id: Uuid::new_v4(),
        markdown_path: PathBuf::from("/tmp/out.md"),
        images_dir: Some(PathBuf::from("/tmp/images")),
        auto_dir: PathBuf::from("/tmp/auto"),
        method: crate::ingestion::models::ProcessingMethod::MinerU {
            gpu: true,
            model_version: "v1".to_string(),
        },
    };
    assert!(output.images_dir.is_some());
}

#[tokio::test]
async fn test_check_gpu_when_disabled() {
    let wrapper = MinerUWrapper::new(PathBuf::from("/tmp/models"), false);
    let gpu = wrapper.check_gpu().await;
    assert!(!gpu);
}

#[tokio::test]
async fn test_check_models_when_missing() {
    let wrapper = MinerUWrapper::new(PathBuf::from("/tmp/nonexistent-models"), false);
    let models = wrapper.check_models().await;
    assert!(!models);
}

#[test]
fn test_parse_progress_layout() {
    let (stage, pct, msg) = MinerUWrapper::parse_progress("Layout analysis complete").unwrap();
    assert_eq!(stage, crate::ingestion::models::IngestionStage::RunningMinerU);
    assert_eq!(pct, 30.0);
    assert_eq!(msg, "Layout analysis");
}

#[test]
fn test_parse_progress_ocr() {
    let (stage, pct, msg) = MinerUWrapper::parse_progress("OCR processing started").unwrap();
    assert_eq!(stage, crate::ingestion::models::IngestionStage::RunningMinerU);
    assert_eq!(pct, 55.0);
    assert_eq!(msg, "OCR processing");
}

#[test]
fn test_parse_progress_no_match() {
    let result = MinerUWrapper::parse_progress("some random line");
    assert!(result.is_none());
}
