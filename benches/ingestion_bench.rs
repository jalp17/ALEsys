use alesys_core::ingestion::{PDFProcessor, IngestionConfig};
use std::time::Instant;
use tempfile::tempdir;

fn bench_pdf_processor_creation() {
    let temp = tempdir().unwrap();
    let config = IngestionConfig::default();
    let start = Instant::now();
    let _ = PDFProcessor::new_with_dir(config.model_dir.clone(), config.max_parallel);
    let elapsed = start.elapsed();
    println!("pdf_processor_creation: {:?}", elapsed);
}

fn bench_organizer_reorganize() {
    let temp = tempdir().unwrap();
    let auto_dir = temp.path().join("auto");
    std::fs::create_dir_all(&auto_dir).unwrap();
    let md_path = auto_dir.join("chapter.md");
    std::fs::write(&md_path, "# Title\n\nContent.\n![img](img.png)\n").unwrap();
    let images_dir = auto_dir.join("images");
    std::fs::create_dir_all(&images_dir).unwrap();
    std::fs::write(images_dir.join("img.png"), "PNGDATA").unwrap();

    let processor = PDFProcessor::new_with_dir(temp.path().to_path_buf(), 1);
    let output = alesys_core::ingestion::mineru_wrapper::MinerUOutput {
        job_id: uuid::Uuid::new_v4(),
        markdown_path: md_path.clone(),
        images_dir: Some(images_dir),
        auto_dir,
        method: alesys_core::ingestion::models::ProcessingMethod::MinerU {
            gpu: true,
            model_version: "v1".to_string(),
        },
    };
    let book_root = temp.path().join("book");

    let start = Instant::now();
    let organizer = processor.organizer();
    let _ = organizer.reorganize(&output, &book_root);
    let elapsed = start.elapsed();
    println!("organizer_reorganize: {:?}", elapsed);
}

fn main() {
    bench_pdf_processor_creation();
    bench_organizer_reorganize();
}
