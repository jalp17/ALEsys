//! Estado compartido de la aplicación

use alesys_core::llm::{
    ChatMessage, ChatResponse, LLMBackend, LLMConfig, LLMEngine, ONNXEmbedder, StreamChunk,
};
use alesys_core::{GraphRAG, SessionManager, AgentManager};
use crate::auth::AuthState;
use futures::stream::BoxStream;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// LLM Queue — limits concurrent inferences via semaphore
#[derive(Clone)]
pub struct LLMQueue {
    semaphore: Arc<Semaphore>,
    engine: Arc<LLMBackend>,
}

impl LLMQueue {
    pub fn new(engine: Arc<LLMBackend>, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            engine,
        }
    }

    pub async fn chat(&self, messages: &[ChatMessage]) -> alesys_core::Result<ChatResponse> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| alesys_core::AlesysError::LLM(format!("Semaphore closed: {}", e)))?;
        self.engine.chat(messages).await
    }

    pub fn chat_stream<'a>(
        &'a self,
        messages: &'a [ChatMessage],
    ) -> BoxStream<'a, alesys_core::Result<StreamChunk>> {
        self.engine.chat_stream(messages)
    }

    #[allow(dead_code)] // Used by health endpoint + metrics
    pub fn is_available(&self) -> bool {
        self.engine.is_available()
    }

    #[allow(dead_code)]
    pub fn backend_name(&self) -> &str {
        self.engine.backend_name()
    }

    #[allow(dead_code)]
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub db: PgPool,
    pub graphrag: Arc<GraphRAG>,
    pub session_manager: SessionManager,
    pub llm_engine: Arc<LLMBackend>,
    pub llm_queue: LLMQueue,
    pub llm_config: LLMConfig,
    pub embedder: Arc<ONNXEmbedder>,
    pub agent_manager: Arc<AgentManager>,
    pub auth_state: Arc<AuthState>,
}

impl AppState {
    pub async fn new(
        db: PgPool,
        llm_config: LLMConfig,
        embedder_path: Option<&str>,
    ) -> anyhow::Result<Self> {
        let graphrag = Arc::new(GraphRAG::new(db.clone()).await?);
        let session_manager = SessionManager::new(db.clone());

        let llm_engine = match LLMBackend::from_config(llm_config.clone()).await {
            Ok(engine) => Arc::new(engine),
            Err(e) => {
                tracing::warn!("No se pudo cargar modelo LLM: {}. Modo solo búsqueda.", e);
                Arc::new(LLMBackend::noop())
            }
        };

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

        let llm_queue = LLMQueue::new(llm_engine.clone(), 4);

        let state = Self {
            db,
            graphrag,
            session_manager,
            llm_engine,
            llm_queue,
            llm_config,
            embedder,
            agent_manager: Arc::new(AgentManager::new()),
            auth_state: Arc::new(AuthState::new()),
        };

        tracing::info!(
            "Estado inicial: LLM={} (backend={}), Embedder={}, DB=connected",
            state.llm_engine.is_available(),
            state.llm_engine.backend_name(),
            state.embedder.is_available(),
        );

        Ok(state)
    }
}
