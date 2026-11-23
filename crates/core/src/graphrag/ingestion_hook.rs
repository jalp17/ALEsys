//! GraphRAG Ingestion Hook
//!
//! Post-ingestión hook: indexa chunks de PDF automáticamente en GraphRAG.
//!
//! Estrategia:
//! - Chunking: por sección (capítulo) + overlap 200 tokens
//! - Si capítulo > 5000 chars, aplicar KnowledgeCuration splitter (max 2500 chars)
//! - Embeddings: ONNX Runtime batch 32
//! - Metadatos: topic, session_id, source_pdf, chapter_title

use crate::graphrag::GraphRAG;
use crate::ingestion::models::{Chapter, IngestionResult};
use crate::knowledge_curation::splitter::{DocumentSplitter, SplitStrategy};
use crate::llm::ONNXEmbedder;
use sqlx::PgPool;
use sqlx::Row;
use std::path::PathBuf;

const BATCH_SIZE: usize = 8;
const OVERLAP_TOKENS: usize = 200;
const MAX_CHAPTER_CHARS: usize = 2000;
const SPLIT_MAX_CHARS: usize = 800;

#[derive(Debug, Clone)]
pub struct IngestionHookConfig {
    pub embedding_model: String,
    pub embedding_dimension: usize,
    pub max_chunk_tokens: usize,
    /// HTTP embedder endpoint (llama.cpp / OpenAI-compatible). When set, the
    /// hook uses it instead of a local ONNX model.
    pub http_embed_url: Option<String>,
}

impl Default for IngestionHookConfig {
    fn default() -> Self {
        Self {
            embedding_model: "small_embeddings".to_string(),
            embedding_dimension: 1024,
            max_chunk_tokens: 512,
            http_embed_url: None,
        }
    }
}

impl IngestionHookConfig {
    /// Build config from environment (EMBEDDING_HTTP_URL / EMBEDDING_HTTP_DIM).
    pub fn from_env() -> Self {
        let http = std::env::var("EMBEDDING_HTTP_URL").ok();
        let dim: usize = std::env::var("EMBEDDING_HTTP_DIM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1024);
        Self {
            embedding_model: "http".to_string(),
            embedding_dimension: dim,
            max_chunk_tokens: 512,
            http_embed_url: http,
        }
    }
}

pub struct GraphRAGIngestionHook {
    config: IngestionHookConfig,
    embedder: ONNXEmbedder,
}

impl GraphRAGIngestionHook {
    pub fn new(config: IngestionHookConfig) -> Self {
        let mut embedder = ONNXEmbedder::new();
        if let Some(url) = config.http_embed_url.clone() {
            embedder.set_http(url, config.embedding_dimension);
        } else {
            embedder.load(&config.embedding_model);
        }
        Self { config, embedder }
    }

    pub async fn index_documents(
        &self,
        graphrag: &crate::graphrag::GraphRAG,
        result: &IngestionResult,
        pool: &PgPool,
    ) -> Result<IndexResult, GraphRAGIngestionError> {
        if result.chapters.is_empty() {
            return Ok(IndexResult::empty());
        }

        let mut all_chunks: Vec<Chunk> = Vec::new();
        let topic = result.output_dir.to_string_lossy().to_string();
        let splitter = DocumentSplitter::new(SplitStrategy::BySize { max_chars: SPLIT_MAX_CHARS });

        for chapter in &result.chapters {
            let content = std::fs::read_to_string(&chapter.markdown_path)
                .map_err(|e| GraphRAGIngestionError::ReadChapter(e.to_string()))?
                .replace('\0', "");

            if content.len() > MAX_CHAPTER_CHARS {
                let split_result = splitter.split(&chapter.id.to_string(), &content);
                for (i, sub_chunk) in split_result.chunks.iter().enumerate() {
                    let sub_chapter = Chapter {
                        id: uuid::Uuid::new_v4(),
                        title: format!("{} (part {})", chapter.title, i + 1),
                        level: chapter.level,
                        start_page: chapter.start_page,
                        end_page: chapter.end_page,
                        markdown_path: chapter.markdown_path.clone(),
                        image_refs: chapter.image_refs.clone(),
                        experiment_id: chapter.experiment_id.clone(),
                        trial_id: chapter.trial_id.clone(),
                        area: chapter.area.clone(),
                        section: chapter.section.clone(),
                        source_type: chapter.source_type.clone(),
                    };
                    let chunks = self.chunk_chapter(&sub_chapter, &sub_chunk.content, &topic, &result.output_dir)?;
                    all_chunks.extend(chunks);
                }
            } else {
                let chunks = self.chunk_chapter(chapter, &content, &topic, &result.output_dir)?;
                all_chunks.extend(chunks);
            }
        }

        if all_chunks.is_empty() {
            return Ok(IndexResult::empty());
        }

#[cfg(feature = "tracking")]
use crate::tracking::{MlflowClient, TrackingConfig};

        let embedded = self.embed_chunks(&all_chunks).await?;
        self.insert_chunks(pool, &embedded, &topic).await?;

        // Track in MLflow if enabled
        #[cfg(feature = "tracking")]
        {
            let tracking = TrackingConfig::from_env();
            if tracking.is_enabled() {
                let client = MlflowClient::new(tracking);
                if let Ok(run_id) = client.create_run("ingestion").await {
                    let _ = client.log_param(&run_id, "topic", &topic).await;
                    let _ = client.log_param(&run_id, "chapters", &result.chapters.len().to_string()).await;
                    let _ = client.log_metric(&run_id, "chunks_indexed", all_chunks.len() as f64).await;
                    let _ = client.end_run(&run_id).await;
                }
            }
        }

        Ok(IndexResult {
            chunks_indexed: all_chunks.len(),
            chapters_processed: result.chapters.len(),
        })
    }

    fn chunk_chapter(
        &self,
        chapter: &Chapter,
        content: &str,
        topic: &str,
        output_dir: &PathBuf,
    ) -> Result<Vec<Chunk>, GraphRAGIngestionError> {
        let mut chunks = Vec::new();
        let sections: Vec<&str> = content.split('\n').collect();

        let mut current_chunk = String::new();
        let mut current_tokens = 0usize;

        for section in sections {
            let section_tokens = self.estimate_tokens(section);

            if current_tokens + section_tokens > self.config.max_chunk_tokens && !current_chunk.is_empty() {
                chunks.push(Chunk {
                    text: current_chunk.clone(),
                    topic: topic.to_string(),
                    session_id: None,
                    source_pdf: chapter.markdown_path.to_string_lossy().to_string(),
                    chapter_title: chapter.title.clone(),
                    chapter_id: chapter.id,
                    experiment_id: chapter.experiment_id.clone(),
                    trial_id: chapter.trial_id.clone(),
                    area: chapter.area.clone(),
                    section: chapter.section.clone(),
                    source_type: chapter.source_type.clone(),
                    data_type: None,
                });

                let overlap_start = current_chunk.len().saturating_sub(OVERLAP_TOKENS * 4);
                let overlap_start = {
                    let mut p = overlap_start.min(current_chunk.len());
                    while p > 0 && !current_chunk.is_char_boundary(p) {
                        p -= 1;
                    }
                    p
                };
                current_chunk = current_chunk[overlap_start..].to_string();
                current_tokens = self.estimate_tokens(&current_chunk);
            }

            current_chunk.push_str(section);
            current_chunk.push('\n');
            current_tokens += section_tokens;
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk {
                text: current_chunk,
                topic: topic.to_string(),
                session_id: None,
                source_pdf: chapter.markdown_path.to_string_lossy().to_string(),
                chapter_title: chapter.title.clone(),
                chapter_id: chapter.id,
                experiment_id: chapter.experiment_id.clone(),
                trial_id: chapter.trial_id.clone(),
                area: chapter.area.clone(),
                section: chapter.section.clone(),
                source_type: chapter.source_type.clone(),
                data_type: None,
            });
        }

        Ok(chunks)
    }

    async fn embed_chunks(&self, chunks: &[Chunk]) -> Result<Vec<EmbeddedChunk>, GraphRAGIngestionError> {
        let mut embedded = Vec::new();

        // Safety: truncate any chunk that exceeds safe token limit (900 tokens ≈ 2700 chars)
        // to prevent llama.cpp from rejecting the entire batch
        let safe_chunks: Vec<Chunk> = chunks.iter().map(|c| {
            if self.estimate_tokens(&c.text) > 900 {
                let safe_chars = 900 * 3; // ~2700 chars max
                let mut truncated = c.text.clone();
                if truncated.len() > safe_chars {
                    // Find char boundary
                    let mut end = safe_chars;
                    while end > 0 && !truncated.is_char_boundary(end) {
                        end -= 1;
                    }
                    truncated.truncate(end);
                    let mut chunk = c.clone();
                    chunk.text = truncated;
                    chunk
                } else {
                    c.clone()
                }
            } else {
                c.clone()
            }
        }).collect();

        for batch in safe_chunks.chunks(BATCH_SIZE) {
            let texts: Vec<&str> = batch.iter().map(|c| c.text.as_str()).collect();

            // Reintentar hasta obtener embeddings validos para TODOS los chunks del batch.
            // No descartar silenciosamente: si un embedding es invalido, reintentar el batch completo.
            let mut attempt = 0u32;
            let max_attempts = 5u32;
            let valid_embeddings = loop {
                attempt += 1;
                let embeddings = match self.embedder.encode_batch(&texts) {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::warn!("Embedding batch error (intento {}): {}", attempt, e);
                        if attempt >= max_attempts {
                            return Err(GraphRAGIngestionError::Embedding(format!(
                                "Embedder no disponible tras {} intentos: {}", attempt, e
                            )));
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
                        continue;
                    }
                };

                if embeddings.len() != batch.len() {
                    tracing::warn!("Mismatch: {} embeddings para {} chunks (intento {})", embeddings.len(), batch.len(), attempt);
                    if attempt >= max_attempts {
                        return Err(GraphRAGIngestionError::Embedding(format!(
                            "Mismatch de embeddings tras {} intentos", attempt
                        )));
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
                    continue;
                }

                // Validar cada embedding individualmente
                let mut all_valid = true;
                let mut invalid_reasons = Vec::new();
                for (chunk, embedding) in batch.iter().zip(embeddings.iter()) {
                    if embedding.is_empty() {
                        all_valid = false;
                        invalid_reasons.push(format!("vacio: {}", chunk.chapter_title));
                        continue;
                    }
                    if embedding.len() != self.config.embedding_dimension {
                        all_valid = false;
                        invalid_reasons.push(format!("dim {} (esperado {}): {}", embedding.len(), self.config.embedding_dimension, chunk.chapter_title));
                        continue;
                    }
                    if embedding.iter().all(|&x| x == 0.0) {
                        all_valid = false;
                        invalid_reasons.push(format!("all-zero: {}", chunk.chapter_title));
                        continue;
                    }
                    if embedding.iter().any(|x| x.is_nan() || x.is_infinite()) {
                        all_valid = false;
                        invalid_reasons.push(format!("NaN/Inf: {}", chunk.chapter_title));
                        continue;
                    }
                    let mean: f32 = embedding.iter().sum::<f32>() / embedding.len() as f32;
                    let variance: f32 = embedding.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / embedding.len() as f32;
                    if variance < 1e-8 {
                        all_valid = false;
                        invalid_reasons.push(format!("varianza baja: {}", chunk.chapter_title));
                        continue;
                    }
                }

                if all_valid {
                    break embeddings;
                }

                // Algunos embeddings invalidos: reintentar el batch completo
                tracing::warn!("Embeddings invalidos (intento {}): {:?}", attempt, invalid_reasons);
                if attempt >= max_attempts {
                    return Err(GraphRAGIngestionError::Embedding(format!(
                        "Embeddings invalidos tras {} intentos: {:?}", attempt, invalid_reasons
                    )));
                }
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            };

            for (chunk, embedding) in batch.iter().zip(valid_embeddings) {
                embedded.push(EmbeddedChunk {
                    chunk: chunk.clone(),
                    embedding,
                });
            }
        }

        if embedded.is_empty() && !chunks.is_empty() {
            return Err(GraphRAGIngestionError::Embedding(
                "Todos los embeddings fueron invalidos".to_string()
            ));
        }

        Ok(embedded)
    }

    async fn insert_chunks(
        &self,
        pool: &PgPool,
        chunks: &[EmbeddedChunk],
        topic: &str,
    ) -> Result<(), GraphRAGIngestionError> {
        for embedded in chunks {
            // Compute SHA-256 hash of chunk content for change detection
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(embedded.chunk.text.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            let row = sqlx::query(
                r#"INSERT INTO documentos (ruta_relativa, tipo, topic, session_id, source_pdf, chapter_title, experiment_id, trial_id, area, section, source_type, data_type, frontmatter, contenido_hash)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, '{}'::jsonb, $13)
                   ON CONFLICT (ruta_relativa) DO UPDATE SET topic = EXCLUDED.topic, contenido_hash = EXCLUDED.contenido_hash
                   RETURNING id"#,
            )
            .bind(format!("chunk:{}", embedded.chunk.chapter_id))
            .bind("ingested_chunk")
            .bind(topic)
            .bind(embedded.chunk.session_id)
            .bind(&embedded.chunk.source_pdf)
            .bind(&embedded.chunk.chapter_title)
            .bind(&embedded.chunk.experiment_id)
            .bind(&embedded.chunk.trial_id)
            .bind(&embedded.chunk.area)
            .bind(&embedded.chunk.section)
            .bind(&embedded.chunk.source_type)
            .bind(&embedded.chunk.data_type)
            .bind(&hash)
            .fetch_one(pool)
            .await
            .map_err(|e| GraphRAGIngestionError::Database(e.to_string()))?;

            let doc_id: i32 = row.get("id");
            let embedding_str = format!(
                "[{}]",
                embedded.embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
            );

            sqlx::query(
                r#"INSERT INTO fragmentos (documento_id, contenido, embedding, indice_orden, embedding_source)
                   VALUES ($1, $2, $3::vector, $4, 'http')"#,
            )
            .bind(doc_id)
            .bind(&embedded.chunk.text)
            .bind(embedding_str)
            .bind(0i32)
            .execute(pool)
            .await
            .map_err(|e| GraphRAGIngestionError::Database(e.to_string()))?;
        }

        Ok(())
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Conservative estimate: 1 token ≈ 3 chars (accounts for code/formulas with high token density)
        (text.len() + 2) / 3
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub text: String,
    pub topic: String,
    pub session_id: Option<Uuid>,
    pub source_pdf: String,
    pub chapter_title: String,
    pub chapter_id: Uuid,
    // Experimental metadata (Fase I+D)
    pub experiment_id: Option<String>,
    pub trial_id: Option<String>,
    pub area: Option<String>,
    pub section: Option<String>,
    pub source_type: Option<String>,
    pub data_type: Option<String>,
}

#[derive(Debug)]
struct EmbeddedChunk {
    chunk: Chunk,
    embedding: Vec<f32>,
}

#[derive(Debug)]
pub struct IndexResult {
    pub chunks_indexed: usize,
    pub chapters_processed: usize,
}

impl IndexResult {
    pub fn empty() -> Self {
        Self {
            chunks_indexed: 0,
            chapters_processed: 0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GraphRAGIngestionError {
    #[error("Failed to read chapter: {0}")]
    ReadChapter(String),
    #[error("Embedding error: {0}")]
    Embedding(String),
    #[error("Database error: {0}")]
    Database(String),
}

use uuid::Uuid;
