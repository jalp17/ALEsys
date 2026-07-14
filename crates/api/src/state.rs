//! Estado compartido de la aplicación

use alesys_core::{GraphRAG, SessionManager};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub graphrag: GraphRAG,
    pub session_manager: SessionManager,
}

impl AppState {
    pub async fn new(db: PgPool) -> anyhow::Result<Self> {
        let graphrag = GraphRAG::new(db.clone()).await?;
        let session_manager = SessionManager::new(db.clone());
        
        Ok(Self {
            db,
            graphrag,
            session_manager,
        })
    }
}