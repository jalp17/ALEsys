//! ALEsys API - Backend REST + WebSocket
//!
//! Endpoints:
//! - POST /api/chat           → Chat con GraphRAG
//! - POST /api/generate       → Generar archivos (FASE 2)
//! - POST /api/sessions       → Gestionar sesiones
//! - GET  /ws/chat            → WebSocket para streaming
//! - GET  /api/graph/stats    → Estadísticas del grafo
//! - GET  /health             → Health check
//!
//! FASE AVANZADA (Fase 7+):
//! - POST /api/execute        → Ejecutar código
//! - POST /api/modify         → Modificar archivos

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod state;
mod websocket;

use handlers::{
    chat_handler, create_session, generate_handler, graph_stats, health_handler, list_sessions,
};
use state::AppState;
use websocket::ws_chat_handler;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "alesys_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let host = std::env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("PGPORT").unwrap_or_else(|_| "5433".to_string());
        let user = std::env::var("PGUSER").unwrap_or_else(|_| "alesys".to_string());
        let password = std::env::var("PGPASSWORD").unwrap_or_else(|_| "alesys".to_string());
        let dbname = std::env::var("PGDATABASE").unwrap_or_else(|_| "alesys".to_string());
        format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, dbname
        )
    });

    let db_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let llm_config = alesys_core::llm::LLMConfig::from_env();

    let embedder_path = std::env::var("EMBEDDING_GGUF_PATH").ok();

    let state = AppState::new(db_pool, llm_config, embedder_path.as_deref()).await?;

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse().unwrap(),
            "http://localhost:8080".parse().unwrap(),
        ])
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let app = Router::new()
        .route("/api/chat", post(chat_handler))
        .route("/ws/chat", get(ws_chat_handler))
        .route("/api/generate", post(generate_handler))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions", post(create_session))
        .route("/api/graph/stats", get(graph_stats))
        .route("/health", get(health_handler))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("ALEsys API listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
