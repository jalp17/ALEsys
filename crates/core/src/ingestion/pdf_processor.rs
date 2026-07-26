//! PDFProcessor - Orchestrates PDF ingestion pipeline
//! Coordinates MinerU + PyMuPDF fallback + Organizer + Bibliography

use crate::ingestion::models::{IngestionJob, IngestionProgress, IngestionResult, IngestionStage, Result, IngestionError, IngestionMode};
use crate::ingestion::mineru_wrapper::MinerUWrapper;
use crate::ingestion::pymupdf_fallback::PyMuPDFFallback;
use crate::ingestion::organizer::Organizer;
use crate::ingestion::IngestionConfig;
use crate::bibliography::extractor::{CitationExtractor, BibliographyResult};
use crate::graphrag::ingestion_hook::{GraphRAGIngestionHook, IngestionHookConfig};
use crate::graphrag::GraphRAG;
use sqlx::PgPool;
use std::path::PathBuf;

pub struct PDFProcessor {
    mineru: MinerUWrapper,
    fallback: PyMuPDFFallback,
    organizer: Organizer,
    max_parallel: usize,
}

impl PDFProcessor {
    pub fn new(config: IngestionConfig) -> Self {
        Self {
            mineru: MinerUWrapper::new(config.model_dir, true),
            fallback: PyMuPDFFallback::new(),
            organizer: Organizer::new(config.output_base_dir.join("_reorg_logs")),
            max_parallel: config.max_parallel,
        }
    }

    pub fn new_with_dir(model_dir: PathBuf, max_parallel: usize) -> Self {
        Self {
            mineru: MinerUWrapper::new(model_dir, true),
            fallback: PyMuPDFFallback::new(),
            organizer: Organizer::new(PathBuf::from("/tmp")),
            max_parallel,
        }
    }

    pub async fn process(&self, job: IngestionJob) -> Result<IngestionResult> {
        let (progress_tx, _) = tokio::sync::mpsc::channel(32);
        self.process_with_progress(job, progress_tx).await
    }

    pub async fn process_with_progress(
        &self,
        job: IngestionJob,
        progress_tx: tokio::sync::mpsc::Sender<IngestionProgress>,
    ) -> Result<IngestionResult> {
        let job_id = job.id;
        
        // Check MinerU availability
        let mineru_info = match self.mineru.check_availability().await {
            Ok(info) => info,
            Err(e) => {
                let _ = progress_tx.send(IngestionProgress {
                    job_id: job.id,
                    stage: IngestionStage::Starting,
                    mode: job.mode.clone(),
                    message: format!("MinerU unavailable: {}", e),
                    progress_pct: 0.0,
                    current_page: None,
                    total_pages: None,
                    database_indexed: None,
                }).await;
                
                // Try fallback
                return self.fallback.process(&job).await
                    .map(|mut r| {
                        r.success = true;
                        r.error = None;
                        r.warnings.push("Used PyMuPDF fallback (MinerU unavailable)".to_string());
                        r
                    });
            }
        };

        // Use MinerU if available and GPU or not forced to fallback
        if !job.force_fallback && (mineru_info.gpu_available || true) {
            // Run MinerU
            match self.mineru.process(&job, progress_tx.clone()).await {
                Ok(mineru_output) => {
                    // Reorganize output
                    let book_root = job.pdf_path.parent().unwrap().join(format!("book_{}", job_id));
                    
                    let _ = progress_tx.send(IngestionProgress {
                        job_id,
                        stage: IngestionStage::OrganizingOutput,
                        mode: job.mode.clone(),
                        message: "Reorganizing output".to_string(),
                        progress_pct: 90.0,
                        current_page: None,
                        total_pages: None,
                        database_indexed: None,
                    }).await;

                    match self.organizer.reorganize(&mineru_output, &book_root).await {
                        Ok(org_output) => {
                            // Extract bibliography from markdown if available
                            let md_content = std::fs::read_to_string(&org_output.markdown_path).unwrap_or_default();
                            let extractor = CitationExtractor::new();
                            let bib_result = extractor.extract_from_text(&md_content, &org_output.markdown_path);
                            
                            Ok(IngestionResult {
                                job_id,
                                success: true,
                                mode: job.mode.clone(),
                                output_dir: book_root,
                                markdown_path: org_output.markdown_path,
                                images_dir: org_output.images_dir,
                                database_generated: false,
                                database_path: None,
                                chapters: vec![],
                                images: vec![],
                                citations: bib_result.citations,
                                stats: Default::default(),
                                warnings: vec![],
                                error: None,
                            })
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(e) => {
                    // MinerU failed, try fallback
                    let _ = progress_tx.send(IngestionProgress {
                        job_id,
                        stage: IngestionStage::RunningFallback,
                        message: format!("MinerU failed: {}, using fallback", e),
                        progress_pct: 0.0,
                        current_page: None,
                        total_pages: None,
                        database_indexed: None,
                        mode: job.mode.clone(),
                    }).await;
                    
                    self.fallback.process(&job).await
                }
            }
        } else {
            // Use fallback
            self.fallback.process(&job).await
        }
    }

    pub async fn process_with_progress_and_graphrag(
        &self,
        job: IngestionJob,
        progress_tx: tokio::sync::mpsc::Sender<IngestionProgress>,
        graphrag: Option<&GraphRAG>,
        pool: Option<&PgPool>,
    ) -> Result<IngestionResult> {
        let progress_tx_for_process = progress_tx.clone();
        let mut result = self.process_with_progress(job.clone(), progress_tx_for_process).await?;

        if let (Some(graphrag), Some(pool)) = (graphrag, pool) {
            let _ = progress_tx.send(IngestionProgress {
                job_id: job.id,
                stage: IngestionStage::IndexingGraphRAG,
                mode: job.mode.clone(),
                message: "Indexing in GraphRAG...".to_string(),
                progress_pct: 95.0,
                current_page: None,
                total_pages: None,
                database_indexed: Some(false),
            }).await;

            let hook = GraphRAGIngestionHook::new(IngestionHookConfig::default());
            match hook.index_documents(graphrag, &result, pool).await {
                Ok(index_result) => {
                    result.database_generated = true;
                    result.stats.database_chunks = Some(index_result.chunks_indexed as u32);
                    let _ = progress_tx.send(IngestionProgress {
                        job_id: job.id,
                        stage: IngestionStage::IndexingGraphRAG,
                        mode: job.mode.clone(),
                        message: format!("GraphRAG indexed: {} chunks", index_result.chunks_indexed),
                        progress_pct: 100.0,
                        current_page: None,
                        total_pages: None,
                        database_indexed: Some(true),
                    }).await;
                }
                Err(e) => {
                    tracing::warn!("GraphRAG indexing failed: {}", e);
                    result.warnings.push(format!("GraphRAG indexing failed: {}", e));
                }
            }
        }

        Ok(result)
    }

    pub async fn process_batch(
        &self,
        jobs: Vec<IngestionJob>,
        graphrag: Option<&GraphRAG>,
        pool: Option<&PgPool>,
    ) -> Result<Vec<IngestionResult>> {
        let mut results = Vec::new();
        for job in jobs {
            let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(32);
            let result = self
                .process_with_progress_and_graphrag(job, progress_tx, graphrag, pool)
                .await;
            if result.is_ok() {
                while let Some(_progress) = progress_rx.recv().await {}
            }
            results.push(result?);
        }
        Ok(results)
    }

    pub fn max_parallel(&self, jobs: usize) -> usize {
        std::cmp::min(jobs, self.max_parallel)
    }

    pub fn set_max_parallel(&mut self, max: usize) {
        self.max_parallel = max;
    }
}