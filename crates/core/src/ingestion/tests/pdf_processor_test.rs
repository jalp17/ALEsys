use crate::ingestion::{
    IngestionConfig, IngestionJob, IngestionMode, PDFProcessor,
};
use std::path::PathBuf;
use uuid::Uuid;

fn make_config() -> IngestionConfig {
    IngestionConfig {
        model_dir: PathBuf::from("/tmp/test-models"),
        output_base_dir: PathBuf::from("/tmp/test-output"),
        fallback_enabled: true,
        default_ocr_langs: vec!["en".to_string()],
        max_parallel: 2,
        timeout_hours: 1,
    }
}

#[test]
fn test_pdf_processor_creation() {
    let processor = PDFProcessor::new(make_config());
}

#[test]
fn test_pdf_processor_new_with_dir() {
    let processor = PDFProcessor::new_with_dir(PathBuf::from("/tmp/models"), 4);
}

#[test]
fn test_pdf_processor_max_parallel() {
    let processor = PDFProcessor::new(make_config());
    assert_eq!(processor.max_parallel(10), 2);
    assert_eq!(processor.max_parallel(1), 1);
}

#[test]
fn test_pdf_processor_set_max_parallel() {
    let mut processor = PDFProcessor::new(make_config());
    processor.set_max_parallel(8);
    assert_eq!(processor.max_parallel(10), 8);
}

#[tokio::test]
async fn test_pdf_processor_process_nonexistent_pdf() {
    let processor = PDFProcessor::new(make_config());
    let job = IngestionJob {
        id: Uuid::new_v4(),
        pdf_path: PathBuf::from("/tmp/does-not-exist.pdf"),
        topic: "test".to_string(),
        session_id: None,
        mode: IngestionMode::FilesOnly,
        force_fallback: false,
        ocr_languages: vec!["en".to_string()],
        extract_formulas: true,
        extract_tables: true,
    };
    let result = processor.process(job).await;
    assert!(result.is_err());
}
