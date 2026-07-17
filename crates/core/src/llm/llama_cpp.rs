//! Motor LLM con llama.cpp (Vulkan GPU)
//!
//! Soporta 150+ arquitecturas y 23 quantizaciones via Vulkan.
//! Requiere feature `llama-cpp` habilitada.

#[cfg(feature = "llama-cpp")]
use llama_cpp::{standard_sampler::StandardSampler, LlamaModel, LlamaParams, SessionParams};

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use super::{ChatMessage, ChatResponse, LLMConfig, LLMEngine, StreamChunk, Usage};
use crate::Result;

pub struct LlamaCppEngine {
    config: LLMConfig,
    #[cfg(feature = "llama-cpp")]
    model: LlamaModel,
}

impl LlamaCppEngine {
    pub async fn new(config: LLMConfig) -> Result<Self> {
        #[cfg(feature = "llama-cpp")]
        {
            if config.model_path.is_empty() {
                return Err(crate::AlesysError::LLM(
                    "LLM_MODEL_PATH no configurado".to_string(),
                ));
            }

            let path = std::path::Path::new(&config.model_path);
            if !path.exists() {
                return Err(crate::AlesysError::LLM(format!(
                    "Modelo no encontrado: {}",
                    config.model_path
                )));
            }

            let n_gpu = config.gpu_layers;

            tracing::info!(
                "Cargando modelo llama.cpp (Vulkan): {} (gpu_layers={})",
                path.display(),
                n_gpu,
            );

            let params = LlamaParams {
                n_gpu_layers: n_gpu,
                ..Default::default()
            };

            let model = LlamaModel::load_from_file(&config.model_path, params)
                .map_err(|e| crate::AlesysError::LLM(format!("Error cargando modelo: {}", e)))?;

            tracing::info!("Modelo cargado exitosamente en GPU via Vulkan");

            Ok(Self { config, model })
        }

        #[cfg(not(feature = "llama-cpp"))]
        {
            let _ = config;
            Err(crate::AlesysError::LLM(
                "Feature 'llama-cpp' no habilitada".to_string(),
            ))
        }
    }

    fn format_messages(&self, messages: &[ChatMessage]) -> String {
        let mut out = String::new();
        for msg in messages {
            match msg.role.as_str() {
                "system" => {
                    out.push_str(&msg.content);
                    out.push('\n');
                }
                "user" => {
                    out.push_str(&format!("Usuario: {}\n", msg.content));
                }
                "assistant" => {
                    out.push_str(&format!("Asistente: {}\n", msg.content));
                }
                _ => {
                    out.push_str(&format!("{}: {}\n", msg.role, msg.content));
                }
            }
        }
        out.push_str("Asistente:");
        out
    }
}

#[async_trait]
impl LLMEngine for LlamaCppEngine {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        #[cfg(feature = "llama-cpp")]
        {
            let prompt = self.format_messages(messages);
            let model = self.model.clone();
            let max_tokens = self.config.max_tokens;
            let context_size = self.config.context_size;

            let response_text = tokio::task::spawn_blocking(move || -> Result<String> {
                let session_params = SessionParams {
                    n_ctx: context_size as u32,
                    ..Default::default()
                };

                let mut session = model.create_session(session_params).map_err(|e| {
                    crate::AlesysError::LLM(format!("Error creando sesión: {}", e))
                })?;

                session.advance_context(prompt).map_err(|e| {
                    crate::AlesysError::LLM(format!("Error en advance_context: {}", e))
                })?;

                let handle = session
                    .start_completing_with(StandardSampler::default(), max_tokens)
                    .map_err(|e| {
                        crate::AlesysError::LLM(format!("Error en start_completing: {}", e))
                    })?;

                let mut out = String::new();
                for token_str in handle.into_strings() {
                    out.push_str(&token_str);
                }

                Ok(out)
            })
            .await
            .map_err(|e| crate::AlesysError::LLM(format!("Join error: {}", e)))??;

            let prompt_tokens = messages.iter().map(|m| m.content.len() / 4).sum();
            let completion_tokens = response_text.len() / 4;

            Ok(ChatResponse {
                content: response_text,
                model: self.config.model_path.clone(),
                usage: Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                },
            })
        }

        #[cfg(not(feature = "llama-cpp"))]
        {
            let _ = messages;
            Err(crate::AlesysError::LLM(
                "Feature 'llama-cpp' no habilitada".to_string(),
            ))
        }
    }

    fn chat_stream<'a>(
        &'a self,
        messages: &'a [ChatMessage],
    ) -> BoxStream<'a, Result<StreamChunk>> {
        #[cfg(feature = "llama-cpp")]
        {
            let prompt = self.format_messages(messages);
            let model = self.model.clone();
            let max_tokens = self.config.max_tokens;
            let context_size = self.config.context_size;

            let (tx, rx) = tokio::sync::mpsc::channel(64);

            tokio::task::spawn_blocking(move || {
                let session_params = SessionParams {
                    n_ctx: context_size as u32,
                    ..Default::default()
                };

                let session = match model.create_session(session_params) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.blocking_send(Err(crate::AlesysError::LLM(
                            format!("Error creando sesión: {}", e),
                        )));
                        return;
                    }
                };

                let mut session = session;
                if let Err(e) = session.advance_context(prompt) {
                    let _ = tx.blocking_send(Err(crate::AlesysError::LLM(
                        format!("Error en advance_context: {}", e),
                    )));
                    return;
                }

                let handle = match session.start_completing_with(StandardSampler::default(), max_tokens) {
                    Ok(h) => h,
                    Err(e) => {
                        let _ = tx.blocking_send(Err(crate::AlesysError::LLM(
                            format!("Error en start_completing: {}", e),
                        )));
                        return;
                    }
                };

                for token_str in handle.into_strings() {
                    if tx.blocking_send(Ok(StreamChunk {
                        delta: token_str,
                        finish_reason: None,
                    })).is_err() {
                        break;
                    }
                }

                let _ = tx.blocking_send(Ok(StreamChunk {
                    delta: String::new(),
                    finish_reason: Some("stop".to_string()),
                }));
            });

            Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
        }

        #[cfg(not(feature = "llama-cpp"))]
        {
            let _ = messages;
            Box::pin(futures::stream::once(async {
                Err(crate::AlesysError::LLM(
                    "Feature 'llama-cpp' no habilitada".to_string(),
                ))
            }))
        }
    }

    fn is_available(&self) -> bool {
        #[cfg(feature = "llama-cpp")]
        {
            true
        }
        #[cfg(not(feature = "llama-cpp"))]
        {
            false
        }
    }

    fn backend_name(&self) -> &str {
        "llama_cpp"
    }
}

/// Información de disponibilidad del backend
pub fn availability_info() -> serde_json::Value {
    serde_json::json!({
        "name": "llama_cpp",
        "description": "Backend principal con Vulkan GPU - 150+ arquitecturas",
        "features": ["llama-cpp"],
        "gpu_support": { "vulkan": true, "cuda": true, "metal": true, "cpu": true },
    })
}
