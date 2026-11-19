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
    http_url: Option<String>,
}

impl ONNXEmbedder {
    pub fn new() -> Self {
        Self {
            loaded: false,
            dimension: 384,
            http_url: None,
        }
    }

    pub fn with_http(url: String, dimension: usize) -> Self {
        Self {
            loaded: true,
            dimension,
            http_url: Some(url),
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

    pub fn set_http(&mut self, url: String, dimension: usize) {
        self.http_url = Some(url);
        self.dimension = dimension;
        self.loaded = true;
    }

    pub fn encode(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(url) = &self.http_url {
            let client = reqwest::Client::new();
            let body = serde_json::json!({"input": text, "model": "bge"});
            let url = format!("{}/v1/embeddings", url.trim_end_matches('/'));
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    for attempt in 0..5u32 {
                        if attempt > 0 {
                            let delay = std::time::Duration::from_secs(2u64.pow(attempt));
                            tokio::time::sleep(delay).await;
                        }
                        match client.post(&url).json(&body).send().await {
                            Ok(resp) => {
                                if let Ok(json) = resp.json::<serde_json::Value>().await {
                                    if let Some(emb) = json.get("data").and_then(|d| d.get(0)).and_then(|e| e.get("embedding")) {
                                        if let Some(arr) = emb.as_array() {
                                            let emb: Vec<f32> = arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect();
                                            if !emb.is_empty() {
                                                return Ok(emb);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    Err(crate::AlesysError::LLM("Embedder no disponible tras 5 intentos".to_string()))
                })
            });
            if let Ok(emb) = result {
                if !emb.is_empty() {
                    return Ok(emb);
                }
            }
        }
        // NO generar embeddings falsos. Si el embedder no esta disponible,
        // retornar error para que el llamador pueda reintentar o esperar.
        Err(crate::AlesysError::LLM(format!(
            "Embedder no disponible (loaded={}, http_url={:?}). Inicie el servicio de embeddings.",
            self.loaded, self.http_url
        )))
    }

    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // True batch: single HTTP request with array input (llama.cpp / OpenAI-compatible).
        // ~7x faster than one-request-per-text for large corpora.
        if let Some(url) = &self.http_url {
            if !texts.is_empty() {
                let client = reqwest::Client::new();
                let body = serde_json::json!({"input": texts, "model": "bge"});
                let url = format!("{}/v1/embeddings", url.trim_end_matches('/'));
                let owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
                // Retry con backoff exponencial si el embedder no responde
                let mut last_err = None;
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        for attempt in 0..5u32 {
                            if attempt > 0 {
                                let delay = std::time::Duration::from_secs(2u64.pow(attempt));
                                tokio::time::sleep(delay).await;
                            }
                            match client.post(&url).json(&body).send().await {
                                Ok(resp) => {
                                    match resp.json::<serde_json::Value>().await {
                                        Ok(json) => return Ok::<_, crate::AlesysError>(json),
                                        Err(e) => {
                                            last_err = Some(crate::AlesysError::LLM(e.to_string()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    last_err = Some(crate::AlesysError::LLM(e.to_string()));
                                }
                            }
                        }
                        Err(last_err.unwrap_or_else(|| crate::AlesysError::LLM("Embedder no disponible tras 5 intentos".to_string())))
                    })
                });
                if let Ok(json) = result {
                    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                        let mut out: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
                        for item in data {
                            if let Some(arr) = item.get("embedding").and_then(|e| e.as_array()) {
                                out.push(
                                    arr.iter()
                                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                                        .collect(),
                                );
                            }
                        }
                        if out.len() == texts.len() {
                            let _ = owned;
                            return Ok(out);
                        }
                    }
                }
            }
        }
        // NO fallback a embeddings falsos. Propagar el error.
        Err(crate::AlesysError::LLM(
            "Embedder no disponible para batch. Inicie el servicio de embeddings.".to_string()
        ))
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
