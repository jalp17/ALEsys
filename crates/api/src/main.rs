//! ALEsys API - Backend REST + WebSocket
//! 
//! Endpoints:
//! - POST /api/chat           → Chat con GraphRAG
//! - POST /api/generate       → Generar archivos (FASE 2)
//! - POST /api/sessions       → Gestionar sesiones
//! - GET  /ws/chat            → WebSocket para streaming
//! 
//! FASE AVANZADA (Fase 7+):
//! - POST /api/execute        → Ejecutar código
//! - POST /api/modify         → Modificar archivos

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
    extract::{State, ws::WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod websocket;
mod state;

use state::AppState;
use handlers::{chat_handler, generate_handler, list_sessions, create_session};
use websocket::ws_chat_handler;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "alesys_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load environment
    dotenvy::dotenv().ok();
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL debe estar configurado");
    
    let db_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Initialize state
    let state = AppState::new(db_pool).await?;
    
    // CORS (configurar según el frontend)
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<Any>().unwrap())
        .allow_origin("http://localhost:8080".parse::<Any>().unwrap())
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);
    
    // Build router
    let app = Router::new()
        // Chat
        .route("/api/chat", post(chat_handler))
        .route("/ws/chat", get(ws_chat_handler))
        
        // Generación de archivos (FASE 2)
        .route("/api/generate", post(generate_handler))
        
        // Sesiones
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions", post(create_session))
        
        // FASE AVANZADA (Fase 7+)
        // .route("/api/execute", post(execute_handler))
        // .route("/api/modify", post(modify_handler))
        
        // State
        .with_state(state)
        
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    
    // Start server
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("🚀 ALEsys API escuchando en {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}