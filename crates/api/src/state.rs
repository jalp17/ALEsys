//! Estado compartido de la aplicación

use alesys_core::llm::{LLMBackend, LLMConfig, LLMEngine, ONNXEmbedder};
use alesys_core::{GraphRAG, SessionManager};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)] // Mantenido para keep-alive del pool; usado via SessionManager
    pub db: PgPool,
    pub graphrag: Arc<GraphRAG>,
    pub session_manager: SessionManager,
    pub llm_engine: Arc<LLMBackend>,
    pub llm_config: LLMConfig,
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
        let llm_engine = match LLMBackend::from_config(llm_config.clone()).await {
            Ok(engine) => Arc::new(engine),
            Err(e) => {
                tracing::warn!("No se pudo cargar modelo LLM: {}. Modo solo búsqueda.", e);
                Arc::new(LLMBackend::noop())
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

        let state = Self {
            db,
            graphrag,
            session_manager,
            llm_engine,
            llm_config,
            embedder,
        };

        // Log startup health status
        tracing::info!(
            "Estado inicial: LLM={} (backend={}), Embedder={}, DB=connected",
            state.llm_engine.is_available(),
            state.llm_engine.backend_name(),
            state.embedder.is_available(),
        );

        Ok(state)
    }
}
