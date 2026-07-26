//! Organizer - Reorganizes MinerU output into clean book/chapter structure
//! Port of /mnt/src_file/reordenar_db_p.py

use crate::ingestion::mineru_wrapper::MinerUOutput;
use crate::ingestion::models::{IngestionError, Result};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;
use tracing::{info, warn};

pub struct Organizer {
    log_dir: PathBuf,
}

impl Organizer {
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_dir: log_dir.join("_reorg_logs"),
        }
    }

    pub async fn reorganize(&self, mineru_output: &MinerUOutput, book_root: &PathBuf) -> Result<OrganizedOutput> {
        let auto_dir = &mineru_output.auto_dir;
        let md_path = &mineru_output.markdown_path;

        // Ensure book root exists
        fs::create_dir_all(book_root)?;

        // Find markdown file (could be in auto_dir or parent)
        let md_path = if md_path.exists() {
            md_path.clone()
        } else {
            // Try to find it in auto_dir
            let candidates: Vec<_> = fs::read_dir(auto_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                .map(|e| e.path())
                .collect();
            
            if candidates.is_empty() {
                return Err(IngestionError::MinerUFailed(format!(
                    "No markdown found in: {}",
                    auto_dir.display()
                )));
            }
            candidates[0].clone()
        };

        // Extract image paths referenced in markdown
        let image_paths = self.extract_image_paths(&md_path)?;

        // Create images directory in book_root
        let images_dir = book_root.join("images");
        fs::create_dir_all(&images_dir)?;

        // Move referenced images
        let moved_images = self.move_referenced_images(auto_dir, &images_dir, &image_paths, &md_path)?;

        // Move markdown to root level
        let final_md = book_root.join(md_path.file_name().unwrap());
        if md_path.parent() != Some(book_root) {
            fs::rename(&md_path, &final_md)?;
        }

        // Clean up auto directory
        self.cleanup_auto_dir(auto_dir, book_root, &final_md)?;

        // Log the operation
        self.write_log(book_root, &md_path, &final_md, &images_dir, &moved_images)?;

        info!("✅ Reorganized output for job {}", mineru_output.job_id);

        Ok(OrganizedOutput {
            markdown_path: final_md,
            images_dir,
            images_moved: moved_images.len() as u32,
            cleaned_dirs: vec![auto_dir.clone()],
        })
    }

    pub(crate) fn extract_image_paths(&self, md_path: &PathBuf) -> Result<HashSet<PathBuf>> {
        let img_regex = Regex::new(r"!\[.*?\]\((.*?)\)").unwrap();
        let md_content = fs::read_to_string(md_path)?;
        
        let mut paths = HashSet::new();
        for line in md_content.lines() {
            for cap in img_regex.captures_iter(line) {
                if let Some(path_match) = cap.get(1) {
                    let path_str = path_match.as_str().trim().trim_matches('"').trim_matches('\'');
                    paths.insert(PathBuf::from(path_str));
                }
            }
        }
        
        Ok(paths)
    }

    fn move_referenced_images(
        &self,
        auto_dir: &PathBuf,
        images_dir: &PathBuf,
        image_paths: &HashSet<PathBuf>,
        md_path: &PathBuf,
    ) -> Result<Vec<PathBuf>> {
        let mut moved = Vec::new();
        
        for rel_path in image_paths {
            let src = auto_dir.join(rel_path);
            if !src.exists() {
                // Try looking in auto/images/ subdirectory
                let alt_src = auto_dir.join("images").join(rel_path);
                if !alt_src.exists() {
                    warn!("⚠️  Imagen referenciada no encontrada: {} (desde {})", 
                        src.display(), md_path.display());
                    continue;
                }
                
                let dst = images_dir.join(rel_path.file_name().unwrap());
                fs::rename(&alt_src, &dst)?;
                moved.push(dst);
            } else {
                let dst = images_dir.join(rel_path.file_name().unwrap());
                fs::rename(&src, &dst)?;
                moved.push(dst);
            }
        }
        
        Ok(moved)
    }

    fn cleanup_auto_dir(&self, auto_dir: &PathBuf, book_root: &PathBuf, keep_md: &PathBuf) -> Result<()> {
        for entry in fs::read_dir(auto_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip the markdown file we want to keep
            if path == *keep_md {
                continue;
            }
            
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
        
        // Remove the markdown file from auto_dir
        if keep_md.exists() && keep_md.parent() == Some(auto_dir) {
            fs::remove_file(keep_md)?;
        }
        
        // Remove the auto_dir itself
        if auto_dir.exists() {
            fs::remove_dir_all(auto_dir)?;
        }
        
        Ok(())
    }

    fn write_log(
        &self,
        book_root: &PathBuf,
        original_md: &PathBuf,
        final_md: &PathBuf,
        images_dir: &PathBuf,
        moved_images: &[PathBuf],
    ) -> Result<()> {
        fs::create_dir_all(&self.log_dir)?;
        
        let libro_name = book_root.file_name().unwrap().to_string_lossy();
        let log_path = self.log_dir.join(format!("{}_reorg.log", libro_name));
        
        let mut log_content = String::new();
        log_content.push_str("\n=== Bloque procesado ===\n");
        log_content.push_str(&format!("Libro (primer nivel): {}\n", book_root.display()));
        log_content.push_str(&format!("Markdown origen: {}\n", original_md.display()));
        log_content.push_str(&format!("Markdown final: {}\n", final_md.display()));
        log_content.push_str(&format!("Carpeta images creada: {}\n", images_dir.display()));
        log_content.push_str(&format!("Imágenes movidas: {}\n", moved_images.len()));
        for img in moved_images {
            log_content.push_str(&format!("   - {}\n", img.file_name().unwrap().display()));
        }
        log_content.push('\n');
        
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        file.write_all(log_content.as_bytes())?;
        
        Ok(())
    }
}

pub struct OrganizedOutput {
    pub markdown_path: PathBuf,
    pub images_dir: PathBuf,
    pub images_moved: u32,
    pub cleaned_dirs: Vec<PathBuf>,
}