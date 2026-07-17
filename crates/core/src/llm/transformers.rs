//! Backend Transformers (subprocess)
//!
//! Implementación de `LLMEngine` usando HuggingFace Transformers como subprocess Python.

use super::config::PythonConfig;
use async_trait::async_trait;
use futures::stream::BoxStream;
use super::{ChatMessage, ChatResponse, LLMConfig, LLMEngine, StreamChunk};
use crate::Result;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::sync::Mutex;

/// Backend Transformers para inferencia con modelos HF
pub struct TransformersEngine {
    process: Option<Mutex<Child>>,
    base_url: String,
    config: LLMConfig,
    python_path: PathBuf,
}

impl TransformersEngine {
    /// Crea una nueva instancia de TransformersEngine
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
        })
    }

    /// Busca ejecutable Python
    fn find_python(_version_req: &str) -> Result<PathBuf> {
        let candidates = vec!["python3".to_string(), "python".to_string()];

        for candidate in &candidates {
            if let Ok(output) = Command::new(candidate).arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                tracing::info!("Python encontrado: {} ({})", candidate, version.trim());
                return Ok(PathBuf::from(candidate));
            }
        }

        Err(crate::AlesysError::LLM("Python no encontrado".to_string()))
    }

    /// Inicia el servidor Transformers
    pub async fn start_server(&mut self, model_path: &str, gpu_layers: Option<u32>) -> Result<()> {
        if self.process.is_some() {
            return Ok(());
        }

        let model = self.config.model_path.as_str();

        // Validar que model_path no contenga caracteres peligrosos para injection
        if model.contains(';') || model.contains('|') || model.contains('&')
            || model.contains('$') || model.contains('`') || model.contains('\n')
        {
            return Err(crate::AlesysError::LLM(
                "model_path contiene caracteres no validos".to_string(),
            ));
        }

        tracing::info!("Iniciando servidor Transformers con modelo: {}", model);

        // Usar JSON file para pasar config en vez de interpolar en -c (evita code injection)
        let server_script = format!(
            r#"import json, sys
config = json.loads(sys.argv[1])
from transformers import AutoModelForCausalLM, AutoTokenizer
import torch
from flask import Flask, request, jsonify
app = Flask(__name__)
tokenizer = AutoTokenizer.from_pretrained(config['model'])
model = AutoModelForCausalLM.from_pretrained(config['model'], device_map='auto')
@app.route('/v1/chat/completions', methods=['POST'])
def chat():
    data = request.json
    inputs = tokenizer(data['messages'][-1]['content'], return_tensors='pt')
    outputs = model.generate(**inputs, max_new_tokens=data.get('max_tokens', 512))
    return jsonify({{'choices': [{{'message': {{'content': tokenizer.decode(outputs[0], skip_special_tokens=True)}}}}]}})
app.run(host='127.0.0.1', port={})"#,
            self.config.server_port
        );

        let config_json = serde_json::json!({
            "model": model,
        }).to_string();

        let mut args = vec![
            "-c".to_string(),
            server_script,
            config_json,
        ];

        if let Some(layers) = gpu_layers {
            args.insert(0, "--device".to_string());
            args.insert(1, "cuda".to_string());
        }

        let child = Command::new(&self.python_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::AlesysError::LLM(format!("Error iniciando Transformers: {}", e)))?;

        self.process = Some(Mutex::new(child));

        // Esperar a que esté listo
        self.wait_for_server().await?;

        tracing::info!("Servidor Transformers iniciado en {}", self.base_url);
        Ok(())
    }

    /// Espera a que el servidor esté respondiendo
    async fn wait_for_server(&self) -> Result<()> {
        let timeout = Duration::from_secs(180); // Transformers puede ser lento
        let start = std::time::Instant::now();
        let client = reqwest::Client::new();

        loop {
            if start.elapsed() > timeout {
                return Err(crate::AlesysError::LLM(
                    "Timeout esperando Transformers".to_string(),
                ));
            }

            match client.get(format!("{}/health", self.base_url)).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                _ => tokio::time::sleep(Duration::from_secs(2)).await,
            }
        }
    }

    /// Detiene el servidor
    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            tracing::info!("Deteniendo servidor Transformers...");
            let mut child = process.lock().await;
            let _ = child.kill();
            tracing::info!("Servidor Transformers detenido");
        }
        Ok(())
    }
}

#[async_trait]
impl LLMEngine for TransformersEngine {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        let openai_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
            })
            .collect();

        let base_url = self.base_url.clone();
        let model_path = self.config.model_path.clone();
        let max_tokens = self.config.max_tokens;
        let temperature = self.config.temperature;
        let top_p = self.config.top_p;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/chat/completions", base_url))
            .json(&serde_json::json!({
                "model": model_path,
                "messages": openai_messages,
                "max_tokens": max_tokens,
                "temperature": temperature,
                "top_p": top_p,
            }))
            .send()
            .await
            .map_err(|e| {
                crate::AlesysError::LLM(format!("Error en request Transformers: {}", e))
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
            model: model_path,
            usage: super::Usage {
                prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as usize,
                completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0)
                    as usize,
                total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as usize,
            },
        })
    }

    fn is_available(&self) -> bool {
        self.process.is_some()
    }

    fn backend_name(&self) -> &str {
        "transformers"
    }

    fn chat_stream<'a>(
        &'a self,
        messages: &'a [ChatMessage],
    ) -> BoxStream<'a, Result<StreamChunk>> {
        Box::pin(futures::stream::once(async move {
            let response = self.chat(messages).await?;
            Ok(StreamChunk {
                delta: response.content,
                finish_reason: Some("stop".to_string()),
            })
        }))
    }
}

impl Drop for TransformersEngine {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            tracing::info!("Deteniendo servidor Transformers en drop...");
            if let Ok(mut child) = process.try_lock() {
                let _ = child.kill();
            }
        }
    }
}

/// Información de disponibilidad del backend
pub fn availability_info() -> serde_json::Value {
    serde_json::json!({
        "name": "transformers",
        "description": "Backend Python con HuggingFace Transformers",
        "features": ["transformers-backend"],
        "requirements": {
            "python": ">=3.9",
            "gpu": "Opcional (CUDA/MPS)",
        },
        "capabilities": [
            "Soporte amplio de modelos HF",
            "AutoModelForCausalLM",
            "Quantización con bitsandbytes",
            "PEFT/LoRA",
        ],
        "limitations": [
            "Requiere servidor HTTP corriendo",
            "Mayor consumo de memoria que backends nativos",
            "Lento para modelos grandes en CPU",
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_availability_info() {
        let info = availability_info();
        assert_eq!(info["name"], "transformers");
    }
}
