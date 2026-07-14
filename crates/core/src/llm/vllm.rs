//! Backend vLLM (subprocess)
//!
//! Implementación de `LLMEngine` usando vLLM como subprocess Python
//! con API compatible OpenAI.

use super::config::{GpuType, PythonConfig};
use super::{ChatMessage, ChatResponse, LLMConfig, LLMEngine, StreamChunk};
use crate::Result;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Backend vLLM para inferencia de alta eficiencia
pub struct VllmEngine {
    process: Option<Mutex<Child>>,
    base_url: String,
    config: LLMConfig,
    python_path: PathBuf,
    started_at: Option<Instant>,
}

impl VllmEngine {
    /// Crea una nueva instancia de VllmEngine
    pub async fn new(config: LLMConfig) -> Result<Self> {
        // Buscar Python en PATH o usar configuración
        let python_path = if let Some(python_path) = &config.python_path {
            PathBuf::from(python_path)
        } else {
            Self::find_python("3.10")? // Versión mínima recomendada
        };

        let base_url = format!("http://127.0.0.1:{}", config.server_port);

        Ok(Self {
            process: None,
            base_url,
            config,
            python_path,
            started_at: None,
        })
    }

    /// Busca ejecutable Python compatible
    fn find_python(version_req: &str) -> Result<PathBuf> {
        let candidates = vec!["python3".to_string(), "python".to_string()];

        for candidate in &candidates {
            if let Ok(output) = Command::new(candidate).arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                tracing::info!("Python encontrado: {} ({})", candidate, version.trim());
                return Ok(PathBuf::from(candidate));
            }
        }

        Err(crate::AlesysError::LLM(format!(
            "Python {} no encontrado",
            version_req
        )))
    }

    /// Inicia el servidor vLLM como subprocess
    pub async fn start_server(&mut self, model_path: &str, gpu_layers: Option<u32>) -> Result<()> {
        if self.process.is_some() {
            tracing::warn!("Servidor vLLM ya está corriendo");
            return Ok(());
        }

        let model = self.config.model_path.as_str();

        tracing::info!("Iniciando servidor vLLM con modelo: {}", model);

        let mut args = vec![
            "-m".to_string(),
            "vllm.entrypoints.openai.api_server".to_string(),
            "--model".to_string(),
            model.to_string(),
            "--host".to_string(),
            "127.0.0.1".to_string(),
            "--port".to_string(),
            self.config.server_port.to_string(),
            "--gpu-memory-utilization".to_string(),
            self.config.gpu_memory_utilization.to_string(),
            "--tensor-parallel-size".to_string(),
            self.config.tensor_parallel_size.to_string(),
        ];

        if let Some(max_model_len) = self.config.max_model_len {
            args.push("--max-model-len".to_string());
            args.push(max_model_len.to_string());
        }

        let child = Command::new(&self.python_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::AlesysError::LLM(format!("Error iniciando vLLM: {}", e)))?;

        self.process = Some(Mutex::new(child));
        self.started_at = Some(Instant::now());

        // Esperar a que el servidor esté listo
        self.wait_for_server().await?;

        tracing::info!("Servidor vLLM iniciado en {}", self.base_url);
        Ok(())
    }

    /// Espera a que el servidor esté respondiendo
    async fn wait_for_server(&self) -> Result<()> {
        let timeout = Duration::from_secs(120);
        let start = Instant::now();
        let client = reqwest::Client::new();

        loop {
            if start.elapsed() > timeout {
                return Err(crate::AlesysError::LLM(
                    "Timeout esperando servidor vLLM".to_string(),
                ));
            }

            match client.get(format!("{}/health", self.base_url)).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!("Servidor vLLM listo");
                    return Ok(());
                }
                _ => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Detiene el servidor vLLM
    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            tracing::info!("Deteniendo servidor vLLM...");
            let mut child = process.lock().await;
            child
                .kill()
                .map_err(|e| crate::AlesysError::LLM(format!("Error deteniendo vLLM: {}", e)))?;
            self.started_at = None;
            tracing::info!("Servidor vLLM detenido");
        }
        Ok(())
    }

    /// Verifica si el servidor está corriendo
    pub async fn is_running(&self) -> bool {
        let client = reqwest::Client::new();
        client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

impl LLMEngine for VllmEngine {
    fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        // vLLM es async por naturaleza, pero la trait es sync
        // Usamos bloqueo para mantener compatibilidad
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let client = reqwest::Client::new();

                // Formatear mensajes para OpenAI API
                let openai_messages: Vec<serde_json::Value> = messages
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "role": m.role,
                            "content": m.content
                        })
                    })
                    .collect();

                let response = client
                    .post(format!("{}/v1/chat/completions", self.base_url))
                    .json(&serde_json::json!({
                        "model": self.config.model_path,
                        "messages": openai_messages,
                        "max_tokens": self.config.max_tokens,
                        "temperature": self.config.temperature,
                        "top_p": self.config.top_p,
                    }))
                    .send()
                    .await
                    .map_err(|e| {
                        crate::AlesysError::LLM(format!("Error en request vLLM: {}", e))
                    })?;

                let body: serde_json::Value = response.json().await.map_err(|e| {
                    crate::AlesysError::LLM(format!("Error parseando respuesta: {}", e))
                })?;

                let content = body["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                let usage = &body["usage"];
                Ok(ChatResponse {
                    content,
                    model: self.config.model_path.clone(),
                    usage: super::Usage {
                        prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as usize,
                        completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0)
                            as usize,
                        total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as usize,
                    },
                })
            })
        })
    }

    fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Box<dyn Iterator<Item = Result<StreamChunk>> + Send>> {
        // Para streaming, usar SSE con vLLM
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
        self.process.is_some() || tokio::runtime::Handle::current().block_on(self.is_running())
    }

    fn backend_name(&self) -> &str {
        "vllm"
    }
}

impl Drop for VllmEngine {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            tracing::info!("Deteniendo servidor vLLM en drop...");
            let mut child = process.lock().await;
            let _ = child.kill();
        }
    }
}

/// Información de disponibilidad del backend
pub fn availability_info() -> serde_json::Value {
    serde_json::json!({
        "name": "vllm",
        "description": "Backend Python de alta eficiencia con soporte GPU",
        "features": ["vllm-backend"],
        "requirements": {
            "python": ">=3.9",
            "gpu": "Recomendado (CUDA)",
        },
        "capabilities": [
            "PagedAttention para eficiencia de memoria",
            "Continuous batching",
            "Tensor parallelism",
            "Speculative decoding",
        ],
        "limitations": [
            "Requiere servidor HTTP corriendo",
            "Mayor latencia que backends nativos",
            "Soporte CUDA limitado en CPU-only",
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_availability_info() {
        let info = availability_info();
        assert_eq!(info["name"], "vllm");
        assert!(info["capabilities"].as_array().unwrap().len() > 0);
    }
}
