//! PyMuPDF Fallback Processor - CPU-only PDF processing when GPU unavailable

use crate::ingestion::models::{ImageRef, IngestionError, IngestionJob, IngestionResult, IngestionStats, ProcessingMethod, Chapter, IngestionMode};
use crate::ingestion::models::{BoundingBox, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

pub struct PyMuPDFFallback {
    tesseract_available: bool,
    tesseract_langs: Vec<String>,
}

impl PyMuPDFFallback {
    pub fn new() -> Self {
        Self {
            tesseract_available: Self::check_tesseract(),
            tesseract_langs: vec!["eng".to_string(), "spa".to_string()],
        }
    }

    fn check_tesseract() -> bool {
        std::process::Command::new("tesseract")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub async fn process(&self, job: &IngestionJob) -> Result<IngestionResult> {
        let start_time = std::time::Instant::now();
        
        info!("Processing PDF with PyMuPDF fallback: {}", job.pdf_path.display());

        // Extract text using pdfplumber via Python
        let pages = self.extract_text_pdfplumber(&job.pdf_path).await?;
        
        // Extract images using PyMuPDF
        let images = self.extract_images_pymupdf(&job.pdf_path, &job.pdf_path.parent().unwrap().join("images")).await?;
        
        // Extract tables
        let tables = self.extract_tables_pdfplumber(&job.pdf_path).await?;

        // Build chapters from text
        let chapters = self.build_chapters(&pages, &images);

        // Generate markdown
        let markdown = self.generate_markdown(&chapters, &images);

        let output_dir = job.pdf_path.parent().unwrap().join("processed");
        tokio::fs::create_dir_all(&output_dir).await?;
        
        let markdown_path = output_dir.join("document.md");
        tokio::fs::write(&markdown_path, markdown).await?;

        let stats = IngestionStats {
            pages_processed: pages.len() as u32,
            chars_extracted: pages.iter().map(|p| p.text.len() as u64).sum(),
            images_extracted: images.len() as u32,
            formulas_detected: 0, // PyMuPDF doesn't detect formulas well
            tables_detected: tables.len() as u32,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            method_used: ProcessingMethod::PyMuPDF { ocr_enabled: self.tesseract_available },
            database_chunks: None,
        };

        Ok(IngestionResult {
            job_id: job.id,
            success: true,
            mode: job.mode.clone(),
            output_dir: output_dir.clone(),
            markdown_path,
            images_dir: output_dir.join("images"),
            database_generated: false,
            database_path: None,
            chapters,
            images,
            citations: vec![],
            stats,
            warnings: vec!["PyMuPDF fallback used - formula detection limited".to_string()],
            error: None,
        })
    }

    async fn extract_text_pdfplumber(&self, pdf_path: &Path) -> Result<Vec<PageText>> {
        let script = r#"
import sys
import pdfplumber

pdf_path = sys.argv[1]
with pdfplumber.open(pdf_path) as pdf:
    for i, page in enumerate(pdf.pages):
        text = page.extract_text() or ""
        tables = page.extract_tables()
        print(f"---PAGE {i}---")
        print(text)
        print(f"---TABLES {len(tables)}---")
        for table in tables:
            for row in table:
                print("|".join(str(c) if c else "" for c in row))
"#;

        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(pdf_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IngestionError::FallbackFailed(format!("pdfplumber failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(self.parse_pdfplumber_output(&stdout))
    }

    pub(crate) fn parse_pdfplumber_output(&self, output: &str) -> Vec<PageText> {
        let mut pages = Vec::new();
        let mut current_page: Option<PageText> = None;
        let mut in_tables = false;

        for line in output.lines() {
            if line.starts_with("---PAGE ") {
                if let Some(page) = current_page.take() {
                    pages.push(page);
                }
                let page_num = line[8..line.len()-4].parse().unwrap_or(0);
                current_page = Some(PageText {
                    page_num,
                    text: String::new(),
                    tables: Vec::new(),
                });
                in_tables = false;
            } else if line.starts_with("---TABLES ") {
                in_tables = true;
            } else if let Some(page) = &mut current_page {
                if in_tables && line.contains('|') {
                    page.tables.push(line.to_string());
                } else if !in_tables {
                    page.text.push_str(line);
                    page.text.push('\n');
                }
            }
        }
        if let Some(page) = current_page {
            pages.push(page);
        }
        pages
    }

    async fn extract_images_pymupdf(&self, pdf_path: &Path, output_dir: &Path) -> Result<Vec<ImageRef>> {
        tokio::fs::create_dir_all(output_dir).await?;

        let output_dir_str = output_dir.to_string_lossy().to_string();
        let script = format!(r#"
import fitz
import sys

pdf_path = sys.argv[1]
output_dir = sys.argv[2]

doc = fitz.open(pdf_path)
for page_num in range(len(doc)):
    page = doc[page_num]
    images = page.get_images(full=True)
    for img_idx, img in enumerate(images):
        xref = img[0]
        pix = fitz.Pixmap(doc, xref)
        if pix.n < 5:
            pix.save(f"{{output_dir}}/page{{page_num}}_img{{img_idx}}.png")
        else:
            pix1 = fitz.Pixmap(fitz.csRGB, pix)
            pix1.save(f"{{output_dir}}/page{{page_num}}_img{{img_idx}}.png")
        print(f"EXTRACTED:page{{page_num}}_img{{img_idx}}.png")
"#);

        let output = Command::new("python3")
            .arg("-c")
            .arg(&script)
            .arg(pdf_path)
            .arg(&output_dir_str)
            .output()
            .await?;

        if !output.status.success() {
            warn!("PyMuPDF image extraction failed: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut images = Vec::new();
        
        for line in stdout.lines() {
            if line.starts_with("EXTRACTED:") {
                let filename = &line[10..];
                let path = output_dir.join(filename);
                if path.exists() {
                    images.push(ImageRef {
                        id: Uuid::new_v4(),
                        chapter_id: Uuid::nil(), // Will be set during chapter building
                        markdown_ref: format!("![image]({})", filename),
                        filesystem_path: path,
                        ocr_text: None,
                        is_formula: false,
                        bbox: None,
                    });
                }
            }
        }

        Ok(images)
    }

    async fn extract_tables_pdfplumber(&self, pdf_path: &Path) -> Result<Vec<Table>> {
        // Tables already extracted in extract_text_pdfplumber
        Ok(Vec::new())
    }

    pub(crate) fn build_chapters(&self, pages: &[PageText], images: &[ImageRef]) -> Vec<Chapter> {
        // Simple chapter detection: look for headers in text
        let mut chapters = Vec::new();
        let mut current_chapter: Option<Chapter> = None;
        
        for (i, page) in pages.iter().enumerate() {
            let lines: Vec<&str> = page.text.lines().collect();
            
            for line in lines {
                let trimmed = line.trim();
                // Detect headers (simple heuristic)
                if (trimmed.starts_with('#') || 
                    (trimmed.len() < 100 && trimmed.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c == ':')))
                    && !trimmed.is_empty() 
                {
                    if let Some(ch) = current_chapter.take() {
                        chapters.push(ch);
                    }
                    current_chapter = Some(Chapter {
                        id: Uuid::new_v4(),
                        title: trimmed.trim_start_matches('#').trim().to_string(),
                        level: trimmed.chars().take_while(|c| *c == '#').count() as u8,
                        start_page: i as u32 + 1,
                        end_page: i as u32 + 1,
                        markdown_path: PathBuf::new(),
                        image_refs: Vec::new(),
                    });
                }
            }
            
            if let Some(ch) = &mut current_chapter {
                ch.end_page = i as u32 + 1;
            }
        }
        
        if let Some(ch) = current_chapter {
            chapters.push(ch);
        }

        // If no chapters detected, create one
        if chapters.is_empty() {
            chapters.push(Chapter {
                id: Uuid::new_v4(),
                title: "Document".to_string(),
                level: 1,
                start_page: 1,
                end_page: pages.len() as u32,
                markdown_path: PathBuf::new(),
                image_refs: Vec::new(),
            });
        }

        chapters
    }

    pub(crate) fn generate_markdown(&self, chapters: &[Chapter], images: &[ImageRef]) -> String {
        let mut md = String::new();
        
        for ch in chapters {
            md.push_str(&format!("{} {}\n\n", "#".repeat(ch.level as usize), ch.title));
            md.push_str(&format!("*Pages {}-{}\n\n", ch.start_page, ch.end_page));
            
            // Add images for this chapter
            for img in images {
                if img.chapter_id == ch.id {
                    md.push_str(&format!("{}\n\n", img.markdown_ref));
                }
            }
        }
        
        md
    }
}

pub(crate) struct PageText {
    pub page_num: usize,
    pub text: String,
    pub tables: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Table {
    pub page: usize,
    pub rows: Vec<Vec<String>>,
}

impl Default for PyMuPDFFallback {
    fn default() -> Self {
        Self::new()
    }
}