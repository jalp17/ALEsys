//! CLI Ingestion Commands

use alesys_core::ingestion::{IngestionConfig, IngestionJob, IngestionMode, PDFProcessor};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Args)]
pub struct IngestionArgs {
    #[command(subcommand)]
    pub command: IngestionCommand,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ModeArg {
    Full,
    FilesOnly,
}

#[derive(Subcommand)]
pub enum IngestionCommand {
    /// Ingest a single PDF file
    Pdf {
        /// Path to PDF file
        pdf_path: PathBuf,
        
        /// Topic/category for the document
        #[arg(short, long, default_value = "uncategorized")]
        topic: String,
        
        /// Processing mode: Full (with DB) or FilesOnly (Colab-compatible)
        #[arg(long, default_value = "files_only")]
        mode: ModeArg,
        
        /// Force CPU fallback (skip GPU)
        #[arg(long, default_value_t = false)]
        force_fallback: bool,
        
        /// OCR languages (comma-separated)
        #[arg(long, default_value = "en,es")]
        ocr_languages: String,
        
        /// Extract formulas
        #[arg(long, default_value_t = true)]
        formulas: bool,
        
        /// Extract tables
        #[arg(long, default_value_t = true)]
        tables: bool,
    },
    
    /// Ingest multiple PDF files in batch
    Batch {
        /// Directory containing PDFs (recursive)
        directory: PathBuf,
        
        /// Topic for all documents
        #[arg(short, long, default_value = "uncategorized")]
        topic: String,
        
        /// Processing mode: Full (with DB) or FilesOnly (Colab-compatible)
        #[arg(long, default_value = "files_only")]
        mode: ModeArg,
        
        /// Max parallel jobs
        #[arg(short, long, default_value_t = 1)]
        parallel: usize,
        
        /// File pattern (glob) - default: *.pdf
        #[arg(long, default_value = "*.pdf")]
        pattern: String,
    },
    
    /// Show ingestion status
    Status {
        /// Job ID to check (optional - shows all active if not provided)
        #[arg(long)]
        job_id: Option<String>,
    },
}

pub async fn handle_ingestion(args: IngestionArgs) -> Result<()> {
    let config = IngestionConfig::default();
    
    match args.command {
        IngestionCommand::Pdf { 
            pdf_path, 
            topic, 
            mode, 
            force_fallback, 
            ocr_languages, 
            formulas, 
            tables 
        } => {
            let job = IngestionJob {
                pdf_path,
                topic,
                mode: match mode {
                    ModeArg::Full => IngestionMode::Full,
                    ModeArg::FilesOnly => IngestionMode::FilesOnly,
                },
                force_fallback,
                ocr_languages: ocr_languages.split(',').map(|s| s.to_string()).collect(),
                extract_formulas: formulas,
                extract_tables: tables,
                ..Default::default()
            };

            let processor = PDFProcessor::new_with_dir(config.model_dir.clone(), config.max_parallel);
            let result = processor.process(job).await?;
            
            if result.success {
                println!("✅ Ingestion successful!");
                println!("  Mode: {:?}", result.mode);
                println!("  Output: {}", result.output_dir.display());
                println!("  Markdown: {}", result.markdown_path.display());
                println!("  Images: {}", result.images_dir.display());
                if result.database_generated {
                    println!("  Database: {}", result.database_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default());
                }
            } else {
                eprintln!("❌ Ingestion failed: {:?}", result.error);
            }
        }
        
        IngestionCommand::Batch { 
            directory, 
            topic, 
            mode, 
            parallel, 
            pattern: _pattern 
        } => {
            // Find all PDFs using WalkDir
            let entries: Vec<_> = WalkDir::new(&directory)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "pdf").unwrap_or(false))
                .collect();

            println!("📁 Found {} PDF files in {}", entries.len(), directory.display());

            let job_mode = match mode {
                ModeArg::Full => IngestionMode::Full,
                ModeArg::FilesOnly => IngestionMode::FilesOnly,
            };
            
            let jobs: Vec<IngestionJob> = entries.into_iter().map(|e| IngestionJob {
                pdf_path: e.path().to_path_buf(),
                topic: topic.clone(),
                mode: job_mode.clone(),
                ..Default::default()
            }).collect();

            let processor = PDFProcessor::new_with_dir(config.model_dir.clone(), parallel);
            let results = processor.process_batch(jobs, None, None).await?;

            let successful = results.iter().filter(|r| r.success).count();
            let failed = results.iter().filter(|r| !r.success).count();

            println!("✅ Batch complete: {} successful, {} failed", successful, failed);
            for r in &results {
                if !r.success {
                    eprintln!("  ❌ {} failed", r.output_dir.display());
                }
            }
        }
        
        IngestionCommand::Status { job_id: _ } => {
            println!("📊 Ingestion status (placeholder - implement with API call)");
        }
    }

    Ok(())
}