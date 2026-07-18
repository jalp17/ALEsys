//! Backend Candle (Rust nativo)
//!
//! Implementación de `LLMEngine` usando candle-core para inferencia nativa en Rust.
//! Soporta CUDA, Metal y CPU. Modelos desde HuggingFace Hub.

use super::config::{GpuType, ModelArch, QuantType};
use super::{ChatMessage, ChatResponse, LLMConfig, LLMEngine, StreamChunk};
use crate::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Backend Candle para inferencia nativa
pub struct CandleEngine {
    device: Device,
    tokenizer: Option<Tokenizer>,
    model: Option<Model>,
    config: LLMConfig,
}

/// Dispositivo de cómputo
#[derive(Debug, Clone)]
enum Device {
    Cpu,
    Cuda(usize), // device id
    Metal,       // macOS only
}

/// Modelo cargado
#[derive(Debug)]
struct Model {
    arch: ModelArch,
    weights_path: PathBuf,
    config: ModelConfig,
}

/// Configuración del modelo
#[derive(Debug, Clone)]
struct ModelConfig {
    hidden_size: usize,
    num_layers: usize,
    num_heads: usize,
    num_kv_heads: usize,
    vocab_size: usize,
    max_position_embeddings: usize,
}

impl CandleEngine {
    /// Crea una nueva instancia de CandleEngine
    pub async fn new(config: LLMConfig) -> Result<Self> {
        let device = Self::select_device(&config)?;

        tracing::info!("CandleEngine inicializado con dispositivo: {:?}", device);

        Ok(Self {
            device,
            tokenizer: None,
            model: None,
            config,
        })
    }

    /// Selecciona el dispositivo basado en configuración y disponibilidad
    fn select_device(config: &LLMConfig) -> Result<Device> {
        // Respetar configuración explícita
        match config.candle_device.as_deref() {
            Some("cpu") => return Ok(Device::Cpu),
            Some("cuda") => {
                #[cfg(feature = "cuda")]
                return Ok(Device::Cuda(0));
                #[cfg(not(feature = "cuda"))]
                return Err(crate::AlesysError::LLM(
                    "CUDA no habilitado en esta build".to_string(),
                ));
            }
            Some("metal") => {
                #[cfg(target_os = "macos")]
                return Ok(Device::Metal);
                #[cfg(not(target_os = "macos"))]
                return Err(crate::AlesysError::LLM(
                    "Metal solo disponible en macOS".to_string(),
                ));
            }
            Some(d) => {
                return Err(crate::AlesysError::LLM(format!(
                    "Dispositivo desconocido: {}",
                    d
                )))
            }
            None => {}
        }

        // Auto-detección
        #[cfg(feature = "cuda")]
        {
            if candle_core::Device::cuda_if_available(0).is_ok() {
                tracing::info!("CUDA disponible, usando GPU");
                return Ok(Device::Cuda(0));
            }
        }

        #[cfg(target_os = "macos")]
        {
            tracing::info!("macOS detectado, usando Metal");
            return Ok(Device::Metal);
        }

        tracing::info!("Usando CPU");
        Ok(Device::Cpu)
    }

    /// Carga modelo desde HuggingFace Hub o directorio local
    pub async fn load_model(&mut self, model_path: &str) -> Result<()> {
        let path = Path::new(model_path);

        if path.is_dir() {
            // Cargar desde directorio local
            self.load_from_dir(path).await
        } else if model_path.contains('/') && !model_path.ends_with(".gguf") {
            // Es un repo ID de HuggingFace
            self.load_from_hf(model_path).await
        } else {
            // Archivo GGUF - candle no soporta GGUF directamente
            // Necesita conversion o usar otro backend
            Err(crate::AlesysError::LLM(
                "Candle no soporta archivos GGUF. Usar llama-cpp o descargar modelo HF".to_string(),
            ))
        }
    }

    /// Carga desde directorio local
    async fn load_from_dir(&mut self, path: &Path) -> Result<()> {
        tracing::info!("Cargando modelo desde {}", path.display());

        // Cargar config.json
        let config_path = path.join("config.json");
        if !config_path.exists() {
            return Err(crate::AlesysError::LLM(
                "config.json no encontrado".to_string(),
            ));
        }

        let config_content = std::fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&config_content)?;

        let model_config = Self::parse_config(&config)?;
        let arch = Self::detect_arch(&config)?;

        tracing::info!("Modelo: {:?}, {} capas", arch, model_config.num_layers);

        // Cargar tokenizer
        let tokenizer_path = path.join("tokenizer.json");
        if tokenizer_path.exists() {
            let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
                .map_err(|e| crate::AlesysError::LLM(format!("Error cargando tokenizer: {}", e)))?;
            self.tokenizer = Some(tokenizer);
        }

        // Cargar pesos (safetensors)
        let weights_path = path.join("model.safetensors");
        if weights_path.exists() {
            tracing::info!("Cargando pesos desde {}", weights_path.display());
            // TODO: Implementar carga real de pesos con candle
            // Por ahora, solo almacenar la info
            self.model = Some(Model {
                arch,
                weights_path,
                config: model_config,
            });
        }

        Ok(())
    }

    /// Carga desde HuggingFace Hub
    async fn load_from_hf(&mut self, repo_id: &str) -> Result<()> {
        tracing::info!("Descargando modelo desde HuggingFace: {}", repo_id);

        let api = hf_hub::api::sync::Api::new()
            .map_err(|e| crate::AlesysError::LLM(format!("Error creando API HF: {}", e)))?;

        let repo = api.model(repo_id.to_string());

        // Descargar archivos necesarios
        let files = vec!["config.json", "tokenizer.json", "model.safetensors"];
        let mut local_path = None;

        for file in files {
            match repo.get(file) {
                Ok(path) => {
                    tracing::info!("Descargado: {}", file);
                    if file == "config.json" {
                        local_path = Some(path.parent().unwrap().to_path_buf());
                    }
                }
                Err(e) => {
                    tracing::warn!("No se pudo descargar {}: {}", file, e);
                }
            }
        }

        if let Some(path) = local_path {
            self.load_from_dir(&path).await
        } else {
            Err(crate::AlesysError::LLM(
                "No se pudieron descargar archivos del modelo".to_string(),
            ))
        }
    }

    /// Parsea config.json a ModelConfig
    fn parse_config(config: &serde_json::Value) -> Result<ModelConfig> {
        Ok(ModelConfig {
            hidden_size: config
                .get("hidden_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(4096) as usize,
            num_layers: config
                .get("num_hidden_layers")
                .and_then(|v| v.as_u64())
                .unwrap_or(32) as usize,
            num_heads: config
                .get("num_attention_heads")
                .and_then(|v| v.as_u64())
                .unwrap_or(32) as usize,
            num_kv_heads: config
                .get("num_key_value_heads")
                .and_then(|v| v.as_u64())
                .unwrap_or(32) as usize,
            vocab_size: config
                .get("vocab_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(32000) as usize,
            max_position_embeddings: config
                .get("max_position_embeddings")
                .and_then(|v| v.as_u64())
                .unwrap_or(4096) as usize,
        })
    }

    /// Detecta arquitectura del modelo
    fn detect_arch(config: &serde_json::Value) -> Result<ModelArch> {
        let arch = config
            .get("architectures")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.as_str())
            .unwrap_or("unknown");

        match arch {
            "LlamaForCausalLM" => Ok(ModelArch::Llama),
            "MistralForCausalLM" => Ok(ModelArch::Mistral),
            "Qwen2ForCausalLM" => Ok(ModelArch::Qwen2),
            "Qwen3ForCausalLM" => Ok(ModelArch::Qwen3),
            "PhiForCausalLM" => Ok(ModelArch::Phi3),
            "GemmaForCausalLM" => Ok(ModelArch::Gemma),
            _ => Err(crate::AlesysError::LLM(format!(
                "Arquitectura no soportada: {}",
                arch
            ))),
        }
    }

    /// Tokeniza texto de entrada
    fn tokenize(&self, text: &str) -> Result<Vec<u32>> {
        let tokenizer = self
            .tokenizer
            .as_ref()
            .ok_or_else(|| crate::AlesysError::LLM("Tokenizer no cargado".to_string()))?;

        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| crate::AlesysError::LLM(format!("Error tokenizando: {}", e)))?;

        Ok(encoding.get_ids().to_vec())
    }

    /// Decodifica tokens a texto
    fn decode(&self, tokens: &[u32]) -> Result<String> {
        let tokenizer = self
            .tokenizer
            .as_ref()
            .ok_or_else(|| crate::AlesysError::LLM("Tokenizer no cargado".to_string()))?;

        let text = tokenizer
            .decode(tokens, true)
            .map_err(|e| crate::AlesysError::LLM(format!("Error decodificando: {}", e)))?;

        Ok(text)
    }
}

#[async_trait]
impl LLMEngine for CandleEngine {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        let _model = self
            .model
            .as_ref()
            .ok_or_else(|| crate::AlesysError::LLM("Modelo no cargado".to_string()))?;

        // Concatenar mensajes en prompt
        let prompt: String = messages
            .iter()
            .map(|m| format!("{}: {}\n", m.role, m.content))
            .collect();

        // Tokenizar
        let tokens = self.tokenize(&prompt)?;

        tracing::info!("Generando respuesta para {} tokens", tokens.len());

        // TODO: Implementar forward pass real con candle
        // Por ahora, retornar placeholder
        let response_text = format!(
            "[Candle {}] Respuesta placeholder para: {}...",
            format!("{:?}", self.device)
                .chars()
                .take(10)
                .collect::<String>(),
            prompt.chars().take(50).collect::<String>()
        );

        Ok(ChatResponse {
            content: response_text,
            model: "candle".to_string(),
            usage: super::Usage {
                prompt_tokens: tokens.len(),
                completion_tokens: 0,
                total_tokens: tokens.len(),
            },
        })
    }

    fn is_available(&self) -> bool {
        self.model.is_some()
    }

    fn backend_name(&self) -> &str {
        "candle"
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

/// Información de disponibilidad del backend
pub fn availability_info() -> serde_json::Value {
    let info = serde_json::json!({
        "name": "candle",
        "description": "Backend Rust nativo con candle-core",
        "features": ["candle-backend"],
        "gpu_support": {
            "cuda": cfg!(feature = "cuda"),
            "metal": cfg!(target_os = "macos"),
            "cpu": true,
        },
        "model_formats": ["safetensors", "pytorch"],
        "limitations": [
            "No soporta GGUF (usar llama-cpp)",
            "No soporta MoE (Mixtral, Qwen3-MoE)",
            "Modelos f32/f16/bf16 únicamente",
        ],
    });

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let config = serde_json::json!({
            "hidden_size": 4096,
            "num_hidden_layers": 32,
            "num_attention_heads": 32,
            "vocab_size": 32000
        });

        let result = CandleEngine::parse_config(&config);
        assert!(result.is_ok());

        let cfg = result.unwrap();
        assert_eq!(cfg.hidden_size, 4096);
        assert_eq!(cfg.num_layers, 32);
    }

    #[test]
    fn test_detect_arch() {
        let config = serde_json::json!({
            "architectures": ["LlamaForCausalLM"]
        });

        let result = CandleEngine::detect_arch(&config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ModelArch::Llama);
    }

    #[test]
    fn test_availability_info() {
        let info = availability_info();
        assert_eq!(info["name"], "candle");
        assert!(info["gpu_support"]["cpu"].as_bool().unwrap());
    }
}
