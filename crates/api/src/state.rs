//! Estado compartido de la aplicación

use alesys_core::llm::{LLMBackend, LLMConfig, ONNXEmbedder};
use alesys_core::{GraphRAG, SessionManager};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub _db: PgPool,
    pub graphrag: Arc<GraphRAG>,
    pub _session_manager: SessionManager,
    pub llm_engine: Arc<LLMBackend>,
    pub embedder: Arc<ONNXEmbedder>,
}

impl AppState {
    pub async fn new(
        db: PgPool,
        llm_config: LLMConfig,
        embedder_path: Option<&str>,
    ) -> anyhow::Result<Self> {
        let graphrag = Arc::new(GraphRAG::new(db.clone()).await?);
        let session_manager = SessionManager::new(db.clone());

        // Inicializar LLM engine (selecciona backend automáticamente)
        let llm_engine = match LLMBackend::from_config(llm_config).await {
            Ok(engine) => Arc::new(engine),
            Err(e) => {
                tracing::warn!("No se pudo cargar modelo LLM: {}. Modo solo búsqueda.", e);
                // Crear un backend sin modelo para modo solo búsqueda
                // Esto permite que la API funcione sin LLM
                Arc::new(LLMBackend::from_config(LLMConfig::default()).await?)
            }
        };

        // Inicializar embedder
        let mut embedder = ONNXEmbedder::new();
        if let Some(path) = embedder_path {
            if let Err(e) = embedder.load(path) {
                tracing::warn!(
                    "No se pudo cargar modelo de embeddings: {}. Usando embeddings dummy.",
                    e
                );
            }
        }
        let embedder = Arc::new(embedder);

        Ok(Self {
            _db: db,
            graphrag,
            _session_manager: session_manager,
            llm_engine,
            embedder,
        })
    }
}
