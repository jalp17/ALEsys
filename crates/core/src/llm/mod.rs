//! Motor de inferencia LLM con soporte para múltiples backends
//!
//! Backends disponibles:
//! - **llama_cpp**: Vulkan GPU, 150+ arquitecturas, 23 quantizaciones (default)
//! - **mistralrs**: CPU, arquitecturas limitadas, sin MoE
//! - **candle**: Rust nativo, CUDA/Metal/CPU (experimental)
//! - **vllm**: Python subprocess, GPU de alto rendimiento
//! - **transformers**: Python subprocess, modelos HF
//!
//! Selección via variable de entorno `LLM_BACKEND=llama_cpp|mistralrs|candle|vllm|transformers|auto`

pub mod backend;
pub mod backend_manager;
pub mod config;

#[cfg(feature = "candle-backend")]
pub mod candle;
#[cfg(feature = "http-backend")]
pub mod http;
#[cfg(feature = "llama-cpp")]
pub mod llama_cpp;
#[cfg(feature = "mistralrs-backend")]
pub mod mistral;
#[cfg(feature = "transformers-backend")]
pub mod transformers;
#[cfg(feature = "vllm-backend")]
pub mod vllm;

pub use backend::LLMBackend;
pub use backend::LLMState;
pub use backend_manager::BackendManager;
pub use config::{Entity, KnowledgeExtraction, LLMBackendType, LLMConfig, Relation};
#[cfg(feature = "http-backend")]
pub use http::HttpLLMEngine;

use crate::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

/// Mensaje de chat
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Respuesta del LLM
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
}

/// Uso de tokens
#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Chunk de streaming
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
}

/// Trait que define la interfaz de un motor LLM
#[async_trait]
pub trait LLMEngine: Send + Sync {
    /// Chat con contexto de documentos
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse>;

    /// Chat con streaming real de tokens (cada chunk se envía individualmente)
    fn chat_stream<'a>(&'a self, messages: &'a [ChatMessage])
        -> BoxStream<'a, Result<StreamChunk>>;

    /// Generación de código (default: chat con system prompt de programación)
    async fn generate_code(&self, prompt: &str, language: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "Eres un asistente de programación. Genera código en {}.",
                    language
                ),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];
        let response = self.chat(&messages).await?;
        Ok(response.content)
    }

    /// Extracción de conocimiento (default: chat con system prompt de extracción)
    async fn extract_knowledge(&self, text: &str, schema: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!("Extrae conocimiento del texto según el esquema: {}", schema),
            },
            ChatMessage {
                role: "user".to_string(),
                content: text.to_string(),
            },
        ];
        let response = self.chat(&messages).await?;
        Ok(response.content)
    }

    /// Verifica si el backend está disponible
    fn is_available(&self) -> bool;

    /// Nombre del backend
    fn backend_name(&self) -> &str;
}

/// Helper: crea un BoxStream con un solo chunk desde un ChatResponse
pub fn single_chunk_stream(response: ChatResponse) -> BoxStream<'static, Result<StreamChunk>> {
    Box::pin(futures::stream::once(async move {
        Ok(StreamChunk {
            delta: response.content,
            finish_reason: Some("stop".to_string()),
        })
    }))
}

/// Helper: crea un BoxStream con un solo error
pub fn error_stream(err: crate::AlesysError) -> BoxStream<'static, Result<StreamChunk>> {
    Box::pin(futures::stream::once(async move { Err(err) }))
}

/// Embedder con ONNX Runtime (stub por ahora)
pub struct ONNXEmbedder {
    loaded: bool,
    dimension: usize,
}

impl ONNXEmbedder {
    pub fn new() -> Self {
        Self {
            loaded: false,
            dimension: 384,
        }
    }

    pub fn load(&mut self, model_path: &str) -> Result<()> {
        if !std::path::Path::new(model_path).exists() {
            tracing::warn!("Modelo ONNX no encontrado: {}", model_path);
            return Ok(());
        }
        tracing::info!("Modelo ONNX configurado: {}", model_path);
        self.loaded = true;
        Ok(())
    }

    pub fn encode(&self, text: &str) -> Result<Vec<f32>> {
        if !self.loaded {
            let mut embedding = vec![0.0f32; self.dimension];
            let hash = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
            for (i, val) in embedding.iter_mut().enumerate() {
                *val = ((hash.wrapping_mul(i as u32 + 1)) as f32) / (u32::MAX as f32);
            }
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for val in embedding.iter_mut() {
                    *val /= norm;
                }
            }
            return Ok(embedding);
        }
        Ok(vec![0.0; self.dimension])
    }

    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.encode(text)).collect()
    }

    pub fn is_available(&self) -> bool {
        self.loaded
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ONNXEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        assert_eq!(msg.role, "user");
    }

    #[test]
    fn test_usage() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_stream_chunk() {
        let chunk = StreamChunk {
            delta: "Hello".to_string(),
            finish_reason: None,
        };
        assert_eq!(chunk.delta, "Hello");
    }
}
