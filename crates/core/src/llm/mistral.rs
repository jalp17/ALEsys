//! Motor LLM con mistralrs (CPU)
//!
//! Soporta modelos GGUF cuantizados via mistralrs.
//! Limitaciones: sin MoE, quantizaciones limitadas, solo CPU.
//! Requiere feature `mistralrs-backend` habilitada.

#[cfg(feature = "mistralrs-backend")]
use mistralrs::{
    GgufModelBuilder, Model, TextMessageRole, TextMessages,
};

use crate::Result;
use super::{LLMConfig, LLMEngine, ChatMessage, ChatResponse, StreamChunk, Usage};

pub struct MistralEngine {
    config: LLMConfig,
    #[cfg(feature = "mistralrs-backend")]
    model: Option<Model>,
}

impl MistralEngine {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "mistralrs-backend")]
            model: None,
        }
    }

    #[cfg(feature = "mistralrs-backend")]
    pub async fn load(&mut self) -> Result<()> {
        if self.config.model_path.is_empty() {
            tracing::warn!("No se especificó modelo LLM, modo solo búsqueda");
            return Ok(());
        }

        let path = std::path::Path::new(&self.config.model_path);
        if !path.exists() {
            return Err(crate::AlesysError::LLM(format!(
                "Modelo no encontrado: {}",
                self.config.model_path
            )));
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let parent = path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".")
            .to_string();

        tracing::info!("Cargando modelo GGUF (mistralrs): {} en {}", filename, parent);

        let mut builder = GgufModelBuilder::new(&parent, vec![&filename]).with_logging();

        if let Some(ref template_path) = self.config.chat_template {
            tracing::info!("Usando chat template personalizado: {}", template_path);
            builder = builder.with_chat_template(template_path);
        }

        let model = builder.build().await?;

        tracing::info!("Modelo cargado exitosamente (mistralrs): {}", filename);
        self.model = Some(model);
        Ok(())
    }
}

impl LLMEngine for MistralEngine {
    fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        #[cfg(feature = "mistralrs-backend")]
        {
            let Some(ref model) = self.model else {
                let content = format!(
                    "[Modo solo búsqueda - LLM no disponible]\n\n\
                     Pregunta: {}\n\n\
                     No hay modelo LLM configurado.",
                    messages.last().map(|m| m.content.as_str()).unwrap_or("")
                );
                return Ok(ChatResponse {
                    content,
                    model: "mistralrs (sin modelo)".to_string(),
                    usage: Usage { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
                });
            };

            let mut text_messages = TextMessages::new();
            for msg in messages {
                let role = match msg.role.as_str() {
                    "system" => TextMessageRole::System,
                    "user" => TextMessageRole::User,
                    "assistant" => TextMessageRole::Assistant,
                    _ => TextMessageRole::User,
                };
                text_messages = text_messages.add_message(role, &msg.content);
            }

            let response = model
                .send_chat_request(text_messages)
                .await
                .map_err(|e| crate::AlesysError::LLM(format!("Error en inferencia: {}", e)))?;

            let content = response.choices[0]
                .message
                .content
                .as_deref()
                .unwrap_or("[Sin respuesta]")
                .to_string();

            let prompt_tokens = messages.iter().map(|m| m.content.len() / 4).sum();
            let completion_tokens = content.len() / 4;

            Ok(ChatResponse {
                content,
                model: "mistralrs".to_string(),
                usage: Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                },
            })
        }

        #[cfg(not(feature = "mistralrs-backend"))]
        {
            let _ = messages;
            Err(crate::AlesysError::LLM(
                "Feature 'mistralrs-backend' no habilitada".to_string()
            ))
        }
    }

    fn chat_stream(&self, messages: &[ChatMessage]) -> Result<Box<dyn Iterator<Item = Result<StreamChunk>> + Send>> {
        let response = self.chat(messages)?;
        let chunks = vec![Ok(StreamChunk {
            delta: response.content,
            finish_reason: Some("stop".to_string()),
        })];
        Ok(Box::new(chunks.into_iter()))
    }

    fn generate_code(&self, prompt: &str, language: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!("Eres un asistente de programación. Genera código en {}.", language),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];
        let response = self.chat(&messages)?;
        Ok(response.content)
    }

    fn extract_knowledge(&self, text: &str, schema: &str) -> Result<String> {
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
        let response = self.chat(&messages)?;
        Ok(response.content)
    }

    fn is_available(&self) -> bool {
        #[cfg(feature = "mistralrs-backend")]
        { self.model.is_some() }
        #[cfg(not(feature = "mistralrs-backend"))]
        { false }
    }

    fn backend_name(&self) -> &str {
        "mistralrs"
    }
}
