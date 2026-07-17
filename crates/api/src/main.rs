//! ALEsys API - Backend REST + WebSocket
//!
//! Endpoints:
//! - POST /api/chat               -> Chat con GraphRAG + sesiones
//! - POST /api/generate           -> Generar archivos (FASE 2)
//! - GET  /api/sessions           -> Listar sesiones activas
//! - POST /api/sessions           -> Crear sesion
//! - GET  /api/sessions/:id       -> Detalle de sesion
//! - DELETE /api/sessions/:id     -> Cerrar sesion
//! - GET  /api/sessions/:id/history -> Historial de chat
//! - GET  /ws/chat                -> WebSocket para streaming
//! - GET  /api/graph/stats        -> Estadisticas del grafo
//! - GET  /health                 -> Health check

use anyhow::Result;
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod state;
mod websocket;

pub(crate) const CHAT_SYSTEM_PROMPT: &str =
    "Eres un asistente de IA experto en programación y análisis de documentos. Responde de forma clara y concisa basándote en el contexto proporcionado.";

use handlers::{
    chat_handler, create_session, delete_session, generate_handler, get_session,
    get_session_history, graph_stats, health_handler, list_sessions,
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

    // Validate critical env vars at startup
    let db_url_result = std::env::var("DATABASE_URL");
    let pg_result = std::env::var("PGHOST");
    if db_url_result.is_err() && pg_result.is_err() {
        tracing::warn!(
            "Neither DATABASE_URL nor PGHOST set — usando defaults de docker-compose"
        );
    }

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

    let cors_origins: Vec<_> = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173,http://localhost:8080".into())
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(cors_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let timeout_secs: u64 = std::env::var("REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);

    let app = Router::new()
        .route("/api/chat", post(chat_handler))
        .route("/ws/chat", get(ws_chat_handler))
        .route("/api/generate", post(generate_handler))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions", post(create_session))
        .route("/api/sessions/:id", get(get_session))
        .route("/api/sessions/:id", delete(delete_session))
        .route("/api/sessions/:id/history", get(get_session_history))
        .route("/api/graph/stats", get(graph_stats))
        .route("/health", get(health_handler))
        .with_state(state)
        .layer(cors)
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(timeout_secs),
        ))
        .layer(TraceLayer::new_for_http());

    let addr = std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("ALEsys API listening on {} (timeout={}s)", addr, timeout_secs);

    let shutdown_signal = async {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Shutdown signal received, draining connections...");
        // Give in-flight requests time to complete
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    tracing::info!("ALEsys API stopped");
    Ok(())
}
