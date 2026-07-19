//! ALEsys Core - Logica de negocio principal
//!
//! Modulos:
//! - llm: Motor de inferencia con multi-backend (llama.cpp, mistralrs, candle, vLLM, transformers)
//! - graphrag: GraphRAG con pgvector + petgraph
//! - generator: Servicio de generacion de codigo (Fase 2)
//!   - engine: CodeGenerator con LLM compartido
//!   - templates: PromptTemplate por lenguaje
//!   - validation: SyntaxValidator post-generacion
//! - session: Gestion de sesiones multi-usuario (Fase 3)
//! - executor: Ejecucion local de subprocesos con limites (Fase 7+)
//! - fs_ops: Operaciones locales de archivos (Fase 7+)
//! - automation: Automatizacion local (LaTeX, Markdown, red)

pub mod generator;
pub mod graphrag;
pub mod llm;
pub mod session;
pub mod executor;
pub mod fs_ops;
pub mod automation;
pub mod agent;
pub mod plugin;
pub mod voice;
pub mod multimodal;

pub use generator::{CodeGenerator, GenerateRequest, GenerationResult};
pub use graphrag::GraphRAG;
pub use llm::LLMEngine;
pub use session::SessionManager;
pub use agent::{AgentManager, AgentInfo, AgentStatus};
pub use plugin::{PluginManager, PluginMetadata, PluginResult};

/// Error types del core
#[derive(Debug, thiserror::Error)]
pub enum AlesysError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("LLM error: {0}")]
    LLM(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("API error: {0}")]
    ApiError(String),
}

impl From<anyhow::Error> for AlesysError {
    fn from(err: anyhow::Error) -> Self {
        AlesysError::LLM(err.to_string())
    }
}

impl From<toml::de::Error> for AlesysError {
    fn from(err: toml::de::Error) -> Self {
        AlesysError::LLM(err.to_string())
    }
}

impl From<serde_json::Error> for AlesysError {
    fn from(err: serde_json::Error) -> Self {
        AlesysError::LLM(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AlesysError>;
