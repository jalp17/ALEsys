//! Backend HTTP unificado para proveedores cloud/remote
//!
//! Soporta 11 proveedores con una sola implementación:
//! - Ollama (local/remote, OpenAI-compatible)
//! - OpenRouter (OpenAI-compatible)
//! - Anthropic (formato propio)
//! - Google Gemini (formato propio)
//! - Perplexity (OpenAI-compatible)
//! - Cerebras (OpenAI-compatible)
//! - Cohere (formato propio)
//! - NVIDIA NIM (OpenAI-compatible)
//! - Groq (OpenAI-compatible)
//! - HuggingFace Inference (OpenAI-compatible)
//! - GitHub Models (OpenAI-compatible)
//!
//! Selección via `LLM_BACKEND=<provider>` + env vars de API key.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use super::{ChatMessage, ChatResponse, LLMConfig, LLMEngine, StreamChunk, Usage};
use crate::Result;

// =============================================================================
// Provider enum
// =============================================================================

/// Proveedores HTTP soportados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Ollama,
    OpenRouter,
    Anthropic,
    Gemini,
    Perplexity,
    Cerebras,
    Cohere,
    Nvidia,
    Groq,
    HuggingFace,
    GitHubModels,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ollama => write!(f, "ollama"),
            Self::OpenRouter => write!(f, "openrouter"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::Gemini => write!(f, "gemini"),
            Self::Perplexity => write!(f, "perplexity"),
            Self::Cerebras => write!(f, "cerebras"),
            Self::Cohere => write!(f, "cohere"),
            Self::Nvidia => write!(f, "nvidia"),
            Self::Groq => write!(f, "groq"),
            Self::HuggingFace => write!(f, "huggingface"),
            Self::GitHubModels => write!(f, "githubmodels"),
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openrouter" => Ok(Self::OpenRouter),
            "anthropic" | "claude" => Ok(Self::Anthropic),
            "gemini" | "google" | "googlegemini" => Ok(Self::Gemini),
            "perplexity" | "pplx" => Ok(Self::Perplexity),
            "cerebras" => Ok(Self::Cerebras),
            "cohere" | "command" => Ok(Self::Cohere),
            "nvidia" | "nim" => Ok(Self::Nvidia),
            "groq" => Ok(Self::Groq),
            "huggingface" | "hf" | "hfinference" => Ok(Self::HuggingFace),
            "githubmodels" | "github" => Ok(Self::GitHubModels),
            _ => Err(anyhow::anyhow!(
                "Proveedor desconocido: '{}'. Opciones: ollama, openrouter, anthropic, gemini, perplexity, cerebras, cohere, nvidia, groq, huggingface, githubmodels",
                s
            )),
        }
    }
}

impl Provider {
    /// Default base URL for each provider
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Ollama => "http://localhost:11434",
            Self::OpenRouter => "https://openrouter.ai/api",
            Self::Anthropic => "https://api.anthropic.com",
            Self::Gemini => "https://generativelanguage.googleapis.com",
            Self::Perplexity => "https://api.perplexity.ai",
            Self::Cerebras => "https://api.cerebras.ai",
            Self::Cohere => "https://api.cohere.com",
            Self::Nvidia => "https://integrate.api.nvidia.com",
            Self::Groq => "https://api.groq.com/openai",
            Self::HuggingFace => "https://api-inference.huggingface.co",
            Self::GitHubModels => "https://models.inference.ai.azure.com",
        }
    }

    /// Env var name for the API key
    pub fn api_key_env(&self) -> &'static str {
        match self {
            Self::Ollama => "OLLAMA_API_KEY",
            Self::OpenRouter => "OPENROUTER_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::Gemini => "GEMINI_API_KEY",
            Self::Perplexity => "PERPLEXITY_API_KEY",
            Self::Cerebras => "CEREBRAS_API_KEY",
            Self::Cohere => "COHERE_API_KEY",
            Self::Nvidia => "NVIDIA_API_KEY",
            Self::Groq => "GROQ_API_KEY",
            Self::HuggingFace => "HF_API_KEY",
            Self::GitHubModels => "GITHUB_TOKEN",
        }
    }

    /// Default model for each provider
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Ollama => "llama3.1",
            Self::OpenRouter => "meta-llama/llama-3.1-8b-instruct:free",
            Self::Anthropic => "claude-sonnet-4-20250514",
            Self::Gemini => "gemini-2.0-flash",
            Self::Perplexity => "sonar",
            Self::Cerebras => "llama-3.3-70b",
            Self::Cohere => "command-r-plus",
            Self::Nvidia => "meta/llama-3.1-8b-instruct",
            Self::Groq => "llama-3.3-70b-versatile",
            Self::HuggingFace => "meta-llama/Llama-3.3-70B-Instruct",
            Self::GitHubModels => "gpt-4o-mini",
        }
    }

    /// Whether this provider uses OpenAI-compatible API format
    pub fn is_openai_compatible(&self) -> bool {
        !matches!(self, Self::Anthropic | Self::Gemini | Self::Cohere)
    }
}

// =============================================================================
// HTTP LLM Engine
// =============================================================================

/// Backend HTTP unificado para todos los proveedores cloud/remote
pub struct HttpLLMEngine {
    config: LLMConfig,
    provider: Provider,
    client: reqwest::Client,
    api_url: String,
    api_key: String,
    model: String,
}

impl HttpLLMEngine {
    pub async fn new(config: LLMConfig, provider: Provider) -> Result<Self> {
        let api_key_env = provider.api_key_env();
        let api_key = std::env::var(api_key_env).unwrap_or_default();

        if api_key.is_empty() && provider != Provider::Ollama {
            tracing::warn!(
                "API key no configurada para {} ({}) — las llamadas fallarán",
                provider,
                api_key_env
            );
        }

        let base_url = std::env::var(format!("{}_BASE_URL", api_key_env))
            .unwrap_or_else(|_| provider.default_base_url().to_string());

        let model = std::env::var(format!("{}_MODEL", api_key_env.replace("API_KEY", "")))
            .unwrap_or_else(|_| {
                if config.model_path.is_empty() {
                    provider.default_model().to_string()
                } else {
                    config.model_path.clone()
                }
            });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| crate::AlesysError::LLM(format!("Error creando HTTP client: {}", e)))?;

        tracing::info!(
            "HttpLLMEngine: provider={}, model={}, url={}",
            provider,
            model,
            base_url,
        );

        Ok(Self {
            config,
            provider,
            client,
            api_url: base_url,
            api_key,
            model,
        })
    }

    // -------------------------------------------------------------------------
    // Message serialization — per provider
    // -------------------------------------------------------------------------

    /// Serialize messages for the provider's API format
    fn serialize_request(&self, messages: &[ChatMessage]) -> serde_json::Value {
        match self.provider {
            Provider::Anthropic => self.serialize_anthropic(messages),
            Provider::Gemini => self.serialize_gemini(messages),
            Provider::Cohere => self.serialize_cohere(messages),
            _ => self.serialize_openai(messages),
        }
    }

    /// OpenAI-compatible format (most providers)
    fn serialize_openai(&self, messages: &[ChatMessage]) -> serde_json::Value {
        let openai_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
            })
            .collect();

        serde_json::json!({
            "model": self.model,
            "messages": openai_messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
        })
    }

    /// Anthropic Messages API format
    fn serialize_anthropic(&self, messages: &[ChatMessage]) -> serde_json::Value {
        let mut system = String::new();
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => {
                    if !system.is_empty() {
                        system.push('\n');
                    }
                    system.push_str(&msg.content);
                }
                _ => {
                    anthropic_messages.push(serde_json::json!({
                        "role": msg.role,
                        "content": msg.content
                    }));
                }
            }
        }

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": anthropic_messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
        });

        if !system.is_empty() {
            body["system"] = serde_json::Value::String(system);
        }

        body
    }

    /// Google Gemini format (contents array)
    fn serialize_gemini(&self, messages: &[ChatMessage]) -> serde_json::Value {
        let mut system_instruction = None;
        let mut contents = Vec::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => {
                    system_instruction = Some(serde_json::json!({
                        "parts": [{"text": msg.content}]
                    }));
                }
                "user" => {
                    contents.push(serde_json::json!({
                        "role": "user",
                        "parts": [{"text": msg.content}]
                    }));
                }
                "assistant" => {
                    contents.push(serde_json::json!({
                        "role": "model",
                        "parts": [{"text": msg.content}]
                    }));
                }
                _ => {}
            }
        }

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": self.config.max_tokens,
                "temperature": self.config.temperature,
                "topP": self.config.top_p,
            }
        });

        if let Some(sys) = system_instruction {
            body["systemInstruction"] = sys;
        }

        body
    }

    /// Cohere Chat API format
    fn serialize_cohere(&self, messages: &[ChatMessage]) -> serde_json::Value {
        let mut chat_history = Vec::new();
        let mut message = String::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => {
                    // Cohere uses preamble for system
                }
                "user" => {
                    chat_history.push(serde_json::json!({
                        "role": "USER",
                        "message": msg.content
                    }));
                }
                "assistant" => {
                    chat_history.push(serde_json::json!({
                        "role": "CHATBOT",
                        "message": msg.content
                    }));
                }
                _ => {}
            }
        }

        // Last user message becomes the main message
        if let Some(last) = chat_history.last() {
            message = last["message"].as_str().unwrap_or("").to_string();
            chat_history.pop();
        }

        let preamble = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let mut body = serde_json::json!({
            "model": self.model,
            "message": message,
            "chat_history": chat_history,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "p": self.config.top_p,
        });

        if !preamble.is_empty() {
            body["preamble"] = serde_json::Value::String(preamble);
        }

        body
    }

    // -------------------------------------------------------------------------
    // API endpoint URL
    // -------------------------------------------------------------------------

    /// Build the full API URL for the request
    fn chat_endpoint(&self) -> String {
        match self.provider {
            Provider::Ollama => format!("{}/api/chat", self.api_url),
            Provider::Anthropic => format!("{}/v1/messages", self.api_url),
            Provider::Gemini => {
                format!(
                    "{}/v1beta/models/{}:generateContent?key={}",
                    self.api_url, self.model, self.api_key
                )
            }
            Provider::Cohere => format!("{}/v2/chat", self.api_url),
            // OpenAI-compatible providers
            _ => format!("{}/v1/chat/completions", self.api_url),
        }
    }

    /// Build the streaming endpoint URL
    fn chat_stream_endpoint(&self) -> String {
        match self.provider {
            Provider::Ollama => format!("{}/api/chat", self.api_url),
            Provider::Anthropic => format!("{}/v1/messages", self.api_url),
            Provider::Gemini => {
                format!(
                    "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                    self.api_url, self.model, self.api_key
                )
            }
            Provider::Cohere => format!("{}/v2/chat", self.api_url),
            _ => format!("{}/v1/chat/completions", self.api_url),
        }
    }

    // -------------------------------------------------------------------------
    // Auth headers
    // -------------------------------------------------------------------------

    fn build_request(&self, url: &str, body: serde_json::Value) -> reqwest::RequestBuilder {
        let mut req = self.client.post(url).json(&body);

        match self.provider {
            Provider::Ollama => {
                // Ollama doesn't need auth by default
            }
            Provider::Anthropic => {
                req = req
                    .header("x-api-key", &self.api_key)
                    .header("anthropic-version", "2023-06-01");
            }
            Provider::Gemini => {
                // API key is in the URL query param
            }
            Provider::Cohere => {
                req = req.header("Authorization", format!("Bearer {}", self.api_key));
            }
            _ => {
                req = req.header("Authorization", format!("Bearer {}", self.api_key));
            }
        }

        req
    }

    // -------------------------------------------------------------------------
    // Response parsing — per provider
    // -------------------------------------------------------------------------

    async fn parse_response(&self, body: serde_json::Value) -> Result<ChatResponse> {
        match self.provider {
            Provider::Anthropic => self.parse_anthropic(body),
            Provider::Gemini => self.parse_gemini(body),
            Provider::Cohere => self.parse_cohere(body),
            _ => self.parse_openai(body),
        }
    }

    fn parse_openai(&self, body: serde_json::Value) -> Result<ChatResponse> {
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let usage = &body["usage"];
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            usage: Usage {
                prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as usize,
                completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as usize,
                total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as usize,
            },
        })
    }

    fn parse_anthropic(&self, body: serde_json::Value) -> Result<ChatResponse> {
        let content = body["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let usage = &body["usage"];
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            usage: Usage {
                prompt_tokens: usage["input_tokens"].as_u64().unwrap_or(0) as usize,
                completion_tokens: usage["output_tokens"].as_u64().unwrap_or(0) as usize,
                total_tokens: (usage["input_tokens"].as_u64().unwrap_or(0)
                    + usage["output_tokens"].as_u64().unwrap_or(0))
                    as usize,
            },
        })
    }

    fn parse_gemini(&self, body: serde_json::Value) -> Result<ChatResponse> {
        let content = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let usage = &body["usageMetadata"];
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            usage: Usage {
                prompt_tokens: usage["promptTokenCount"].as_u64().unwrap_or(0) as usize,
                completion_tokens: usage["candidatesTokenCount"].as_u64().unwrap_or(0) as usize,
                total_tokens: usage["totalTokenCount"].as_u64().unwrap_or(0) as usize,
            },
        })
    }

    fn parse_cohere(&self, body: serde_json::Value) -> Result<ChatResponse> {
        let content = body["message"]["content"]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let tokens = body["tokens"].clone();
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            usage: Usage {
                prompt_tokens: tokens["input_tokens"].as_u64().unwrap_or(0) as usize,
                completion_tokens: tokens["output_tokens"].as_u64().unwrap_or(0) as usize,
                total_tokens: (tokens["input_tokens"].as_u64().unwrap_or(0)
                    + tokens["output_tokens"].as_u64().unwrap_or(0))
                    as usize,
            },
        })
    }
}

// =============================================================================
// LLMEngine trait implementation
// =============================================================================

#[async_trait]
impl LLMEngine for HttpLLMEngine {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        let body = self.serialize_request(messages);
        let url = self.chat_endpoint();

        tracing::debug!("HTTP LLM request: provider={}, url={}", self.provider, url);

        let response = self.build_request(&url, body).send().await.map_err(|e| {
            crate::AlesysError::LLM(format!("Error en request HTTP ({}): {}", self.provider, e))
        })?;

        let status = response.status();
        let body: serde_json::Value = response.json().await.map_err(|e| {
            crate::AlesysError::LLM(format!(
                "Error parseando respuesta ({}): {}",
                self.provider, e
            ))
        })?;

        if !status.is_success() {
            let error_msg = body["error"]["message"]
                .as_str()
                .or_else(|| body["message"].as_str())
                .unwrap_or("unknown error");
            return Err(crate::AlesysError::LLM(format!(
                "API error {} ({}): {}",
                status.as_u16(),
                self.provider,
                error_msg
            )));
        }

        self.parse_response(body).await
    }

    fn chat_stream<'a>(
        &'a self,
        messages: &'a [ChatMessage],
    ) -> BoxStream<'a, Result<StreamChunk>> {
        let body = self.serialize_request(messages);
        let url = self.chat_stream_endpoint();
        let provider = self.provider;

        // Build request with streaming flag
        let mut req = self.build_request(&url, body.clone());

        // Add streaming flag for OpenAI-compatible
        if provider.is_openai_compatible() {
            let mut stream_body = body.clone();
            stream_body["stream"] = serde_json::Value::Bool(true);
            req = self.client.post(&url).json(&stream_body);
            // Re-apply auth headers
            match provider {
                Provider::Ollama => {}
                _ => {
                    req = req.header("Authorization", format!("Bearer {}", self.api_key));
                }
            }
        }

        // Add streaming-specific fields
        match provider {
            Provider::Anthropic => {
                let mut stream_body = body.clone();
                stream_body["stream"] = serde_json::Value::Bool(true);
                req = self
                    .client
                    .post(&url)
                    .json(&stream_body)
                    .header("x-api-key", &self.api_key)
                    .header("anthropic-version", "2023-06-01");
            }
            Provider::Cohere => {
                let mut stream_body = body.clone();
                stream_body["stream"] = serde_json::Value::Bool(true);
                req = self
                    .client
                    .post(&url)
                    .json(&stream_body)
                    .header("Authorization", format!("Bearer {}", self.api_key));
            }
            Provider::Gemini => {
                // SSE is in the URL via alt=sse — nothing extra needed
            }
            _ => {
                // OpenAI-compatible: stream already set above
            }
        }

        let stream_fut = async move {
            let (tx, rx) = tokio::sync::mpsc::channel(64);

            let response = req.send().await.map_err(|e| {
                crate::AlesysError::LLM(format!("Error en stream request ({}): {}", provider, e))
            })?;

            let status = response.status();
            if !status.is_success() {
                let body: serde_json::Value = response.json().await.unwrap_or_default();
                let error_msg = body["error"]["message"]
                    .as_str()
                    .or_else(|| body["message"].as_str())
                    .unwrap_or("unknown error");
                return Err(crate::AlesysError::LLM(format!(
                    "API error {} ({}): {}",
                    status.as_u16(),
                    provider,
                    error_msg
                )));
            }

            let stream = response.bytes_stream();
            tokio::spawn(async move {
                let mut buffer = String::new();
                let mut stream = std::pin::pin!(stream);

                while let Some(chunk_result) = stream.next().await {
                    let chunk = match chunk_result {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = tx
                                .send(Err(crate::AlesysError::LLM(format!(
                                    "Stream read error: {}",
                                    e
                                ))))
                                .await;
                            break;
                        }
                    };

                    buffer.push_str(&String::from_utf8_lossy(&chunk));

                    // Process complete lines
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].trim().to_string();
                        buffer = buffer[newline_pos + 1..].to_string();

                        if line.is_empty() || line.starts_with(':') {
                            continue;
                        }

                        let delta = match provider {
                            Provider::Anthropic => parse_anthropic_sse_line(&line),
                            Provider::Gemini => parse_gemini_sse_line(&line),
                            Provider::Cohere => parse_cohere_sse_line(&line),
                            _ => parse_openai_sse_line(&line),
                        };

                        if let Some(chunk) = delta {
                            if tx.send(Ok(chunk)).await.is_err() {
                                return;
                            }
                        }
                    }
                }

                // Send final stop chunk
                let _ = tx
                    .send(Ok(StreamChunk {
                        delta: String::new(),
                        finish_reason: Some("stop".to_string()),
                    }))
                    .await;
            });

            Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
                as BoxStream<'static, Result<StreamChunk>>)
        };

        Box::pin(
            futures::stream::once(stream_fut).flat_map(|result| match result {
                Ok(stream) => stream,
                Err(e) => Box::pin(futures::stream::once(async move { Err(e) }))
                    as BoxStream<'static, Result<StreamChunk>>,
            }),
        )
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty() || self.provider == Provider::Ollama
    }

    fn backend_name(&self) -> &str {
        // Return a static str — use Box::leak for convenience
        Box::leak(self.provider.to_string().into_boxed_str())
    }
}

// =============================================================================
// SSE line parsers per provider
// =============================================================================

fn parse_openai_sse_line(line: &str) -> Option<StreamChunk> {
    let line = line.strip_prefix("data: ").unwrap_or(line);

    if line == "[DONE]" {
        return Some(StreamChunk {
            delta: String::new(),
            finish_reason: Some("stop".to_string()),
        });
    }

    let json: serde_json::Value = serde_json::from_str(line).ok()?;
    let delta = json["choices"][0]["delta"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let finish_reason = json["choices"][0]["finish_reason"]
        .as_str()
        .map(|s| s.to_string());

    if delta.is_empty() && finish_reason.is_none() {
        return None;
    }

    Some(StreamChunk {
        delta,
        finish_reason,
    })
}

fn parse_anthropic_sse_line(line: &str) -> Option<StreamChunk> {
    let line = line.strip_prefix("data: ").unwrap_or(line);

    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    match json["type"].as_str()? {
        "content_block_delta" => {
            let delta = json["delta"]["text"].as_str().unwrap_or("").to_string();
            Some(StreamChunk {
                delta,
                finish_reason: None,
            })
        }
        "message_stop" => Some(StreamChunk {
            delta: String::new(),
            finish_reason: Some("stop".to_string()),
        }),
        _ => None,
    }
}

fn parse_gemini_sse_line(line: &str) -> Option<StreamChunk> {
    let line = line.strip_prefix("data: ").unwrap_or(line);

    let json: serde_json::Value = serde_json::from_str(line).ok()?;
    let delta = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let finish_reason = json["candidates"][0]["finishReason"]
        .as_str()
        .map(|s| s.to_string());

    if delta.is_empty() && finish_reason.is_none() {
        return None;
    }

    Some(StreamChunk {
        delta,
        finish_reason,
    })
}

fn parse_cohere_sse_line(line: &str) -> Option<StreamChunk> {
    let line = line.strip_prefix("data: ").unwrap_or(line);

    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    match json["event-type"].as_str()? {
        "content-delta" => {
            let delta = json["delta"]["message"]["content"]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();
            Some(StreamChunk {
                delta,
                finish_reason: None,
            })
        }
        "message-end" => Some(StreamChunk {
            delta: String::new(),
            finish_reason: Some("stop".to_string()),
        }),
        _ => None,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_display() {
        assert_eq!(Provider::Ollama.to_string(), "ollama");
        assert_eq!(Provider::Anthropic.to_string(), "anthropic");
        assert_eq!(Provider::Groq.to_string(), "groq");
    }

    #[test]
    fn test_provider_from_str() {
        assert_eq!("ollama".parse::<Provider>().unwrap(), Provider::Ollama);
        assert_eq!(
            "anthropic".parse::<Provider>().unwrap(),
            Provider::Anthropic
        );
        assert_eq!("claude".parse::<Provider>().unwrap(), Provider::Anthropic);
        assert_eq!("gemini".parse::<Provider>().unwrap(), Provider::Gemini);
        assert_eq!("google".parse::<Provider>().unwrap(), Provider::Gemini);
        assert_eq!("groq".parse::<Provider>().unwrap(), Provider::Groq);
        assert_eq!("hf".parse::<Provider>().unwrap(), Provider::HuggingFace);
        assert_eq!(
            "github".parse::<Provider>().unwrap(),
            Provider::GitHubModels
        );
        assert_eq!("nim".parse::<Provider>().unwrap(), Provider::Nvidia);
        assert!("unknown".parse::<Provider>().is_err());
    }

    #[test]
    fn test_provider_default_base_url() {
        assert_eq!(
            Provider::Anthropic.default_base_url(),
            "https://api.anthropic.com"
        );
        assert_eq!(
            Provider::Groq.default_base_url(),
            "https://api.groq.com/openai"
        );
        assert_eq!(
            Provider::HuggingFace.default_base_url(),
            "https://api-inference.huggingface.co"
        );
    }

    #[test]
    fn test_provider_api_key_env() {
        assert_eq!(Provider::Anthropic.api_key_env(), "ANTHROPIC_API_KEY");
        assert_eq!(Provider::Groq.api_key_env(), "GROQ_API_KEY");
        assert_eq!(Provider::GitHubModels.api_key_env(), "GITHUB_TOKEN");
    }

    #[test]
    fn test_provider_is_openai_compatible() {
        assert!(Provider::Ollama.is_openai_compatible());
        assert!(Provider::Groq.is_openai_compatible());
        assert!(Provider::Nvidia.is_openai_compatible());
        assert!(!Provider::Anthropic.is_openai_compatible());
        assert!(!Provider::Gemini.is_openai_compatible());
        assert!(!Provider::Cohere.is_openai_compatible());
    }

    #[test]
    fn test_parse_openai_sse_line() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let chunk = parse_openai_sse_line(line).unwrap();
        assert_eq!(chunk.delta, "Hello");
        assert!(chunk.finish_reason.is_none());
    }

    #[test]
    fn test_parse_openai_sse_done() {
        let chunk = parse_openai_sse_line("data: [DONE]").unwrap();
        assert_eq!(chunk.delta, "");
        assert_eq!(chunk.finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn test_parse_anthropic_sse_line() {
        let line = r#"data: {"type":"content_block_delta","delta":{"text":"Hi"}}"#;
        let chunk = parse_anthropic_sse_line(line).unwrap();
        assert_eq!(chunk.delta, "Hi");
    }

    #[test]
    fn test_serialize_openai_format() {
        let config = LLMConfig::default();
        let engine = HttpLLMEngine {
            config,
            provider: Provider::Groq,
            client: reqwest::Client::new(),
            api_url: "https://api.groq.com/openai".to_string(),
            api_key: "test".to_string(),
            model: "llama-3.3-70b-versatile".to_string(),
        };

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let body = engine.serialize_request(&messages);
        assert_eq!(body["model"], "llama-3.3-70b-versatile");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["role"], "user");
    }

    #[test]
    fn test_serialize_anthropic_format() {
        let config = LLMConfig::default();
        let engine = HttpLLMEngine {
            config,
            provider: Provider::Anthropic,
            client: reqwest::Client::new(),
            api_url: "https://api.anthropic.com".to_string(),
            api_key: "test".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        };

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let body = engine.serialize_request(&messages);
        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["system"], "You are helpful");
        assert_eq!(body["messages"][0]["role"], "user");
    }

    #[test]
    fn test_serialize_gemini_format() {
        let config = LLMConfig::default();
        let engine = HttpLLMEngine {
            config,
            provider: Provider::Gemini,
            client: reqwest::Client::new(),
            api_url: "https://generativelanguage.googleapis.com".to_string(),
            api_key: "test".to_string(),
            model: "gemini-2.0-flash".to_string(),
        };

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let body = engine.serialize_request(&messages);
        assert_eq!(
            body["systemInstruction"]["parts"][0]["text"],
            "You are helpful"
        );
        assert_eq!(body["contents"][0]["role"], "user");
        assert_eq!(body["generationConfig"]["maxOutputTokens"], 2048);
    }
}
