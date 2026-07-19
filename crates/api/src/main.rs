//! ALEsys API - Backend REST + WebSocket
//!
//! Endpoints (v1):
//! - POST /api/v1/chat               -> Chat con GraphRAG + sesiones
//! - POST /api/v1/generate           -> Generar archivos
//! - GET  /api/v1/sessions           -> Listar sesiones activas
//! - POST /api/v1/sessions           -> Crear sesion
//! - GET  /api/v1/sessions/:id       -> Detalle de sesion
//! - DELETE /api/v1/sessions/:id     -> Cerrar sesion
//! - GET  /api/v1/sessions/:id/history -> Historial de chat
//! - GET  /ws/chat                   -> WebSocket para streaming
//! - GET  /api/v1/graph/stats        -> Estadisticas del grafo
//! - GET  /health                    -> Health check
//!
//! Legacy /api/* routes are also available for backwards compatibility.

use anyhow::Result;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static METRICS_HANDLE: OnceLock<metrics_exporter_prometheus::PrometheusHandle> = OnceLock::new();

mod auth;
mod handlers;
mod state;
mod websocket;

pub(crate) const CHAT_SYSTEM_PROMPT: &str =
    "Eres un asistente de IA experto en programación y análisis de documentos. Responde de forma clara y concisa basándote en el contexto proporcionado.";

use handlers::{
    advanced_search_handler, chat_handler, create_session, delete_session, export_graph_json,
    generate_handler, get_centrality, get_communities, get_config, get_graph, get_session,
    get_session_history, get_shortest_path, graph_stats, health_handler, list_sessions,
    search_graph, list_agents, agent_stats, agent_execute, agent_read_file, agent_write_file, agent_list_dir,
    login, get_current_user,
    list_plugins, execute_plugin, enable_plugin, disable_plugin,
    marketplace_list, marketplace_install, marketplace_uninstall,
    pair_programmer_analyze, pair_programmer_refactor, pair_programmer_project,
    learning_feedback, learning_insights,
    debug_analyze,
    test_generate,
};
use state::AppState;
use websocket::ws_chat_handler;
use websocket::ws_agent_handler;

/// Metrics endpoint — Prometheus format
async fn metrics_handler() -> impl IntoResponse {
    let body = match METRICS_HANDLE.get() {
        Some(handle) => handle.render(),
        None => "# metrics not initialized\n".to_string(),
    };
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

/// Simple sliding-window rate limiter per IP
struct RateLimiterState {
    window_secs: u64,
    max_requests: usize,
    windows: RwLock<HashMap<std::net::IpAddr, (u64, usize)>>,
}

impl RateLimiterState {
    fn new(max_requests_per_min: usize) -> Self {
        Self {
            window_secs: 60,
            max_requests: max_requests_per_min,
            windows: RwLock::new(HashMap::new()),
        }
    }

    async fn check(&self, ip: std::net::IpAddr) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let window_start = now / self.window_secs * self.window_secs;

        let mut windows = self.windows.write().await;
        let entry = windows.entry(ip).or_insert((window_start, 0));

        if entry.0 < window_start {
            *entry = (window_start, 1);
            true
        } else if entry.1 < self.max_requests {
            entry.1 += 1;
            true
        } else {
            false
        }
    }
}

async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiterState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse::<std::net::IpAddr>().ok())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));

    if limiter.check(ip).await {
        next.run(request).await
    } else {
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "Rate limit exceeded",
                "code": "RATE_LIMITED",
                "retry_after": 60,
            })),
        )
            .into_response()
    }
}

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
        tracing::warn!("Neither DATABASE_URL nor PGHOST set — usando defaults de docker-compose");
    }

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            let host = std::env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string());
            let port = std::env::var("PGPORT").unwrap_or_else(|_| "5433".to_string());
            let user = std::env::var("PGUSER").unwrap_or_else(|_| "alesys".to_string());
            let password = std::env::var("PGPASSWORD").unwrap_or_else(|_| {
                tracing::warn!("PGPASSWORD not set — using default for development only");
                "alesys".to_string()
            });
            let dbname = std::env::var("PGDATABASE").unwrap_or_else(|_| "alesys".to_string());
            format!(
                "postgres://{}:{}@{}:{}/{}",
                user, password, host, port, dbname
            )
        }
    };

    let db_pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(
            std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(25),
        )
        .min_connections(
            std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
        )
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(300))
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!(
        "Database pool configured (max={})",
        db_pool.options().get_max_connections()
    );

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

    // Rate limiting config
    let rate_limit_per_min: u32 = std::env::var("RATE_LIMIT_PER_MIN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    tracing::info!(
        "Rate limit configured: {} req/min per IP",
        rate_limit_per_min
    );

    let rate_limiter = Arc::new(RateLimiterState::new(rate_limit_per_min as usize));

    let api_v1 = Router::new()
        .route("/chat", post(chat_handler))
        .route("/generate", post(generate_handler))
        .route("/sessions", get(list_sessions))
        .route("/sessions", post(create_session))
        .route("/sessions/:id", get(get_session))
        .route("/sessions/:id", delete(delete_session))
        .route("/sessions/:id/history", get(get_session_history))
        .route("/graph", get(get_graph))
        .route("/graph/stats", get(graph_stats))
        .route("/graph/centrality", get(get_centrality))
        .route("/graph/communities", get(get_communities))
        .route("/graph/path", get(get_shortest_path))
        .route("/graph/search", get(search_graph))
        .route("/graph/export", get(export_graph_json))
        .route("/search/advanced", post(advanced_search_handler))
        .route("/config", get(get_config))
        .route("/auth/login", post(login))
        .route("/auth/me", get(get_current_user))
        .route("/agents", get(list_agents))
        .route("/agents/stats", get(agent_stats))
        .route("/agents/:id/execute", post(agent_execute))
        .route("/agents/:id/files", get(agent_read_file).post(agent_write_file))
        .route("/agents/:id/files/list", get(agent_list_dir))
        // Plugin endpoints
        .route("/plugins", get(list_plugins))
        .route("/plugins/:id/execute", post(execute_plugin))
        .route("/plugins/:id/enable", post(enable_plugin))
        .route("/plugins/:id/disable", post(disable_plugin))
        // Marketplace endpoints
        .route("/marketplace/plugins", get(marketplace_list))
        .route("/marketplace/install/:id", post(marketplace_install))
        .route("/marketplace/uninstall/:id", delete(marketplace_uninstall))
        // Pair programmer endpoints
        .route("/pair-programmer/analyze", post(pair_programmer_analyze))
        .route("/pair-programmer/refactor", post(pair_programmer_refactor))
        .route("/pair-programmer/project", get(pair_programmer_project))
        // Learning endpoints
        .route("/learning/feedback", post(learning_feedback))
        .route("/learning/insights", get(learning_insights))
        // Debug assistant endpoints
        .route("/debug/analyze", post(debug_analyze))
        // Test generation endpoints
        .route("/test-generate", post(test_generate));

    let app = Router::new()
        .nest("/api/v1", api_v1.clone())
        .nest("/api", api_v1)
        .route("/ws/chat", get(ws_chat_handler))
        .route("/ws/agent", get(ws_agent_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(timeout_secs),
        ))
        .layer(TraceLayer::new_for_http());

    let addr = std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!(
        "ALEsys API listening on {} (timeout={}s)",
        addr,
        timeout_secs
    );

    // Initialize Prometheus metrics
    match metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
    {
        Ok(handle) => {
            let _ = METRICS_HANDLE.set(handle);
            tracing::info!("Prometheus metrics initialized");
        }
        Err(e) => {
            tracing::warn!("Failed to initialize metrics recorder: {}", e);
        }
    }

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
