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

/// Estado del backend LLM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LLMState {
    /// No cargado (modo search-only, 0 MB RAM)
    Unloaded,
    /// Cargado y listo para usar
    Loaded,
    /// Error al cargar
    Error,
}

/// Backend unificado que delega a la implementación correcta
/// Soporta carga lazy (no carga el modelo hasta que se llama a load())
pub enum LLMBackend {
    #[cfg(feature = "llama-cpp")]
    LlamaCpp(Option<LlamaCppEngine>),

    #[cfg(feature = "mistralrs-backend")]
    Mistralrs(Option<MistralEngine>),

    #[cfg(feature = "candle-backend")]
    Candle(Option<CandleEngine>),

    #[cfg(feature = "vllm-backend")]
    Vllm(Option<VllmEngine>),

    #[cfg(feature = "transformers-backend")]
    Transformers(Option<TransformersEngine>),

    #[cfg(feature = "http-backend")]
    Http(Option<Box<HttpLLMEngine>>),

    /// Backend sin LLM — permite modo solo búsqueda sin crash
    Noop,
}

impl LLMBackend {
    /// Crea el backend pero NO lo carga - modo lazy
    /// El modelo se cargará solo cuando se llame a `load()`
    pub async fn from_config_lazy(config: LLMConfig) -> Result<Self> {
        tracing::info!(
            "Backend {} configurado (NO cargado) - usar endpoint /api/v1/llm/load para cargar",
            config.backend
        );

        match config.backend {
            #[cfg(feature = "llama-cpp")]
            LLMBackendType::LlamaCpp => {
                tracing::info!(
                    "llama.cpp configurado (Vulkan GPU) - modelo NO cargado aún"
                );
                Ok(Self::LlamaCpp(None))
            }

            #[cfg(feature = "mistralrs-backend")]
            LLMBackendType::Mistralrs => {
                tracing::warn!(
                    "mistralrs configurado (CPU ONLY) - modelo NO cargado aún"
                );
                Ok(Self::Mistralrs(None))
            }

            #[cfg(feature = "candle-backend")]
            LLMBackendType::Candle => {
                tracing::info!("candle configurado (Rust nativo) - modelo NO cargado aún");
                Ok(Self::Candle(None))
            }

            #[cfg(feature = "vllm-backend")]
            LLMBackendType::Vllm => {
                tracing::info!("vLLM configurado (Python GPU) - modelo NO cargado aún");
                Ok(Self::Vllm(None))
            }

            #[cfg(feature = "transformers-backend")]
            LLMBackendType::Transformers => {
                tracing::info!("Transformers configurado (Python) - modelo NO cargado aún");
                Ok(Self::Transformers(None))
            }

            // --- HTTP providers ---
            #[cfg(feature = "http-backend")]
            LLMBackendType::Ollama => {
                tracing::info!("Ollama configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::OpenRouter => {
                tracing::info!("OpenRouter configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::Anthropic => {
                tracing::info!("Anthropic configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::Gemini => {
                tracing::info!("Gemini configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::Perplexity => {
                tracing::info!("Perplexity configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::Cerebras => {
                tracing::info!("Cerebras configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::Cohere => {
                tracing::info!("Cohere configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::Nvidia => {
                tracing::info!("Nvidia configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::Groq => {
                tracing::info!("Groq configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::HuggingFace => {
                tracing::info!("HuggingFace configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }
            #[cfg(feature = "http-backend")]
            LLMBackendType::GitHubModels => {
                tracing::info!("GitHubModels configurado (HTTP) - modelo NO cargado aún");
                Ok(Self::Http(None))
            }

            #[cfg(any(
                feature = "llama-cpp",
                feature = "mistralrs-backend",
                feature = "candle-backend",
                feature = "vllm-backend",
                feature = "transformers-backend"
            ))]
            LLMBackendType::Auto => {
                tracing::info!("Auto-selección configurada - modelo se cargará on-demand");
                // Auto-select pero sin cargar
                Self::auto_select(config.clone()).await
            }

            #[cfg(not(any(
                feature = "llama-cpp",
                feature = "mistralrs-backend",
                feature = "candle-backend",
                feature = "vllm-backend",
                feature = "transformers-backend"
            )))]
            LLMBackendType::Auto => Err(crate::AlesysError::LLM(
                "Auto-selección requiere al menos un backend local".to_string(),
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

    /// Carga el modelo LLM en memoria (on-demand)
    /// Esto es lo que consume RAM (600 MB - 8 GB dependiendo del modelo)
    pub async fn load(&mut self, config: &LLMConfig) -> Result<()> {
        tracing::info!("Cargando modelo LLM en memoria...");

        match self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(engine_opt) => {
                if engine_opt.is_some() {
                    tracing::info!("Modelo ya está cargado");
                    return Ok(());
                }
                let engine = LlamaCppEngine::new(config.clone()).await?;
                *engine_opt = Some(engine);
                tracing::info!("✅ Modelo llama.cpp cargado exitosamente");
            }

            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(engine_opt) => {
                if engine_opt.is_some() {
                    tracing::info!("Modelo ya está cargado");
                    return Ok(());
                }
                let mut engine = MistralEngine::new(config.clone());
                engine.load().await?;
                *engine_opt = Some(engine);
                tracing::info!("✅ Modelo mistralrs cargado exitosamente");
            }

            #[cfg(feature = "candle-backend")]
            Self::Candle(engine_opt) => {
                if engine_opt.is_some() {
                    tracing::info!("Modelo ya está cargado");
                    return Ok(());
                }
                let engine = CandleEngine::new(config.clone()).await?;
                *engine_opt = Some(engine);
                tracing::info!("✅ Modelo candle cargado exitosamente");
            }

            #[cfg(feature = "vllm-backend")]
            Self::Vllm(engine_opt) => {
                if engine_opt.is_some() {
                    tracing::info!("Modelo ya está cargado");
                    return Ok(());
                }
                let engine = VllmEngine::new(config.clone()).await?;
                *engine_opt = Some(engine);
                tracing::info!("✅ Modelo vLLM cargado exitosamente");
            }

            #[cfg(feature = "transformers-backend")]
            Self::Transformers(engine_opt) => {
                if engine_opt.is_some() {
                    tracing::info!("Modelo ya está cargado");
                    return Ok(());
                }
                let engine = TransformersEngine::new(config.clone()).await?;
                *engine_opt = Some(engine);
                tracing::info!("✅ Modelo Transformers cargado exitosamente");
            }

            #[cfg(feature = "http-backend")]
            Self::Http(engine_opt) => {
                if engine_opt.is_some() {
                    tracing::info!("Backend HTTP ya está configurado");
                    return Ok(());
                }
                // Para HTTP, necesitamos saber qué provider usar
                // Asumimos Ollama como default si no está especificado
                let engine = HttpLLMEngine::new(config.clone(), super::http::Provider::Ollama).await?;
                *engine_opt = Some(Box::new(engine));
                tracing::info!("✅ Backend HTTP configurado exitosamente");
            }

            Self::Noop => {
                return Err(crate::AlesysError::LLM(
                    "Backend Noop no puede cargar modelos".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Descarga el modelo LLM de la memoria (libera RAM)
    pub async fn unload(&mut self) -> Result<()> {
        tracing::info!("Descargando modelo LLM de la memoria...");

        match self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(engine_opt) => {
                if engine_opt.is_some() {
                    *engine_opt = None;
                    tracing::info!("✅ Modelo llama.cpp descargado - RAM liberada");
                } else {
                    tracing::info!("Modelo ya estaba descargado");
                }
            }

            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(engine_opt) => {
                if engine_opt.is_some() {
                    *engine_opt = None;
                    tracing::info!("✅ Modelo mistralrs descargado - RAM liberada");
                } else {
                    tracing::info!("Modelo ya estaba descargado");
                }
            }

            #[cfg(feature = "candle-backend")]
            Self::Candle(engine_opt) => {
                if engine_opt.is_some() {
                    *engine_opt = None;
                    tracing::info!("✅ Modelo candle descargado - RAM liberada");
                } else {
                    tracing::info!("Modelo ya estaba descargado");
                }
            }

            #[cfg(feature = "vllm-backend")]
            Self::Vllm(engine_opt) => {
                if engine_opt.is_some() {
                    *engine_opt = None;
                    tracing::info!("✅ Modelo vLLM descargado - RAM liberada");
                } else {
                    tracing::info!("Modelo ya estaba descargado");
                }
            }

            #[cfg(feature = "transformers-backend")]
            Self::Transformers(engine_opt) => {
                if engine_opt.is_some() {
                    *engine_opt = None;
                    tracing::info!("✅ Modelo Transformers descargado - RAM liberada");
                } else {
                    tracing::info!("Modelo ya estaba descargado");
                }
            }

            #[cfg(feature = "http-backend")]
            Self::Http(engine_opt) => {
                if engine_opt.is_some() {
                    *engine_opt = None;
                    tracing::info!("✅ Backend HTTP descargado");
                } else {
                    tracing::info!("Backend HTTP ya estaba descargado");
                }
            }

            Self::Noop => {
                tracing::info!("Backend Noop - no hay modelo que descargar");
            }
        }

        Ok(())
    }

    /// Verifica si el modelo está cargado en memoria
    pub fn is_loaded(&self) -> bool {
        match self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(engine_opt) => engine_opt.is_some(),

            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(engine_opt) => engine_opt.is_some(),

            #[cfg(feature = "candle-backend")]
            Self::Candle(engine_opt) => engine_opt.is_some(),

            #[cfg(feature = "vllm-backend")]
            Self::Vllm(engine_opt) => engine_opt.is_some(),

            #[cfg(feature = "transformers-backend")]
            Self::Transformers(engine_opt) => engine_opt.is_some(),

            #[cfg(feature = "http-backend")]
            Self::Http(engine_opt) => engine_opt.is_some(),

            Self::Noop => false,
        }
    }

    /// Obtiene el estado actual del backend
    pub fn state(&self) -> LLMState {
        if self.is_loaded() {
            LLMState::Loaded
        } else {
            LLMState::Unloaded
        }
    }

    /// Crea el backend y lo carga inmediatamente (comportamiento antiguo)
    /// Usar solo para compatibilidad - preferir `from_config_lazy()` + `load()`
    pub async fn from_config(config: LLMConfig) -> Result<Self> {
        let mut backend = Self::from_config_lazy(config.clone()).await?;
        backend.load(&config).await?;
        Ok(backend)
    }

    /// Crea un backend noop para modo solo búsqueda (sin LLM)
    pub fn noop() -> Self {
        Self::Noop
    }

    /// Auto-selección inteligente de backend (lazy - no carga el modelo)
    /// Nota: HTTP providers no se auto-seleccionan (requieren API keys explícitas)
    #[cfg(any(
        feature = "llama-cpp",
        feature = "mistralrs-backend",
        feature = "candle-backend",
        feature = "vllm-backend",
        feature = "transformers-backend"
    ))]
    async fn auto_select(_config: LLMConfig) -> Result<Self> {
        // Auto-selección solo determina el TIPO de backend, NO carga el modelo
        // El modelo se cargará cuando el usuario llame a load()

        #[cfg(feature = "llama-cpp")]
        {
            tracing::info!("Auto-select: llama_cpp será usado (Vulkan GPU) - modelo NO cargado aún");
            return Ok(Self::LlamaCpp(None));
        }

        #[cfg(feature = "candle-backend")]
        {
            tracing::info!("Auto-select: candle será usado (Rust nativo) - modelo NO cargado aún");
            return Ok(Self::Candle(None));
        }

        #[cfg(feature = "vllm-backend")]
        {
            tracing::info!("Auto-select: vllm será usado (Python GPU) - modelo NO cargado aún");
            return Ok(Self::Vllm(None));
        }

        #[cfg(feature = "mistralrs-backend")]
        {
            tracing::info!("Auto-select: mistralrs será usado (CPU fallback) - modelo NO cargado aún");
            return Ok(Self::Mistralrs(None));
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
    #[allow(dead_code)]
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
            Self::LlamaCpp(e) => e.as_ref()
                .ok_or_else(|| crate::AlesysError::LLM("LLM no cargado".to_string()))?
                .$method($($arg),*).await,
            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(e) => e.as_ref()
                .ok_or_else(|| crate::AlesysError::LLM("LLM no cargado".to_string()))?
                .$method($($arg),*).await,
            #[cfg(feature = "candle-backend")]
            Self::Candle(e) => e.as_ref()
                .ok_or_else(|| crate::AlesysError::LLM("LLM no cargado".to_string()))?
                .$method($($arg),*).await,
            #[cfg(feature = "vllm-backend")]
            Self::Vllm(e) => e.as_ref()
                .ok_or_else(|| crate::AlesysError::LLM("LLM no cargado".to_string()))?
                .$method($($arg),*).await,
            #[cfg(feature = "transformers-backend")]
            Self::Transformers(e) => e.as_ref()
                .ok_or_else(|| crate::AlesysError::LLM("LLM no cargado".to_string()))?
                .$method($($arg),*).await,
            #[cfg(feature = "http-backend")]
            Self::Http(e) => e.as_ref()
                .ok_or_else(|| crate::AlesysError::LLM("LLM no cargado".to_string()))?
                .$method($($arg),*).await,
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
            Self::LlamaCpp(Some(e)) => e.chat_stream(messages),
            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(Some(e)) => e.chat_stream(messages),
            #[cfg(feature = "candle-backend")]
            Self::Candle(Some(e)) => e.chat_stream(messages),
            #[cfg(feature = "vllm-backend")]
            Self::Vllm(Some(e)) => e.chat_stream(messages),
            #[cfg(feature = "transformers-backend")]
            Self::Transformers(Some(e)) => e.chat_stream(messages),
            #[cfg(feature = "http-backend")]
            Self::Http(Some(e)) => e.chat_stream(messages),
            // Si el engine es None (unloaded) o Noop, retornar error stream
            _ => Box::pin(futures::stream::once(async {
                Err(crate::AlesysError::LLM(
                    "LLM no cargado. Usar POST /api/v1/llm/load para cargar.".to_string(),
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
            Self::LlamaCpp(engine_opt) => engine_opt.as_ref().map(|e| e.is_available()).unwrap_or(false),

            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(engine_opt) => engine_opt.as_ref().map(|e| e.is_available()).unwrap_or(false),

            #[cfg(feature = "candle-backend")]
            Self::Candle(engine_opt) => engine_opt.as_ref().map(|e| e.is_available()).unwrap_or(false),

            #[cfg(feature = "vllm-backend")]
            Self::Vllm(engine_opt) => engine_opt.as_ref().map(|e| e.is_available()).unwrap_or(false),

            #[cfg(feature = "transformers-backend")]
            Self::Transformers(engine_opt) => engine_opt.as_ref().map(|e| e.is_available()).unwrap_or(false),

            #[cfg(feature = "http-backend")]
            Self::Http(engine_opt) => engine_opt.as_ref().map(|e| e.is_available()).unwrap_or(false),

            Self::Noop => false,
        }
    }

    fn backend_name(&self) -> &str {
        match self {
            #[cfg(feature = "llama-cpp")]
            Self::LlamaCpp(_) => "llama_cpp",

            #[cfg(feature = "mistralrs-backend")]
            Self::Mistralrs(_) => "mistralrs",

            #[cfg(feature = "candle-backend")]
            Self::Candle(_) => "candle",

            #[cfg(feature = "vllm-backend")]
            Self::Vllm(_) => "vllm",

            #[cfg(feature = "transformers-backend")]
            Self::Transformers(_) => "transformers",

            #[cfg(feature = "http-backend")]
            Self::Http(_) => "http",

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
