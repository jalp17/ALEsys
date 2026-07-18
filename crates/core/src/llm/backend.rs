//! Backend unificado de inferencia LLM
//!
//! Selecciona automáticamente entre backends según configuración y disponibilidad:
//! - llama.cpp (Vulkan/CUDA GPU) - Principal
//! - mistralrs (CPU) - Fallback
//! - candle (Rust nativo) - Experimental
//! - vLLM (Python subprocess) - GPU de alto rendimiento
//! - transformers (Python subprocess) - Modelos HF
//! - HTTP providers (Ollama, Anthropic, Gemini, Groq, etc.) - Cloud/remote

use super::{ChatMessage, ChatResponse, LLMBackendType, LLMConfig, LLMEngine, Result, StreamChunk};
use async_trait::async_trait;
use futures::stream::BoxStream;

#[cfg(feature = "llama-cpp")]
use super::llama_cpp::LlamaCppEngine;

#[cfg(feature = "mistralrs-backend")]
use super::mistral::MistralEngine;

#[cfg(feature = "candle-backend")]
use super::candle::CandleEngine;

#[cfg(feature = "vllm-backend")]
use super::vllm::VllmEngine;

#[cfg(feature = "transformers-backend")]
use super::transformers::TransformersEngine;

#[cfg(feature = "http-backend")]
use super::http::HttpLLMEngine;

/// Backend unificado que delega a la implementación correcta
pub enum LLMBackend {
    #[cfg(feature = "llama-cpp")]
    LlamaCpp(LlamaCppEngine),

    #[cfg(feature = "mistralrs-backend")]
    Mistralrs(MistralEngine),

    #[cfg(feature = "candle-backend")]
    Candle(CandleEngine),

    #[cfg(feature = "vllm-backend")]
    Vllm(VllmEngine),

    #[cfg(feature = "transformers-backend")]
    Transformers(TransformersEngine),

    #[cfg(feature = "http-backend")]
    Http(Box<HttpLLMEngine>),

    /// Backend sin LLM — permite modo solo búsqueda sin crash
    Noop,
}

impl LLMBackend {
    /// Crea el backend según la configuración
    pub async fn from_config(config: LLMConfig) -> Result<Self> {
        match config.backend {
            #[cfg(feature = "llama-cpp")]
            LLMBackendType::LlamaCpp => {
                tracing::info!(
                    "Usando backend llama.cpp (Vulkan GPU) — 150+ arquitecturas, 23 quantizaciones"
                );
                let engine = LlamaCppEngine::new(config).await?;
                Ok(Self::LlamaCpp(engine))
            }

            #[cfg(feature = "mistralrs-backend")]
            LLMBackendType::Mistralrs => {
                tracing::warn!(
                    "BACKEND MISTRALRS (CPU ONLY):\
                     - Arquitecturas: Llama, Qwen2/3, Mistral, Phi2/3, Starcoder2, Bloom, Falcon, Mamba, Rwkv\
                     - NO soportado: MoE (qwen3moe, deepseek, phimoe) — panic en indexed_moe_forward\
                     - Quantizaciones: Q4_K_M, Q8_0, F16, F32 (NO IQ4_*, IQ2_*, etc.)\
                     - Para GPU/Vulkan y 150+ arquitecturas: usar LLM_BACKEND=llama_cpp"
                );
                let mut engine = MistralEngine::new(config);
                engine.load().await?;
                Ok(Self::Mistralrs(engine))
            }

            #[cfg(feature = "candle-backend")]
            LLMBackendType::Candle => {
                tracing::info!("Usando backend candle (Rust nativo) — CUDA/Metal/CPU");
                let engine = CandleEngine::new(config).await?;
                Ok(Self::Candle(engine))
            }

            #[cfg(feature = "vllm-backend")]
            LLMBackendType::Vllm => {
                tracing::info!("Usando backend vLLM (Python subprocess) — GPU de alto rendimiento");
                let engine = VllmEngine::new(config).await?;
                Ok(Self::Vllm(engine))
            }

            #[cfg(feature = "transformers-backend")]
            LLMBackendType::Transformers => {
                tracing::info!("Usando backend Transformers (Python subprocess) — Modelos HF");
                let engine = TransformersEngine::new(config).await?;
                Ok(Self::Transformers(engine))
            }

            // --- HTTP providers ---
            #[cfg(feature = "http-backend")]
            LLMBackendType::Ollama => Self::create_http(config, super::http::Provider::Ollama),
            #[cfg(feature = "http-backend")]
            LLMBackendType::OpenRouter => Self::create_http(config, super::http::Provider::OpenRouter),
            #[cfg(feature = "http-backend")]
            LLMBackendType::Anthropic => Self::create_http(config, super::http::Provider::Anthropic),
            #[cfg(feature = "http-backend")]
            LLMBackendType::Gemini => Self::create_http(config, super::http::Provider::Gemini),
            #[cfg(feature = "http-backend")]
            LLMBackendType::Perplexity => Self::create_http(config, super::http::Provider::Perplexity),
            #[cfg(feature = "http-backend")]
            LLMBackendType::Cerebras => Self::create_http(config, super::http::Provider::Cerebras),
            #[cfg(feature = "http-backend")]
            LLMBackendType::Cohere => Self::create_http(config, super::http::Provider::Cohere),
            #[cfg(feature = "http-backend")]
            LLMBackendType::Nvidia => Self::create_http(config, super::http::Provider::Nvidia),
            #[cfg(feature = "http-backend")]
            LLMBackendType::Groq => Self::create_http(config, super::http::Provider::Groq),
            #[cfg(feature = "http-backend")]
            LLMBackendType::HuggingFace => Self::create_http(config, super::http::Provider::HuggingFace),
            #[cfg(feature = "http-backend")]
            LLMBackendType::GitHubModels => Self::create_http(config, super::http::Provider::GitHubModels),

            #[cfg(any(
                feature = "llama-cpp",
                feature = "mistralrs-backend",
                feature = "candle-backend",
                feature = "vllm-backend",
                feature = "transformers-backend"
            ))]
            LLMBackendType::Auto => {
                tracing::info!("Auto-seleccionando backend...");
                Self::auto_select(config).await
            }

            #[cfg(not(any(
                feature = "llama-cpp",
                feature = "mistralrs-backend",
                feature = "candle-backend",
                feature = "vllm-backend",
                feature = "transformers-backend"
            )))]
            LLMBackendType::Auto => Err(crate::AlesysError::LLM(
                "Auto-selección requiere al menos un backend local (llama-cpp, mistralrs, candle, vllm, transformers)".to_string(),
            )),

            #[cfg(not(feature = "llama-cpp"))]
            LLMBackendType::LlamaCpp => Err(crate::AlesysError::LLM(
                "Feature 'llama-cpp' no habilitada".to_string(),
            )),

            #[cfg(not(feature = "mistralrs-backend"))]
            LLMBackendType::Mistralrs => Err(crate::AlesysError::LLM(
                "Feature 'mistralrs-backend' no habilitada".to_string(),
            )),

            #[cfg(not(feature = "candle-backend"))]
            LLMBackendType::Candle => Err(crate::AlesysError::LLM(
                "Feature 'candle-backend' no habilitada".to_string(),
            )),

            #[cfg(not(feature = "vllm-backend"))]
            LLMBackendType::Vllm => Err(crate::AlesysError::LLM(
                "Feature 'vllm-backend' no habilitada".to_string(),
            )),

            #[cfg(not(feature = "transformers-backend"))]
            LLMBackendType::Transformers => Err(crate::AlesysError::LLM(
                "Feature 'transformers-backend' no habilitada".to_string(),
            )),
        }
    }

    /// Helper: crea un backend HTTP para un provider dado
    #[cfg(feature = "http-backend")]
    fn create_http(config: LLMConfig, provider: super::http::Provider) -> Result<Self> {
        tracing::info!(
            "Usando backend HTTP (cloud/remote) — provider={}, model={}",
            provider,
            if config.model_path.is_empty() {
                provider.default_model()
            } else {
                &config.model_path
            },
        );
        // HttpLLMEngine::new is async because it builds reqwest::Client
        // We block here since from_config is already async and client build is fast
        let engine = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { HttpLLMEngine::new(config, provider).await })
        })?;
        Ok(Self::Http(Box::new(engine)))
    }

    /// Crea un backend noop para modo solo búsqueda (sin LLM)
    pub fn noop() -> Self {
        Self::Noop
    }

    /// Auto-selección inteligente de backend
    /// Nota: HTTP providers no se auto-seleccionan (requieren API keys explícitas)
    #[cfg(any(
        feature = "llama-cpp",
        feature = "mistralrs-backend",
        feature = "candle-backend",
        feature = "vllm-backend",
        feature = "transformers-backend"
    ))]
    async fn auto_select(config: LLMConfig) -> Result<Self> {
        // Detectar GPU disponible
        let gpu = Self::detect_gpu().await;
        tracing::info!("GPU detectada: {:?}", gpu);

        // Prioridad: llama.cpp > candle > vllm > transformers > mistralrs
        // HTTP providers are not auto-selected (require explicit API keys)
        #[cfg(feature = "llama-cpp")]
        {
            tracing::info!("Intentando llama.cpp (Vulkan GPU)...");
            match LlamaCppEngine::new(config.clone()).await {
                Ok(engine) => {
                    tracing::info!("llama.cpp disponible");
                    return Ok(Self::LlamaCpp(engine));
                }
                Err(e) => {
                    tracing::warn!("llama.cpp no disponible: {}", e);
                }
            }
        }

        #[cfg(feature = "candle-backend")]
        {
            tracing::info!("Intentando candle (Rust nativo)...");
            match CandleEngine::new(config.clone()).await {
                Ok(engine) => {
                    tracing::info!("Candle disponible");
                    return Ok(Self::Candle(engine));
                }
                Err(e) => {
                    tracing::warn!("Candle no disponible: {}", e);
                }
            }
        }

        #[cfg(feature = "vllm-backend")]
        if gpu == super::config::GpuType::Cuda {
            tracing::info!("Intentando vLLM (Python GPU)...");
            match VllmEngine::new(config.clone()).await {
                Ok(engine) => {
                    tracing::info!("vLLM disponible");
                    return Ok(Self::Vllm(engine));
                }
                Err(e) => {
                    tracing::warn!("vLLM no disponible: {}", e);
                }
            }
        }

        #[cfg(feature = "mistralrs-backend")]
        {
            tracing::info!("Intentando mistralrs (CPU fallback)...");
            match MistralEngine::new(config.clone()) {
                engine => {
                    tracing::info!("Mistralrs disponible (CPU)");
                    return Ok(Self::Mistralrs(engine));
                }
            }
        }

        Err(crate::AlesysError::LLM(
            "Ningún backend LLM disponible. Habilitar al menos una feature: llama-cpp, mistralrs-backend, candle-backend, http-backend".to_string()
        ))
    }

    #[cfg(not(any(
        feature = "llama-cpp",
        feature = "mistralrs-backend",
        feature = "candle-backend",
        feature = "vllm-backend",
        feature = "transformers-backend"
    )))]
    #[allow(dead_code)] // Only reachable when no local backends + http-backend
    async fn auto_select(_config: LLMConfig) -> Result<Self> {
        Err(crate::AlesysError::LLM(
            "No LLM backend feature enabled. Habilitar al menos una: llama-cpp, mistralrs-backend, candle-backend, http-backend".to_string()
        ))
    }

    /// Detecta GPU disponible en el sistema
    #[cfg(any(
        feature = "llama-cpp",
        feature = "mistralrs-backend",
        feature = "candle-backend",
        feature = "vllm-backend",
        feature = "transformers-backend"
    ))]
    async fn detect_gpu() -> super::config::GpuType {
        // CUDA (nvidia-smi)
        if let Ok(output) = tokio::process::Command::new("nvidia-smi")
            .arg("--query-gpu=name")
            .arg("--format=csv,noheader")
            .output()
            .await
        {
            if output.status.success() && !output.stdout.is_empty() {
                let name = String::from_utf8_lossy(&output.stdout);
                tracing::info!("GPU NVIDIA detectada: {}", name.trim());
                return super::config::GpuType::Cuda;
            }
        }

        // Vulkan (vulkaninfo)
        if let Ok(output) = tokio::process::Command::new("vulkaninfo")
            .arg("--summary")
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("GPU") || stdout.contains("deviceName") {
                    tracing::info!("GPU Vulkan detectada");
                    return super::config::GpuType::Vulkan;
                }
            }
        }

        // Metal (macOS)
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = tokio::process::Command::new("system_profiler")
                .arg("SPDisplaysDataType")
                .output()
                .await
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("Metal") {
                        tracing::info!("GPU Metal detectada");
                        return super::config::GpuType::Metal;
                    }
                }
            }
        }

        tracing::info!("No se detectó GPU, usando CPU");
        super::config::GpuType::None
    }

    /// Información de disponibilidad de todos los backends
    #[allow(clippy::vec_init_then_push)]
    #[allow(unused_mut)]
    pub fn availability_info() -> Vec<serde_json::Value> {
        let mut info = vec![];

        #[cfg(feature = "llama-cpp")]
        info.push(super::llama_cpp::availability_info());

        #[cfg(feature = "candle-backend")]
        info.push(super::candle::availability_info());

        #[cfg(feature = "vllm-backend")]
        info.push(super::vllm::availability_info());

        #[cfg(feature = "transformers-backend")]
        info.push(super::transformers::availability_info());

        #[cfg(feature = "mistralrs-backend")]
        info.push(serde_json::json!({
            "name": "mistralrs",
            "description": "Backend CPU con mistral.rs",
            "features": ["mistralrs-backend"],
            "gpu_support": { "cuda": false, "vulkan": false, "cpu": true },
        }));

        #[cfg(feature = "http-backend")]
        info.push(serde_json::json!({
            "name": "http",
            "description": "Backend HTTP para proveedores cloud (Ollama, Anthropic, Gemini, Groq, etc.)",
            "features": ["http-backend"],
            "providers": ["ollama", "openrouter", "anthropic", "gemini", "perplexity", "cerebras", "cohere", "nvidia", "groq", "huggingface", "githubmodels"],
        }));

        info
    }
}

macro_rules! delegate_backend {
    ($self:expr, $method:ident($($arg:expr),*)) => {
        match $self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(e) => e.$method($($arg),*).await,
            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(e) => e.$method($($arg),*).await,
            #[cfg(feature = "candle-backend")]
            Self::Candle(e) => e.$method($($arg),*).await,
            #[cfg(feature = "vllm-backend")]
            Self::Vllm(e) => e.$method($($arg),*).await,
            #[cfg(feature = "transformers-backend")]
            Self::Transformers(e) => e.$method($($arg),*).await,
            #[cfg(feature = "http-backend")]
            Self::Http(e) => e.$method($($arg),*).await,
            Self::Noop => Err(crate::AlesysError::LLM(
                "LLM no disponible — modo solo búsqueda".to_string(),
            )),
        }
    };
}

#[async_trait]
#[allow(unused_variables)]
impl LLMEngine for LLMBackend {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse> {
        delegate_backend!(self, chat(messages))
    }

    fn chat_stream<'a>(
        &'a self,
        messages: &'a [ChatMessage],
    ) -> BoxStream<'a, Result<StreamChunk>> {
        match self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(e) => e.chat_stream(messages),
            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(e) => e.chat_stream(messages),
            #[cfg(feature = "candle-backend")]
            Self::Candle(e) => e.chat_stream(messages),
            #[cfg(feature = "vllm-backend")]
            Self::Vllm(e) => e.chat_stream(messages),
            #[cfg(feature = "transformers-backend")]
            Self::Transformers(e) => e.chat_stream(messages),
            #[cfg(feature = "http-backend")]
            Self::Http(e) => e.chat_stream(messages),
            Self::Noop => Box::pin(futures::stream::once(async {
                Err(crate::AlesysError::LLM(
                    "LLM no disponible — modo solo búsqueda".to_string(),
                ))
            })),
        }
    }

    async fn generate_code(&self, prompt: &str, language: &str) -> Result<String> {
        delegate_backend!(self, generate_code(prompt, language))
    }

    async fn extract_knowledge(&self, text: &str, schema: &str) -> Result<String> {
        delegate_backend!(self, extract_knowledge(text, schema))
    }

    fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(e) => e.is_available(),
            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(e) => e.is_available(),
            #[cfg(feature = "candle-backend")]
            Self::Candle(e) => e.is_available(),
            #[cfg(feature = "vllm-backend")]
            Self::Vllm(e) => e.is_available(),
            #[cfg(feature = "transformers-backend")]
            Self::Transformers(e) => e.is_available(),
            #[cfg(feature = "http-backend")]
            Self::Http(e) => e.is_available(),
            Self::Noop => false,
        }
    }

    fn backend_name(&self) -> &str {
        match self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(e) => e.backend_name(),
            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(e) => e.backend_name(),
            #[cfg(feature = "candle-backend")]
            Self::Candle(e) => e.backend_name(),
            #[cfg(feature = "vllm-backend")]
            Self::Vllm(e) => e.backend_name(),
            #[cfg(feature = "transformers-backend")]
            Self::Transformers(e) => e.backend_name(),
            #[cfg(feature = "http-backend")]
            Self::Http(e) => e.backend_name(),
            Self::Noop => "noop",
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(any(
        feature = "llama-cpp",
        feature = "mistralrs-backend",
        feature = "candle-backend",
        feature = "vllm-backend",
        feature = "transformers-backend",
        feature = "http-backend"
    ))]
    #[test]
    fn test_availability_info() {
        use super::*;
        let info = LLMBackend::availability_info();
        // Al menos un backend debe estar habilitado
        assert!(!info.is_empty());
    }
}
