use crate::ingestion::models::{IngestionError, IngestionJob, IngestionResult, IngestionProgress, IngestionStage, IngestionMode, ProcessingMethod, Result};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

const MINERU_TIMEOUT_HOURS: u64 = 20;
const MAGIC_PDF_CMD: &str = "magic-pdf";

pub struct MinerUWrapper {
    pub(crate) model_dir: PathBuf,
    pub(crate) use_gpu: bool,
    pub(crate) timeout: Duration,
}

impl MinerUWrapper {
    pub fn new(model_dir: PathBuf, use_gpu: bool) -> Self {
        Self {
            model_dir,
            use_gpu,
            timeout: Duration::from_secs(MINERU_TIMEOUT_HOURS * 3600),
        }
    }

    pub async fn check_availability(&self) -> Result<MinerUInfo> {
        // Check if magic-pdf is installed
        let output = Command::new(MAGIC_PDF_CMD)
            .arg("--version")
            .output()
            .await
            .map_err(|e| IngestionError::Config(format!("magic-pdf not found: {}", e)))?;

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Check GPU availability
        let gpu_available = self.check_gpu().await;
        
        // Check model directory
        let models_exist = self.check_models().await;

        Ok(MinerUInfo {
            version,
            gpu_available,
            models_exist,
            model_dir: self.model_dir.clone(),
        })
    }

    pub(crate) async fn check_gpu(&self) -> bool {
        if !self.use_gpu {
            return false;
        }
        
        // Check nvidia-smi
        let output = Command::new("nvidia-smi")
            .arg("--query-gpu=memory.total")
            .arg("--format=csv,noheader,nounits")
            .output()
            .await;
            
        output.is_ok() && !output.unwrap().stdout.is_empty()
    }

    pub(crate) async fn check_models(&self) -> bool {
        if !self.model_dir.exists() {
            return false;
        }
        
        // Check for required model directories
        let required = ["layout", "formula", "table"];
        for req in required {
            if !self.model_dir.join(req).exists() {
                return false;
            }
        }
        true
    }

    pub async fn process(&self, job: &IngestionJob, progress_tx: tokio::sync::mpsc::Sender<IngestionProgress>) -> Result<MinerUOutput> {
        let job_id = job.id;
        let pdf_path = &job.pdf_path;
        let output_dir = pdf_path.parent().unwrap().join(format!("mineru_output_{}", job_id));
        
        // Ensure output dir exists
        tokio::fs::create_dir_all(&output_dir).await?;
        
        // Send starting progress
        let _ = progress_tx.send(IngestionProgress {
            job_id,
            stage: IngestionStage::Starting,
            mode: job.mode.clone(),
            message: "Starting MinerU processing".to_string(),
            progress_pct: 0.0,
            current_page: None,
            total_pages: None,
            database_indexed: None,
        }).await;

        // Build command
        let mut cmd = Command::new(MAGIC_PDF_CMD);
        cmd.arg("-p").arg(pdf_path)
            .arg("-o").arg(&output_dir)
            .arg("--lang").arg(&job.ocr_languages.join(","))
            .arg("--model-dir").arg(&self.model_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        if self.use_gpu {
            cmd.arg("--device").arg("cuda");
        } else {
            cmd.arg("--device").arg("cpu");
        }

        if job.extract_formulas {
            cmd.arg("--formula").arg("true");
        }
        
        if job.extract_tables {
            cmd.arg("--table").arg("true");
        }

        info!("Running MinerU: {:?}", cmd);
        
        // Send GPU detection progress
        let _ = progress_tx.send(IngestionProgress {
            job_id,
            stage: IngestionStage::DetectingGpu,
            mode: IngestionMode::FilesOnly,
            message: if self.use_gpu { "GPU detected, using CUDA" } else { "Using CPU mode" }.to_string(),
            progress_pct: 5.0,
            current_page: None,
            total_pages: None,
            database_indexed: None,
        }).await;

        // Run with timeout
        let mut child = cmd.spawn()
            .map_err(|e| IngestionError::MinerUFailed(format!("Failed to spawn: {}", e)))?;

        // Stream output for progress
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
let progress_tx_clone = progress_tx.clone();
        let job_id_clone = job_id;
        
        // Spawn task to read stdout
        let stdout_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("MinerU stdout: {}", line);
                // Parse progress from output if available
                if let Some((stage, pct, msg)) = Self::parse_progress(&line) {
                    Self::send_progress(&progress_tx_clone, job_id_clone, stage, pct, msg).await;
                }
            }
        });

        let progress_tx_clone = progress_tx.clone();
        let job_id_clone = job_id;
        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.contains("ERROR") || line.contains("Error") {
                    error!("MinerU stderr: {}", line);
                } else {
                    debug!("MinerU stderr: {}", line);
                }
            }
        });

        // Wait for completion with timeout
        let result = timeout(self.timeout, child.wait()).await;

        stdout_task.abort();
        stderr_task.abort();

        match result {
            Ok(Ok(status)) => {
                if status.success() {
                    info!("MinerU completed successfully for job {}", job_id);
                    
                    Self::send_progress(&progress_tx, job_id, IngestionStage::Completed, 100.0, "MinerU processing completed".to_string()).await;

                    // Find the generated output
                    let output = self.find_output(&output_dir, &job_id).await?;
                    Ok(output)
                } else {
                    error!("MinerU failed with status: {}", status);
                    Err(IngestionError::MinerUFailed(format!("Exit code: {}", status.code().unwrap_or(-1))))
                }
            }
            Ok(Err(e)) => {
                error!("MinerU process error: {}", e);
                Err(IngestionError::MinerUFailed(e.to_string()))
            }
            Err(_) => {
                error!("MinerU timeout after {} hours", MINERU_TIMEOUT_HOURS);
                Err(IngestionError::Timeout(format!("MinerU timeout after {} hours", MINERU_TIMEOUT_HOURS)))
            }
        }
    }

    pub(crate) fn parse_progress(line: &str) -> Option<(IngestionStage, f32, String)> {
        // MinerU outputs progress in various formats
        // Try to parse common patterns
        if line.contains("Processing page") || line.contains("page ") {
            // Extract page numbers if possible
            return Some((
                IngestionStage::RunningMinerU,
                50.0,
                format!("Processing: {}", line.trim())
            ));
        }
        
        if line.contains("Layout analysis") {
            return Some((
                IngestionStage::RunningMinerU,
                30.0,
                "Layout analysis".to_string()
            ));
        }
        
        if line.contains("Formula detection") || line.contains("formula") {
            return Some((
                IngestionStage::RunningMinerU,
                40.0,
                "Formula detection".to_string()
            ));
        }
        
        if line.contains("Table detection") || line.contains("table") {
            return Some((
                IngestionStage::RunningMinerU,
                45.0,
                "Table detection".to_string()
            ));
        }
        
        if line.contains("OCR") || line.contains("ocr") {
            return Some((
                IngestionStage::RunningMinerU,
                55.0,
                "OCR processing".to_string()
            ));
        }
        
        if line.contains("Writing") || line.contains("output") {
            return Some((
                IngestionStage::RunningMinerU,
                70.0,
                "Writing output".to_string()
            ));
        }
        
        None
    }

    async fn send_progress(tx: &tokio::sync::mpsc::Sender<IngestionProgress>, job_id: Uuid, stage: IngestionStage, pct: f32, msg: String) {
        let _ = tx.send(IngestionProgress {
            job_id,
            stage,
            mode: IngestionMode::FilesOnly,
            message: msg,
            progress_pct: pct,
            current_page: None,
            total_pages: None,
            database_indexed: None,
        }).await;
    }

    async fn find_output(&self, base_dir: &PathBuf, job_id: &Uuid) -> Result<MinerUOutput> {
        // MinerU creates structure like: output_dir/auto/xxx.md, output_dir/auto/images/
        let auto_dir = base_dir.join("auto");
        
        if !auto_dir.exists() {
            return Err(IngestionError::MinerUFailed("No auto output directory found".to_string()));
        }

        // Find the .md file
        let mut md_files = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&auto_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                md_files.push(path);
            }
        }

        if md_files.is_empty() {
            return Err(IngestionError::MinerUFailed("No markdown output found".to_string()));
        }

        // Find images directory
        let images_dir = auto_dir.join("images");
        let images_exist = images_dir.exists();

        // Find model info
        let model_version = self.get_model_version().await;

        Ok(MinerUOutput {
            job_id: *job_id,
            markdown_path: md_files[0].clone(),
            images_dir: if images_exist { Some(images_dir) } else { None },
            auto_dir,
            method: ProcessingMethod::MinerU {
                gpu: self.use_gpu,
                model_version,
            },
        })
    }

    async fn get_model_version(&self) -> String {
        // Try to read version from model dir
        let version_file = self.model_dir.join("VERSION");
        if version_file.exists() {
            tokio::fs::read_to_string(version_file).await.unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct MinerUInfo {
    pub version: String,
    pub gpu_available: bool,
    pub models_exist: bool,
    pub model_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MinerUOutput {
    pub job_id: Uuid,
    pub markdown_path: PathBuf,
    pub images_dir: Option<PathBuf>,
    pub auto_dir: PathBuf,
    pub method: ProcessingMethod,
}