//! Ingestion Plugin for Fase 11 Plugin System

use crate::ingestion::{IngestionConfig, IngestionJob, IngestionProgress, IngestionStage, PDFProcessor};
use crate::plugin::{Plugin, PluginContext, PluginMetadata, PluginPermission, PluginResult};
use crate::bibliography::Citation;
use crate::ingestion::models::IngestionMode;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

pub struct IngestionPlugin {
    pub(crate) processor: Option<PDFProcessor>,
    pub(crate) model_dir: PathBuf,
    pub(crate) max_parallel: usize,
}

impl IngestionPlugin {
    pub fn new(config: IngestionConfig) -> Self {
        Self {
            processor: None,
            model_dir: config.model_dir,
            max_parallel: config.max_parallel,
        }
    }
}

impl Plugin for IngestionPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "ingestion".to_string(),
            name: "PDF Ingestion Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "ALEsys".to_string(),
            description: "Extracts text and images from PDF documents".to_string(),
            min_alesys_version: "0.1.0".to_string(),
            permissions: vec![
                PluginPermission::FilesystemRead { allowed_paths: vec!["/tmp".to_string(), "/data".to_string()] },
                PluginPermission::FilesystemWrite { allowed_paths: vec!["/tmp".to_string()] },
                PluginPermission::Execute { allowed_commands: vec!["python3".to_string(), "magic-pdf".to_string()] },
            ],
            hooks: vec!["ingest.pdf".to_string(), "ingest.batch".to_string()],
        }
    }

    fn init(&mut self, context: &PluginContext) -> Result<(), String> {
        info!("Initializing IngestionPlugin for request {}", context.request_id);
        
        // Verify Python environment
        let output = std::process::Command::new("python3")
            .arg("--version")
            .output()
            .map_err(|e| format!("Python not found: {}", e))?;
            
        if !output.status.success() {
            return Err("Python3 not available".to_string());
        }

        // Check for magic-pdf (MinerU)
        let output = std::process::Command::new("magic-pdf")
            .arg("--version")
            .output();
            
        match output {
            Ok(o) if o.status.success() => {
                info!("MinerU (magic-pdf) available")
            }
            _ => {
                warn!("MinerU not available, will use PyMuPDF fallback")
            }
        }

        self.processor = Some(PDFProcessor::new_with_dir(self.model_dir.clone(), self.max_parallel));
        Ok(())
    }

    fn execute(&self, command: &str, args: &[String], context: &PluginContext)
        -> Result<PluginResult, String> {
        let processor = self.processor.as_ref().ok_or_else(|| "Plugin not initialized".to_string())?;
        
        match command {
            "ingest.pdf" => {
                let pdf_path = args.get(0).map(|s| PathBuf::from(s))
                    .ok_or_else(|| "Missing pdf_path".to_string())?;
                let topic = args.get(1).cloned().unwrap_or_else(|| "uncategorized".to_string());
                
                let job = IngestionJob {
                    id: uuid::Uuid::new_v4(),
                    pdf_path,
                    topic,
                    ..Default::default()
                };

                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
                
                match runtime.block_on(processor.process(job)) {
                    Ok(result) => {
                        let mut metadata = HashMap::new();
                        metadata.insert("output_dir".to_string(), result.output_dir.to_string_lossy().into_owned());
                        metadata.insert("markdown_path".to_string(), result.markdown_path.to_string_lossy().into_owned());
                        metadata.insert("citations_count".to_string(), result.citations.len().to_string());
                        
                        Ok(PluginResult {
                            success: result.success,
                            output: Some(result.output_dir.to_string_lossy().into_owned()),
                            error: result.error,
                            metadata,
                        })
                    }
                    Err(e) => Err(e.to_string())
                }
            }
            "ingest.batch" => Err("Batch ingestion not yet implemented".to_string()),
            _ => Err(format!("Unknown command: {}", command))
        }
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.processor = None;
        Ok(())
    }

    fn can_handle(&self, command: &str) -> bool {
        matches!(command, "ingest.pdf" | "ingest.batch")
    }

    fn supported_commands(&self) -> Vec<String> {
        vec!["ingest.pdf".to_string(), "ingest.batch".to_string()]
    }
}