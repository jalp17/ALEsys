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

const BATCH_SIZE: usize = 32;
const OVERLAP_TOKENS: usize = 200;
const MAX_CHAPTER_CHARS: usize = 5000;
const SPLIT_MAX_CHARS: usize = 2500;

#[derive(Debug, Clone)]
pub struct IngestionHookConfig {
    pub embedding_model: String,
    pub embedding_dimension: usize,
    pub max_chunk_tokens: usize,
}

impl Default for IngestionHookConfig {
    fn default() -> Self {
        Self {
            embedding_model: "small_embeddings".to_string(),
            embedding_dimension: 384,
            max_chunk_tokens: 512,
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
        embedder.load(&config.embedding_model);
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
                .map_err(|e| GraphRAGIngestionError::ReadChapter(e.to_string()))?;

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
                    };
                    let chunks = self.chunk_chapter(&sub_chapter, &sub_chunk.content, &topic, &result.output_dir)?;
                    all_chunks.extend(chunks);
                }
            } else {
                let chunks = self.chunk_chapter(chapter, &content, &topic, &result.output_dir)?;
                all_chunks.extend(chunks);
            }
        }

        let embedded = self.embed_chunks(&all_chunks).await?;
        self.insert_chunks(pool, &embedded, &topic).await?;

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
                    source_pdf: output_dir.to_string_lossy().to_string(),
                    chapter_title: chapter.title.clone(),
                    chapter_id: chapter.id,
                });

                let overlap_start = current_chunk.len().saturating_sub(OVERLAP_TOKENS * 4);
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
                source_pdf: output_dir.to_string_lossy().to_string(),
                chapter_title: chapter.title.clone(),
                chapter_id: chapter.id,
            });
        }

        Ok(chunks)
    }

    async fn embed_chunks(&self, chunks: &[Chunk]) -> Result<Vec<EmbeddedChunk>, GraphRAGIngestionError> {
        let mut embedded = Vec::new();

        for batch in chunks.chunks(BATCH_SIZE) {
            let texts: Vec<&str> = batch.iter().map(|c| c.text.as_str()).collect();
            let embeddings = self.embedder.encode_batch(&texts)
                .map_err(|e| GraphRAGIngestionError::Embedding(e.to_string()))?;

            for (chunk, embedding) in batch.iter().zip(embeddings) {
                embedded.push(EmbeddedChunk {
                    chunk: chunk.clone(),
                    embedding,
                });
            }
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
            let row = sqlx::query(
                r#"INSERT INTO documentos (ruta_relativa, tipo, topic, session_id, source_pdf, chapter_title)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (ruta_relativa) DO UPDATE SET topic = EXCLUDED.topic
                   RETURNING id"#,
            )
            .bind(format!("chunk:{}", embedded.chunk.chapter_id))
            .bind("ingested_chunk")
            .bind(topic)
            .bind(embedded.chunk.session_id)
            .bind(&embedded.chunk.source_pdf)
            .bind(&embedded.chunk.chapter_title)
            .fetch_one(pool)
            .await
            .map_err(|e| GraphRAGIngestionError::Database(e.to_string()))?;

            let doc_id: i32 = row.get("id");
            let embedding_str = format!(
                "[{}]",
                embedded.embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
            );

            sqlx::query(
                r#"INSERT INTO fragmentos (documento_id, contenido, embedding, indice_orden)
                   VALUES ($1, $2, $3::vector, $4)"#,
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
        text.split_whitespace().count()
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
