//! Configuración unificada para LLM engines

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Backend de inferencia LLM disponible
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LLMBackendType {
    LlamaCpp,
    Mistralrs,
    Candle,
    Vllm,
    Transformers,
    /// Backend HTTP para proveedores cloud/remote (Ollama, Anthropic, Gemini, Groq, etc.)
    #[cfg(feature = "http-backend")]
    #[serde(rename = "ollama")]
    Ollama,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "anthropic")]
    Anthropic,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "gemini")]
    Gemini,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "perplexity")]
    Perplexity,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "cerebras")]
    Cerebras,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "cohere")]
    Cohere,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "nvidia")]
    Nvidia,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "groq")]
    Groq,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "huggingface")]
    HuggingFace,
    #[cfg(feature = "http-backend")]
    #[serde(rename = "githubmodels")]
    GitHubModels,
    #[default]
    Auto,
}

impl std::fmt::Display for LLMBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LlamaCpp => write!(f, "llama_cpp"),
            Self::Mistralrs => write!(f, "mistralrs"),
            Self::Candle => write!(f, "candle"),
            Self::Vllm => write!(f, "vllm"),
            Self::Transformers => write!(f, "transformers"),
            #[cfg(feature = "http-backend")]
            Self::Ollama => write!(f, "ollama"),
            #[cfg(feature = "http-backend")]
            Self::OpenRouter => write!(f, "openrouter"),
            #[cfg(feature = "http-backend")]
            Self::Anthropic => write!(f, "anthropic"),
            #[cfg(feature = "http-backend")]
            Self::Gemini => write!(f, "gemini"),
            #[cfg(feature = "http-backend")]
            Self::Perplexity => write!(f, "perplexity"),
            #[cfg(feature = "http-backend")]
            Self::Cerebras => write!(f, "cerebras"),
            #[cfg(feature = "http-backend")]
            Self::Cohere => write!(f, "cohere"),
            #[cfg(feature = "http-backend")]
            Self::Nvidia => write!(f, "nvidia"),
            #[cfg(feature = "http-backend")]
            Self::Groq => write!(f, "groq"),
            #[cfg(feature = "http-backend")]
            Self::HuggingFace => write!(f, "huggingface"),
            #[cfg(feature = "http-backend")]
            Self::GitHubModels => write!(f, "githubmodels"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl std::str::FromStr for LLMBackendType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "llama_cpp" | "llamacpp" | "llama.cpp" => Ok(Self::LlamaCpp),
            "mistralrs" | "mistral" => Ok(Self::Mistralrs),
            "candle" => Ok(Self::Candle),
            "vllm" => Ok(Self::Vllm),
            "transformers" | "hf" => Ok(Self::Transformers),
            #[cfg(feature = "http-backend")]
            "ollama" => Ok(Self::Ollama),
            #[cfg(feature = "http-backend")]
            "openrouter" => Ok(Self::OpenRouter),
            #[cfg(feature = "http-backend")]
            "anthropic" | "claude" => Ok(Self::Anthropic),
            #[cfg(feature = "http-backend")]
            "gemini" | "google" => Ok(Self::Gemini),
            #[cfg(feature = "http-backend")]
            "perplexity" | "pplx" => Ok(Self::Perplexity),
            #[cfg(feature = "http-backend")]
            "cerebras" => Ok(Self::Cerebras),
            #[cfg(feature = "http-backend")]
            "cohere" | "command" => Ok(Self::Cohere),
            #[cfg(feature = "http-backend")]
            "nvidia" | "nim" => Ok(Self::Nvidia),
            #[cfg(feature = "http-backend")]
            "groq" => Ok(Self::Groq),
            #[cfg(feature = "http-backend")]
            "huggingface" | "hfinference" => Ok(Self::HuggingFace),
            #[cfg(feature = "http-backend")]
            "githubmodels" | "github" => Ok(Self::GitHubModels),
            "auto" | "" => Ok(Self::Auto),
            _ => Err(anyhow::anyhow!(
                "Backend desconocido: '{}'. Opciones: llama_cpp, mistralrs, candle, vllm, transformers, auto",
                s
            )),
        }
    }
}

#[cfg(feature = "http-backend")]
impl LLMBackendType {
    /// Convierte un LLMBackendType HTTP al Provider enum de http.rs
    pub fn to_http_provider(&self) -> Option<super::http::Provider> {
        match self {
            Self::Ollama => Some(super::http::Provider::Ollama),
            Self::OpenRouter => Some(super::http::Provider::OpenRouter),
            Self::Anthropic => Some(super::http::Provider::Anthropic),
            Self::Gemini => Some(super::http::Provider::Gemini),
            Self::Perplexity => Some(super::http::Provider::Perplexity),
            Self::Cerebras => Some(super::http::Provider::Cerebras),
            Self::Cohere => Some(super::http::Provider::Cohere),
            Self::Nvidia => Some(super::http::Provider::Nvidia),
            Self::Groq => Some(super::http::Provider::Groq),
            Self::HuggingFace => Some(super::http::Provider::HuggingFace),
            Self::GitHubModels => Some(super::http::Provider::GitHubModels),
            _ => None,
        }
    }

    /// Returns true if this backend type is an HTTP cloud/remote provider
    pub fn is_http_provider(&self) -> bool {
        matches!(
            self,
            Self::Ollama
                | Self::OpenRouter
                | Self::Anthropic
                | Self::Gemini
                | Self::Perplexity
                | Self::Cerebras
                | Self::Cohere
                | Self::Nvidia
                | Self::Groq
                | Self::HuggingFace
                | Self::GitHubModels
        )
    }
}

/// Configuración del motor LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Ruta al modelo GGUF o repo HF
    pub model_path: String,

    /// Backend a utilizar (Auto selecciona automáticamente)
    #[serde(default)]
    pub backend: LLMBackendType,

    /// Máximo de tokens a generar
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// Temperatura para muestreo
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Top-p para muestreo
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// Tamaño del contexto (tokens)
    #[serde(default = "default_context_size")]
    pub context_size: usize,

    // --- Parámetros específicos de llama.cpp ---
    /// Capas a offloadar a GPU (99 = todas)
    #[serde(default = "default_gpu_layers")]
    pub gpu_layers: u32,

    /// Número de threads para inferencia (None = auto)
    #[serde(default)]
    pub n_threads: Option<u32>,

    // --- Parámetros específicos de mistralrs ---
    /// Ruta al chat template JSON
    #[serde(default)]
    pub chat_template: Option<String>,

    // --- Parámetros de Python backends ---
    /// Puerto para servidor Python (vLLM/Transformers)
    #[serde(default = "default_server_port")]
    pub server_port: u16,

    /// Path al ejecutable Python
    #[serde(default)]
    pub python_path: Option<String>,

    /// Directorio virtualenv
    #[serde(default)]
    pub venv_path: Option<String>,

    // --- Parámetros específicos de Candle ---
    /// Dispositivo Candle (auto, cpu, cuda:0, metal)
    #[serde(default)]
    pub candle_device: Option<String>,

    /// Tipo de datos para Candle (auto, f32, f16, bf16)
    #[serde(default)]
    pub candle_dtype: Option<String>,

    // --- Parámetros específicos de vLLM ---
    /// GPU memory utilization (0.0 - 1.0)
    #[serde(default = "default_gpu_memory_utilization")]
    pub gpu_memory_utilization: f32,

    /// Max model length (override auto-detect)
    #[serde(default)]
    pub max_model_len: Option<usize>,

    /// Tensor parallel size (multi-GPU)
    #[serde(default = "default_tensor_parallel_size")]
    pub tensor_parallel_size: u32,
}

fn default_max_tokens() -> usize {
    2048
}
fn default_temperature() -> f32 {
    0.7
}
fn default_top_p() -> f32 {
    0.9
}
fn default_context_size() -> usize {
    4096
}
fn default_gpu_layers() -> u32 {
    99
}
fn default_server_port() -> u16 {
    8000
}
fn default_gpu_memory_utilization() -> f32 {
    0.9
}
fn default_tensor_parallel_size() -> u32 {
    1
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            backend: LLMBackendType::default(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            top_p: default_top_p(),
            context_size: default_context_size(),
            gpu_layers: default_gpu_layers(),
            n_threads: None,
            chat_template: None,
            server_port: default_server_port(),
            python_path: None,
            venv_path: None,
            candle_device: None,
            candle_dtype: None,
            gpu_memory_utilization: default_gpu_memory_utilization(),
            max_model_len: None,
            tensor_parallel_size: default_tensor_parallel_size(),
        }
    }
}

impl LLMConfig {
    /// Construye config desde variables de entorno
    pub fn from_env() -> Self {
        let backend: LLMBackendType = std::env::var("LLM_BACKEND")
            .unwrap_or_else(|_| "auto".to_string())
            .parse()
            .unwrap_or_default();

        let temperature = std::env::var("LLM_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_temperature());

        let top_p = std::env::var("LLM_TOP_P")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_top_p());

        let gpu_memory_utilization = std::env::var("LLM_GPU_MEMORY_UTILIZATION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_gpu_memory_utilization());

        // Range validation
        let temperature = if !(0.0..=2.0).contains(&temperature) {
            tracing::warn!(
                "LLM_TEMPERATURE={} fuera de rango [0.0, 2.0], usando default",
                temperature
            );
            default_temperature()
        } else {
            temperature
        };

        let top_p = if !(0.0..=1.0).contains(&top_p) {
            tracing::warn!(
                "LLM_TOP_P={} fuera de rango [0.0, 1.0], usando default",
                top_p
            );
            default_top_p()
        } else {
            top_p
        };

        let gpu_memory_utilization = if !(0.0..=1.0).contains(&gpu_memory_utilization) {
            tracing::warn!(
                "LLM_GPU_MEMORY_UTILIZATION={} fuera de rango [0.0, 1.0], usando default",
                gpu_memory_utilization
            );
            default_gpu_memory_utilization()
        } else {
            gpu_memory_utilization
        };

        let model_path = std::env::var("LLM_MODEL_PATH").unwrap_or_default();
        if model_path.is_empty() && backend.to_string() != "auto" {
            tracing::warn!(
                "LLM_MODEL_PATH vacío con backend={:?} — el modelo no se cargará hasta configurar la ruta",
                backend
            );
        }

        Self {
            model_path,
            backend,
            max_tokens: std::env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_max_tokens()),
            temperature,
            top_p,
            context_size: std::env::var("LLM_CONTEXT_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_context_size()),
            gpu_layers: std::env::var("LLM_GPU_LAYERS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_gpu_layers()),
            n_threads: std::env::var("LLM_N_THREADS")
                .ok()
                .and_then(|v| v.parse().ok()),
            chat_template: std::env::var("LLM_CHAT_TEMPLATE").ok(),
            server_port: std::env::var("LLM_SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_server_port()),
            python_path: std::env::var("LLM_PYTHON_PATH").ok(),
            venv_path: std::env::var("LLM_VENV_PATH").ok(),
            candle_device: std::env::var("LLM_CANDLE_DEVICE").ok(),
            candle_dtype: std::env::var("LLM_CANDLE_DTYPE").ok(),
            gpu_memory_utilization,
            max_model_len: std::env::var("LLM_MAX_MODEL_LEN")
                .ok()
                .and_then(|v| v.parse().ok()),
            tensor_parallel_size: std::env::var("LLM_TENSOR_PARALLEL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_tensor_parallel_size()),
        }
    }
}

// =============================================================================
// Model Registry Types
// =============================================================================

/// Configuración de build de un backend desde models.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendBuildConfig {
    pub name: String,
    pub enabled: bool,
    pub gpu_backends: Vec<String>,
    pub cpu_support: bool,
    pub architectures: Vec<String>,
    pub quantizations: Vec<String>,
    pub build_mode: BuildMode,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub python: Option<PythonConfig>,
    #[serde(default)]
    pub limitations: Vec<String>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub server_port: Option<u16>,
    #[serde(default)]
    pub health_endpoint: Option<String>,
}

/// Modo de build del backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BuildMode {
    CiPrebuilt,
    LocalCompile,
    PythonInstall,
}

/// Configuración de dependencias Python
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    pub version: String,
    pub packages: Vec<String>,
    #[serde(default)]
    pub index_url: Option<String>,
    #[serde(default)]
    pub uv: bool,
}

/// Registry centralizado de modelos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub backends: HashMap<String, BackendBuildConfig>,
    pub tests: Option<TestModels>,
}

/// Modelos para tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestModels {
    pub moe_model: Option<String>,
    pub small_model: Option<String>,
    pub small_embeddings: Option<String>,
}

/// Información detectada de un modelo
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub arch: ModelArch,
    pub quant: QuantType,
    pub is_moe: bool,
    pub parameter_count: Option<f64>, // en billions
}

/// Arquitectura del modelo
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelArch {
    Llama,
    Mistral,
    Mixtral,
    Qwen2,
    Qwen3,
    Qwen3MoE,
    Phi2,
    Phi3,
    Gemma,
    Gemma2,
    DeepSeek,
    DeepSeekMoE,
    Starcoder,
    Bloom,
    Falcon,
    Mamba,
    Rwkv,
    Bert,
    T5,
    Unknown(String),
}

impl ModelArch {
    pub fn is_mistralrs_compatible(&self) -> bool {
        matches!(
            self,
            Self::Llama
                | Self::Mistral
                | Self::Qwen2
                | Self::Qwen3
                | Self::Phi2
                | Self::Phi3
                | Self::Starcoder
                | Self::Bloom
                | Self::Falcon
                | Self::Mamba
                | Self::Rwkv
        )
    }

    pub fn is_candle_compatible(&self) -> bool {
        matches!(
            self,
            Self::Llama
                | Self::Mistral
                | Self::Qwen2
                | Self::Qwen3
                | Self::Phi3
                | Self::Gemma
                | Self::Gemma2
                | Self::Bert
                | Self::T5
        )
    }
}

/// Tipo de cuantización
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuantType {
    F32,
    F16,
    BF16,
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
    Q8_0,
    Q2K,
    Q3K,
    Q4K,
    Q5K,
    Q6K,
    IQ1S,
    IQ1M,
    IQ2XXS,
    IQ2XS,
    IQ2S,
    IQ3XXS,
    IQ3S,
    IQ4NL,
    IQ4XS,
    AWQ,
    GPTQ,
    FP8,
    Unknown(String),
}

/// GPU detectada
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuType {
    Cuda,
    Rocm,
    Metal,
    Vulkan,
    None,
}

/// Extrae entidades de un texto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeExtraction {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub origin: String,
    pub destination: String,
    pub relation_type: String,
}
