//! ALEsys Core - Lógica de negocio principal
//! 
//! Módulos:
//! - llm: Motor de inferencia con mistralrs + ort
//! - graphrag: GraphRAG con pgvector + petgraph
//! - session: Gestión de sesiones multi-usuario
//! - sandbox: Ejecución de código (FASE AVANZADA)

pub mod llm;
pub mod graphrag;
pub mod session;
pub mod sandbox;  // FASE AVANZADA - no implementar hasta Fase 7

pub use llm::LLMEngine;
pub use graphrag::GraphRAG;
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
    Sandbox(String),  // FASE AVANZADA
}

pub type Result<T> = std::result::Result<T, AlesysError>;