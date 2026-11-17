//! Markdown directory indexer.
//!
//! Indexes *already processed* content (MinerU-style `book/chapter/N.md` trees,
//! Obsidian vaults, any Markdown corpus) directly into GraphRAG — no PDF
//! re-processing, no GPU/MinerU required. Embeddings still go through the
//! configured embedder (HTTP llama.cpp by default).

use crate::graphrag::GraphRAG;
use crate::graphrag::ingestion_hook::{GraphRAGIngestionHook, IngestionHookConfig};
use crate::ingestion::models::Chapter;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// One discovered Markdown source file.
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    /// Book/vault name (top-level dir relative to root)
    pub topic: String,
    /// Chapter/note title (file stem)
    pub title: String,
    pub bytes: usize,
}

/// Scan a directory tree for Markdown files.
///
/// Layout-agnostic: any `*.md` under `root` is indexed. The `topic` is the
/// first path component relative to `root` (e.g. `goldstain_mecanica_clasica`
/// or `02-COMPUTACION`), so each book/vault-section becomes its own topic.
pub fn discover_markdown(root: &Path, max_files: usize) -> Result<Vec<DiscoveredFile>, String> {
    if !root.exists() {
        return Err(format!("Directorio no existe: {}", root.display()));
    }
    let mut out: Vec<DiscoveredFile> = Vec::new();
    walk(root, root, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    if out.len() > max_files {
        out.truncate(max_files);
    }
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<DiscoveredFile>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read_dir {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let topic = rel
                .components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .unwrap_or_else(|| "_root".to_string());
            let title = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string());
            let bytes = std::fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0);
            out.push(DiscoveredFile { path, topic, title, bytes });
        }
    }
    Ok(())
}

/// Result of indexing a directory.
#[derive(Debug, Clone, Default)]
pub struct MarkdownIndexResult {
    pub files_indexed: usize,
    pub chunks_indexed: usize,
    pub topics: Vec<String>,
    pub skipped_empty: usize,
    /// Non-fatal topic/file errors collected while indexing.
    pub errors: Vec<String>,
    /// True when indexing stopped early because `is_cancelled` fired.
    pub cancelled: bool,
}

/// Files indexed per `index_documents` call. Batching keeps embedding batches
/// bounded, makes progress incremental, and lets cancellation land quickly.
const CHAPTER_BATCH_SIZE: usize = 8;

/// Progress snapshot reported while indexing a Markdown corpus.
#[derive(Debug, Clone)]
pub struct MarkdownProgress {
    pub files_done: usize,
    pub files_total: usize,
    pub chunks_indexed: usize,
    pub current_file: Option<String>,
    pub current_topic: Option<String>,
}

/// Synchronous progress callback (`Send + Sync` so callers can push it into a channel).
pub type ProgressCallback<'a> = &'a (dyn Fn(MarkdownProgress) + Send + Sync);

/// Indexes discovered Markdown files into GraphRAG.
pub struct MarkdownIndexer {
    hook: GraphRAGIngestionHook,
    pool: PgPool,
}

impl MarkdownIndexer {
    pub fn new(pool: PgPool) -> Self {
        Self {
            hook: GraphRAGIngestionHook::new(IngestionHookConfig::from_env()),
            pool,
        }
    }

    /// Index without progress reporting.
    pub async fn index_files(
        &self,
        files: &[DiscoveredFile],
        graphrag: &GraphRAG,
        topic_override: Option<&str>,
    ) -> MarkdownIndexResult {
        self.index_files_with_progress(files, graphrag, topic_override, None, None)
            .await
    }

    /// Index reporting per-file progress through `on_progress`.
    /// If `is_cancelled` is provided and fires, indexing stops at the next batch boundary.
    pub async fn index_files_with_progress(
        &self,
        files: &[DiscoveredFile],
        graphrag: &GraphRAG,
        topic_override: Option<&str>,
        on_progress: Option<ProgressCallback<'_>>,
        is_cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> MarkdownIndexResult {
        let mut result = MarkdownIndexResult::default();
        let mut topics: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

        // Group by topic so each topic produces one IngestionResult (batch embed)
        let mut by_topic: std::collections::BTreeMap<String, Vec<&DiscoveredFile>> =
            std::collections::BTreeMap::new();
        for f in files {
            let t = topic_override.unwrap_or(&f.topic).to_string();
            by_topic.entry(t).or_default().push(f);
        }

        let files_total = files.len();
        let mut files_done: usize = 0;
        // Takes all values by argument so it never borrows `result`.
        let report = |files_done: usize, current_file: &str, topic: &str, chunks: usize| {
            if let Some(cb) = on_progress {
                cb(MarkdownProgress {
                    files_done,
                    files_total,
                    chunks_indexed: chunks,
                    current_file: Some(current_file.to_string()),
                    current_topic: Some(topic.to_string()),
                });
            }
        };

        for (topic, group) in by_topic {
            // Check cancellation before starting each topic
            if is_cancelled.is_some_and(|c| c.load(std::sync::atomic::Ordering::SeqCst)) {
                result.cancelled = true;
                result.errors.push("Cancelled before topic".to_string());
                break;
            }

            let mut chapters = Vec::new();
            for f in &group {
                let content = match std::fs::read_to_string(&f.path) {
                    Ok(c) => c,
                    Err(_) => {
                        files_done += 1;
                        let done = files_done;
                        let chunks = result.chunks_indexed;
                        report(done, &f.path.display().to_string(), &topic, chunks);
                        continue;
                    }
                };
                if content.trim().is_empty() {
                    result.skipped_empty += 1;
                    files_done += 1;
                    let done = files_done;
                    let chunks = result.chunks_indexed;
                    report(done, &f.path.display().to_string(), &topic, chunks);
                    continue;
                }
                // Use only the first heading as the chapter title if present
                let title = first_heading(&content).unwrap_or_else(|| f.title.clone());
                chapters.push(Chapter {
                    id: Uuid::new_v4(),
                    title,
                    level: 1,
                    start_page: 0,
                    end_page: 0,
                    markdown_path: f.path.clone(),
                    image_refs: vec![],
                    experiment_id: None,
                    trial_id: None,
                    area: None,
                    section: None,
                    source_type: Some("markdown".to_string()),
                });
                result.files_indexed += 1;
                files_done += 1;
                let done = files_done;
                let chunks = result.chunks_indexed;
                report(done, &f.path.display().to_string(), &topic, chunks);
            }

            if chapters.is_empty() {
                continue;
            }

            // Process chapters in batches of CHAPTER_BATCH_SIZE so cancellation
            // lands quickly and progress is incremental.
            for batch in chapters.chunks(CHAPTER_BATCH_SIZE) {
                if is_cancelled.is_some_and(|c| c.load(std::sync::atomic::Ordering::SeqCst)) {
                    result.cancelled = true;
                    result.errors.push("Cancelled mid-topic".to_string());
                    break;
                }

                let result_dir = batch.first().and_then(|c| c.markdown_path.parent())
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."));

                let ingestion_result = crate::ingestion::models::IngestionResult {
                    job_id: Uuid::new_v4(),
                    success: true,
                    mode: crate::ingestion::models::IngestionMode::Markdown,
                    output_dir: result_dir,
                    markdown_path: batch.first().map(|c| c.markdown_path.clone()).unwrap_or_default(),
                    images_dir: PathBuf::new(),
                    database_generated: false,
                    database_path: None,
                    chapters: batch.to_vec(),
                    images: vec![],
                    tables: vec![],
                    citations: vec![],
                    stats: Default::default(),
                    warnings: vec![],
                    error: None,
                };

                match self.hook.index_documents(graphrag, &ingestion_result, &self.pool).await {
                    Ok(idx) => result.chunks_indexed += idx.chunks_indexed,
                    Err(e) => {
                        let msg = format!("Markdown index error (topic={}): {}", topic, e);
                        tracing::error!("{}", msg);
                        result.errors.push(msg);
                    }
                }

                let done = files_done;
                let chunks = result.chunks_indexed;
                report(done, &format!("<topic {}>", topic), &topic, chunks);
            }

            if result.cancelled {
                break;
            }
            topics.insert(topic);
        }

        result.topics = topics.into_iter().collect();
        result
    }
}

fn groups_markdown(group: &[&DiscoveredFile]) -> PathBuf {
    group.first().map(|f| f.path.clone()).unwrap_or_else(|| PathBuf::from("."))
}

fn first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("# ") {
            let r = rest.trim();
            if !r.is_empty() {
                return Some(r.to_string());
            }
        }
    }
    None
}