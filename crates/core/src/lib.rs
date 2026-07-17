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
//! - sandbox: Ejecucion de codigo en Docker (Fase 7)

pub mod generator;
pub mod graphrag;
pub mod llm;
pub mod sandbox;
pub mod session; // Fase 3 - Gestion de sesiones multi-usuario

pub use generator::{CodeGenerator, GenerateRequest, GenerationResult};
pub use graphrag::GraphRAG;
pub use llm::LLMEngine;
pub use session::SessionManager;

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

    #[error("Sandbox error: {0}")]
    Sandbox(String),

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
