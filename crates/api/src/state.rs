//! Estado compartido de la aplicación

use crate::auth::AuthState;
use alesys_core::db::execute_sql;
use alesys_core::ingestion::IngestionConfig;
use alesys_core::llm::{
    ChatMessage, ChatResponse, LLMBackend, LLMConfig, LLMEngine, ONNXEmbedder, StreamChunk,
    LLMState,
};
use alesys_core::{GraphRAG, SessionManager, AgentManager, PluginManager};
use futures::stream::BoxStream;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};

/// LLM Queue — limits concurrent inferences via semaphore
#[derive(Clone)]
pub struct LLMQueue {
    semaphore: Arc<Semaphore>,
    engine: Arc<RwLock<LLMBackend>>,
}

impl LLMQueue {
    pub fn new(engine: Arc<RwLock<LLMBackend>>, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            engine,
        }
    }

    /// Crea una queue para backend lazy (aún no cargado)
    pub fn new_lazy(engine: Arc<RwLock<LLMBackend>>, max_concurrent: usize) -> Self {
        Self::new(engine, max_concurrent)
    }

    pub async fn chat(&self, messages: &[ChatMessage]) -> alesys_core::Result<ChatResponse> {
        // Verificar si el modelo está cargado
        {
            let engine = self.engine.read().await;
            if !engine.is_loaded() {
                return Err(alesys_core::AlesysError::LLM(
                    "LLM no cargado. Usar POST /api/v1/llm/load para cargar el modelo.".to_string()
                ));
            }
            if !engine.is_available() {
                return Err(alesys_core::AlesysError::LLM(
                    "LLM no está disponible. Verificar configuración.".to_string()
                ));
            }
        }

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| alesys_core::AlesysError::LLM(format!("Semaphore closed: {}", e)))?;
        
        let engine = self.engine.read().await;
        engine.chat(messages).await
    }

    pub fn chat_stream<'a>(
        &'a self,
        messages: &'a [ChatMessage],
    ) -> BoxStream<'a, alesys_core::Result<StreamChunk>> {
        // Streaming con verificación lazy - implementación simplificada
        // Retorna un stream que verifica el estado antes de proceder
        use futures::stream::once;
        
        // Clonar referencias para el stream async
        let engine = self.engine.clone();
        let messages = messages.to_vec();
        
        Box::pin(once(async move {
            // Verificar si está cargado
            let eng = engine.read().await;
            if !eng.is_loaded() {
                return Err(alesys_core::AlesysError::LLM(
                    "LLM no cargado. Usar POST /api/v1/llm/load para cargar.".to_string()
                ));
            }
            if !eng.is_available() {
                return Err(alesys_core::AlesysError::LLM(
                    "LLM no disponible".to_string()
                ));
            }
            
            // En una implementación completa, aquí se obtendría el stream real
            // Por ahora, retornamos un chunk vacío como placeholder
            Ok(StreamChunk {
                delta: "[Lazy load] Streaming no implementado aún. Usar chat() no-streaming.".to_string(),
                finish_reason: Some("stop".to_string()),
            })
        }))
    }

    #[allow(dead_code)] // Used by health endpoint + metrics
    pub fn is_available(&self) -> bool {
        false // Siempre false para lazy - verificar is_loaded() en su lugar
    }

    /// Verifica si el modelo está cargado
    pub async fn is_loaded(&self) -> bool {
        let engine = self.engine.read().await;
        engine.is_loaded()
    }

    /// Carga el modelo
    pub async fn load(&self, config: &LLMConfig) -> alesys_core::Result<()> {
        let mut engine = self.engine.write().await;
        engine.load(config).await
            .map_err(|e| alesys_core::AlesysError::LLM(format!("Error cargando modelo: {}", e)))
    }

    /// Descarga el modelo
    pub async fn unload(&self) -> alesys_core::Result<()> {
        let mut engine = self.engine.write().await;
        engine.unload().await
            .map_err(|e| alesys_core::AlesysError::LLM(format!("Error descargando modelo: {}", e)))
    }

    /// Obtiene el estado del LLM
    pub async fn state(&self) -> alesys_core::llm::LLMState {
        let engine = self.engine.read().await;
        engine.state()
    }

    #[allow(dead_code)]
    pub fn backend_name(&self) -> String {
        // Para síncrono, retornar el nombre sin await (usar en contextos no-async)
        "unknown".to_string()
    }

    /// Nombre del backend (async)
    pub async fn backend_name_async(&self) -> String {
        let engine = self.engine.read().await;
        engine.backend_name().to_string()
    }

    #[allow(dead_code)]
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// Per-user ingestion concurrency limiter
#[derive(Clone)]
pub struct IngestionSemaphore {
    inner: Arc<RwLock<HashMap<String, usize>>>,
    max_per_user: usize,
}

impl IngestionSemaphore {
    pub fn new(max_per_user: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            max_per_user,
        }
    }

    pub async fn acquire(&self, user_id: &str) -> IngestionToken {
        loop {
            let granted = {
                let mut guard = self.inner.write().await;
                let count = guard.entry(user_id.to_string()).or_insert(0);
                if *count < self.max_per_user {
                    *count += 1;
                    true
                } else {
                    false
                }
            };

            if granted {
                return IngestionToken {
                    user_id: user_id.to_string(),
                    inner: self.inner.clone(),
                };
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}

pub struct IngestionToken {
    user_id: String,
    inner: Arc<RwLock<HashMap<String, usize>>>,
}

impl Drop for IngestionToken {
    fn drop(&mut self) {
        let inner = self.inner.clone();
        let user_id = self.user_id.clone();
        tokio::spawn(async move {
            let mut guard = inner.write().await;
            if let Some(count) = guard.get_mut(&user_id) {
                *count -= 1;
                if *count == 0 {
                    guard.remove(&user_id);
                }
            }
        });
    }
}

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub db: PgPool,
    pub graphrag: Arc<GraphRAG>,
    pub session_manager: SessionManager,
    pub llm_engine: Arc<RwLock<LLMBackend>>,
    pub llm_queue: LLMQueue,
    pub llm_config: LLMConfig,
    pub embedder: Arc<ONNXEmbedder>,
    pub agent_manager: Arc<AgentManager>,
    pub auth_state: Arc<AuthState>,
    pub plugin_manager: Arc<PluginManager>,
    pub ingestion_config: IngestionConfig,
    pub ingestion_semaphore: IngestionSemaphore,
}

impl AppState {
    pub async fn new(
        db: PgPool,
        llm_config: LLMConfig,
        embedder_path: Option<&str>,
        ingestion_config: IngestionConfig,
    ) -> anyhow::Result<Self> {
        let graphrag = Arc::new(GraphRAG::new(db.clone()).await?);
        let session_manager = SessionManager::new(db.clone());

        // LAZY LOAD: Crear backend pero NO cargar el modelo
        // El modelo se cargará solo cuando el usuario lo solicite explícitamente
        let llm_engine = match LLMBackend::from_config_lazy(llm_config.clone()).await {
            Ok(engine) => {
                tracing::info!(
                    "LLM backend configurado (NO cargado): {} - usar /api/v1/llm/load para cargar",
                    llm_config.backend
                );
                Arc::new(RwLock::new(engine))
            }
            Err(e) => {
                tracing::warn!("No se pudo configurar backend LLM: {}. Modo solo búsqueda.", e);
                Arc::new(RwLock::new(LLMBackend::noop()))
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

        // LLM Queue con referencia al engine lazy
        let llm_queue = LLMQueue::new_lazy(llm_engine.clone(), 4);

        // Initialize plugin manager
        let plugin_dir = std::env::var("PLUGIN_DIR")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
                format!("{}/.alesys/plugins", home)
            });
        let plugin_manager = Arc::new(PluginManager::new(
            std::path::PathBuf::from(plugin_dir),
            &db,
        ));

        let ingestion_migration = include_str!("../../../crates/core/migrations/20260726_create_ingestion_jobs.sql");
        if let Err(e) = execute_sql(&db, ingestion_migration).await {
            tracing::warn!("Ingestion jobs migration failed: {}", e);
        }

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
            plugin_manager,
            ingestion_config,
            ingestion_semaphore: IngestionSemaphore::new(5),
        };

        tracing::info!(
            "Estado inicial: LLM={}(backend={}), Embedder={}, DB=connected",
            {
                let engine = state.llm_engine.read().await;
                if engine.is_loaded() { "loaded" } else { "unloaded" }
            },
            {
                let engine = state.llm_engine.read().await;
                engine.backend_name().to_string()
            },
            state.embedder.is_available(),
        );

        Ok(state)
    }
}
