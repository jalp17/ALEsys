//! Handlers de los endpoints HTTP

use crate::auth::{self, Claims, JwtConfig, Role};
use crate::state::AppState;
use alesys_core::agent::protocol::AgentCommand;
use alesys_core::graphrag::search::AdvancedSearchQuery;
use alesys_core::graphrag::SearchResultSource;
use alesys_core::llm::{ChatMessage, LLMEngine};
use alesys_core::session::ChatMessage as SessionChatMessage;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Request para chat
#[derive(Deserialize)]
pub struct ChatRequest {
    pub query: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub _stream: Option<bool>,
}

/// Error JSON response
#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "VALIDATION" => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self)).into_response()
    }
}

/// Response de chat
#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub sources: Vec<Source>,
    pub query: String,
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct Source {
    pub fragment_id: i32,
    pub document_id: i32,
    pub path: String,
    pub similarity: f32,
    pub source_type: String,
}

/// Handler para POST /api/chat
///
/// Si se provee session_id, carga historial previo y guarda mensajes.
pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        "Chat request: '{}' (session: {:?})",
        payload.query,
        payload.session_id
    );

    // 1. Cargar historial de sesion si existe
    let mut messages: Vec<ChatMessage> = Vec::new();

    if let Some(ref session_id) = payload.session_id {
        let history = state
            .session_manager
            .get_session_history(session_id, 20)
            .await
            .map_err(|e| {
                tracing::error!("Error cargando historial sesion {}: {}", session_id, e);
                ApiError {
                    error: "Error interno cargando historial".into(),
                    code: "INTERNAL".into(),
                }
            })?;

        for msg in history {
            messages.push(ChatMessage {
                role: msg.role,
                content: msg.content,
            });
        }
    }

    // 2. Generar embedding del query
    let query_embedding = state.embedder.encode(&payload.query).map_err(|e| {
        tracing::error!("Error generando embedding: {}", e);
        ApiError {
            error: "Error interno generando embedding".into(),
            code: "INTERNAL".into(),
        }
    })?;

    // 3. Busqueda hibrida (vector + grafo)
    let search_results = state
        .graphrag
        .hybrid_search(&query_embedding, 5, 1)
        .await
        .map_err(|e| {
            tracing::error!("Error en busqueda hibrida: {}", e);
            ApiError {
                error: "Error interno en busqueda".into(),
                code: "INTERNAL".into(),
            }
        })?;

    // 4. Construir contexto RAG
    let context = alesys_core::graphrag::build_rag_context(&search_results, 2000);

    // 5. Agregar system prompt si no hay historial
    if messages.is_empty() {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: crate::CHAT_SYSTEM_PROMPT.to_string(),
        });
    }

    // 6. Agregar query con contexto RAG
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: format!("Contexto:\n{}\n\nPregunta: {}", context, payload.query),
    });

    // 7. Llamar al LLM
    let llm_response = state.llm_queue.chat(&messages).await.map_err(|e| {
        tracing::error!("Error en LLM chat: {}", e);
        ApiError {
            error: "Error generando respuesta".into(),
            code: "INTERNAL".into(),
        }
    })?;

    // 8. Guardar mensajes en sesion si hay session_id
    if let Some(ref session_id) = payload.session_id {
        let user_msg = SessionChatMessage {
            role: "user".to_string(),
            content: payload.query.clone(),
            timestamp: Utc::now(),
            sources: None,
        };
        if let Err(e) = state
            .session_manager
            .add_message(session_id, &user_msg)
            .await
        {
            tracing::warn!(
                "No se pudo guardar mensaje de usuario en sesion {}: {}",
                session_id,
                e
            );
        }

        let source_paths: Vec<String> = search_results
            .iter()
            .filter_map(|r| r.doc_path.clone())
            .collect();
        let assistant_msg = SessionChatMessage {
            role: "assistant".to_string(),
            content: llm_response.content.clone(),
            timestamp: Utc::now(),
            sources: if source_paths.is_empty() {
                None
            } else {
                Some(source_paths)
            },
        };
        if let Err(e) = state
            .session_manager
            .add_message(session_id, &assistant_msg)
            .await
        {
            tracing::warn!(
                "No se pudo guardar mensaje de asistente en sesion {}: {}",
                session_id,
                e
            );
        }
    }

    // 9. Convertir resultados a formato de respuesta
    let sources: Vec<Source> = search_results
        .iter()
        .map(|r| Source {
            fragment_id: r.fragment_id,
            document_id: r.document_id,
            path: r
                .doc_path
                .clone()
                .unwrap_or_else(|| "desconocido".to_string()),
            similarity: r.similarity,
            source_type: match r.source {
                SearchResultSource::Vector => "vector".to_string(),
                SearchResultSource::Graph => "graph".to_string(),
            },
        })
        .collect();

    let response = ChatResponse {
        response: llm_response.content,
        sources,
        query: payload.query,
        session_id: payload.session_id,
    };

    Ok(Json(response))
}

/// Request para generacion
#[derive(Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub language: String,
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub context: Option<GenerateContext>,
}

/// Contexto de archivos existentes enviado desde el frontend
#[derive(Deserialize)]
pub struct GenerateContext {
    pub project_type: Option<String>,
    #[serde(default)]
    pub existing_files: Vec<GenerateFileInfo>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Deserialize)]
pub struct GenerateFileInfo {
    pub name: String,
    pub content: String,
}

/// Response de generacion
#[derive(Serialize)]
pub struct GenerateResponse {
    pub file_name: String,
    pub content: String,
    pub language: String,
    pub explanation: String,
    pub suggestions: Vec<String>,
}

/// Handler para POST /api/generate
pub async fn generate_handler(
    State(state): State<AppState>,
    Json(payload): Json<GenerateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        "Generate request: '{}' -> {}",
        payload.prompt,
        payload.language
    );

    let context = payload
        .context
        .map(|ctx| alesys_core::generator::BuildContext {
            project_type: ctx.project_type,
            existing_files: ctx
                .existing_files
                .into_iter()
                .map(|f| alesys_core::generator::FileInfo {
                    name: f.name,
                    content: f.content,
                })
                .collect(),
            dependencies: ctx.dependencies,
        });

    let gen_request = alesys_core::generator::GenerateRequest {
        prompt: payload.prompt,
        language: payload.language,
        context,
        max_tokens: payload.max_tokens.unwrap_or(2048),
    };

    let generator = alesys_core::generator::CodeGenerator::new(state.llm_engine.clone());

    let result = generator.generate(gen_request).await.map_err(|e| {
        tracing::error!("Error generando codigo: {}", e);
        ApiError {
            error: "Error generando codigo".into(),
            code: "INTERNAL".into(),
        }
    })?;

    let response = GenerateResponse {
        file_name: result.file_name,
        content: result.content,
        language: result.language,
        explanation: result.explanation,
        suggestions: result.suggestions,
    };

    Ok(Json(response))
}

// === Session Handlers ===

/// Request para crear sesion
#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub name: Option<String>,
}

/// Response de sesion
#[derive(Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub last_activity: String,
    pub is_active: bool,
}

/// Handler para GET /api/sessions
pub async fn list_sessions(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let sessions = state
        .session_manager
        .get_active_sessions(0)
        .await
        .map_err(|e| {
            tracing::error!("Error listando sesiones: {}", e);
            ApiError {
                error: "Error interno listando sesiones".into(),
                code: "INTERNAL".into(),
            }
        })?;

    let responses: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id,
            name: s.name,
            created_at: s.created_at.to_rfc3339(),
            last_activity: s.last_activity.to_rfc3339(),
            is_active: s.is_active,
        })
        .collect();

    Ok(Json(serde_json::json!({ "sessions": responses })))
}

/// Handler para POST /api/sessions
pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = state
        .session_manager
        .create_session(0, payload.name)
        .await
        .map_err(|e| {
            tracing::error!("Error creando sesion: {}", e);
            ApiError {
                error: "Error interno creando sesion".into(),
                code: "INTERNAL".into(),
            }
        })?;

    tracing::info!("Sesion creada: {}", session_id);

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "message": "Sesion creada correctamente"
    })))
}

/// Handler para GET /api/sessions/:id
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let session = state
        .session_manager
        .get_by_id(&session_id)
        .await
        .map_err(|e| {
            tracing::error!("Error obteniendo sesion {}: {}", session_id, e);
            ApiError {
                error: "Error interno obteniendo sesion".into(),
                code: "INTERNAL".into(),
            }
        })?;

    match session {
        Some(s) => Ok(Json(serde_json::json!({
            "id": s.id,
            "name": s.name,
            "created_at": s.created_at.to_rfc3339(),
            "last_activity": s.last_activity.to_rfc3339(),
            "is_active": s.is_active,
        }))),
        None => Err(ApiError {
            error: "Sesion no encontrada".into(),
            code: "NOT_FOUND".into(),
        }),
    }
}

/// Handler para DELETE /api/sessions/:id
pub async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .session_manager
        .close_session(&session_id)
        .await
        .map_err(|e| {
            tracing::error!("Error cerrando sesion {}: {}", session_id, e);
            ApiError {
                error: "Error interno cerrando sesion".into(),
                code: "INTERNAL".into(),
            }
        })?;

    tracing::info!("Sesion cerrada: {}", session_id);

    Ok(Json(serde_json::json!({
        "message": "Sesion cerrada correctamente"
    })))
}

/// Handler para GET /api/sessions/:id/history
pub async fn get_session_history(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let messages = state
        .session_manager
        .get_session_history(&session_id, 100)
        .await
        .map_err(|e| {
            tracing::error!("Error cargando historial sesion {}: {}", session_id, e);
            ApiError {
                error: "Error interno cargando historial".into(),
                code: "INTERNAL".into(),
            }
        })?;

    let responses: Vec<serde_json::Value> = messages
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
                "timestamp": m.timestamp.to_rfc3339(),
                "sources": m.sources,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "messages": responses })))
}

/// Handler para GET /api/graph/stats
pub async fn graph_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.graphrag.graph_stats();
    Json(serde_json::json!({
        "nodes": stats.nodes,
        "edges": stats.edges,
    }))
}

/// Handler para GET /api/v1/graph
pub async fn get_graph(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<alesys_core::graphrag::api::GraphQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state
        .graphrag
        .get_graph_api(&query, 0) // user_id 0 = admin
        .await
        .map_err(|e| {
            tracing::error!("Error obteniendo grafo: {}", e);
            ApiError {
                error: "Error obteniendo grafo".into(),
                code: "INTERNAL".into(),
            }
        })?;
    Ok(Json(response))
}

/// Handler para GET /api/v1/graph/centrality
pub async fn get_centrality(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<alesys_core::graphrag::api::CentralityQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state.graphrag.get_centrality(&query).await.map_err(|e| {
        tracing::error!("Error calculando centralidad: {}", e);
        ApiError {
            error: "Error calculando centralidad".into(),
            code: "INTERNAL".into(),
        }
    })?;
    Ok(Json(response))
}

/// Handler para GET /api/v1/graph/communities
pub async fn get_communities(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<alesys_core::graphrag::api::CommunitiesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state.graphrag.get_communities(&query).await.map_err(|e| {
        tracing::error!("Error calculando comunidades: {}", e);
        ApiError {
            error: "Error calculando comunidades".into(),
            code: "INTERNAL".into(),
        }
    })?;
    Ok(Json(response))
}

/// Handler para GET /api/v1/graph/path
pub async fn get_shortest_path(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<alesys_core::graphrag::api::PathQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state
        .graphrag
        .get_shortest_path(&query)
        .await
        .map_err(|e| {
            tracing::error!("Error calculando camino: {}", e);
            ApiError {
                error: "Error calculando camino".into(),
                code: "INTERNAL".into(),
            }
        })?;
    Ok(Json(response))
}

/// Handler para GET /api/v1/graph/search
pub async fn search_graph(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit: usize = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20)
        .min(100);

    let results = state
        .graphrag
        .search_graph(query, limit)
        .await
        .map_err(|e| {
            tracing::error!("Error buscando en grafo: {}", e);
            ApiError {
                error: "Error buscando en grafo".into(),
                code: "INTERNAL".into(),
            }
        })?;
    Ok(Json(serde_json::json!({ "nodes": results })))
}

/// Handler para GET /api/v1/graph/export — export graph as JSON
pub async fn export_graph_json(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state
        .graphrag
        .get_graph_api(
            &alesys_core::graphrag::api::GraphQuery {
                doc_type: None,
                edge_type: None,
                depth: None,
                limit: Some(10000),
                cursor: None,
                center_node_id: None,
                include_metrics: Some(true),
            },
            0,
        )
        .await
        .map_err(|e| {
            tracing::error!("Error exportando grafo: {}", e);
            ApiError {
                error: "Error exportando grafo".into(),
                code: "INTERNAL".into(),
            }
        })?;

    Ok(Json(response))
}

/// Handler para POST /api/v1/search/advanced
///
/// Búsqueda híbrida avanzada con RRF, filtros múltiples, query expansion
/// y highlighting de términos.
pub async fn advanced_search_handler(
    State(state): State<AppState>,
    Json(payload): Json<AdvancedSearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        "Advanced search: '{}' (filters: types={}, areas={}, date={}-{})",
        payload.query,
        payload.filters.doc_types.len(),
        payload.filters.areas.len(),
        payload.filters.date_from.as_deref().unwrap_or("*"),
        payload.filters.date_to.as_deref().unwrap_or("*"),
    );

    // Generate embedding if query is non-empty
    let embedding = if !payload.query.is_empty() {
        let emb = state.embedder.encode(&payload.query).map_err(|e| {
            tracing::error!("Error generando embedding: {}", e);
            ApiError {
                error: "Error generando embedding".into(),
                code: "INTERNAL".into(),
            }
        })?;
        Some(emb)
    } else {
        None
    };

    let response = alesys_core::graphrag::search::advanced_search(
        &state.db,
        &payload,
        embedding.as_deref(),
        Some(&state.graphrag),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error en advanced search: {}", e);
        ApiError {
            error: "Error en búsqueda avanzada".into(),
            code: "INTERNAL".into(),
        }
    })?;

    Ok(Json(response))
}

/// Health check endpoint
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_optional(&state.db)
        .await
        .is_ok();

    let status = if db_ok { "ok" } else { "degraded" };

    Json(serde_json::json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "db": if db_ok { "connected" } else { "disconnected" },
        "llm": state.llm_engine.is_available(),
        "embedder": state.embedder.is_available(),
    }))
}

/// Handler para GET /api/v1/config — returns current runtime configuration
pub async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_optional(&state.db)
        .await
        .is_ok();

    Json(serde_json::json!({
        "llm": {
            "backend": state.llm_config.backend,
            "model_path": state.llm_config.model_path,
            "temperature": state.llm_config.temperature,
            "top_p": state.llm_config.top_p,
            "max_tokens": state.llm_config.max_tokens,
            "context_size": state.llm_config.context_size,
            "gpu_layers": state.llm_config.gpu_layers,
        },
        "embeddings": {
            "dimension": 384,
            "loaded": state.embedder.is_available(),
        },
        "health": {
            "llm_available": state.llm_engine.is_available(),
            "embedder_available": state.embedder.is_available(),
            "db_connected": db_ok,
            "version": env!("CARGO_PKG_VERSION"),
        },
    }))
}

// =============================================================================
// Phase 9: Agent Handlers
// =============================================================================

#[allow(dead_code)]
fn default_timeout() -> u64 {
    30_000
}

#[allow(dead_code)]
/// Request para POST /api/v1/agents/:id/execute
#[derive(Deserialize)]
pub struct AgentExecuteRequest {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub workdir: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[allow(dead_code)]
/// Response de POST /api/v1/agents/:id/execute
#[derive(Serialize)]
pub struct AgentExecuteResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

#[allow(dead_code)]
/// Request para POST /api/v1/agents/:id/files
#[derive(Deserialize)]
pub struct AgentWriteFileRequest {
    pub path: String,
    pub content: String,
}

/// Handler para GET /api/v1/agents
pub async fn list_agents(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let agents = state.agent_manager.list_agents().await;
    Ok(Json(serde_json::json!({ "agents": agents })))
}

/// Handler para GET /api/v1/agents/stats
pub async fn agent_stats(State(state): State<AppState>) -> impl IntoResponse {
    let total = state.agent_manager.get_agent_count().await;
    let connected = state.agent_manager.get_connected_count().await;
    Json(serde_json::json!({
        "total": total,
        "connected": connected,
    }))
}

/// POST /api/v1/agents/:id/execute - Execute command on agent
pub async fn agent_execute(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(req): Json<AgentExecuteRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cmd = AgentCommand::Execute {
        id: Uuid::new_v4().to_string(),
        command: req.command,
        args: req.args,
        workdir: req.workdir,
        timeout_ms: req.timeout_ms,
    };

    let timeout = Duration::from_millis(req.timeout_ms);
    match state.agent_manager.send_command(&agent_id, cmd, Some(timeout)).await {
        Ok(alesys_core::agent::protocol::AgentResponse::ExecuteResult { exit_code, stdout, stderr, .. }) => {
            Ok(Json(serde_json::json!({
                "exit_code": exit_code,
                "stdout": stdout,
                "stderr": stderr,
            })))
        }
        Ok(_) => Err(ApiError { error: "Unexpected response".into(), code: "UNEXPECTED".into() }),
        Err(e) => Err(ApiError { error: e, code: "AGENT_ERROR".into() }),
    }
}

/// GET /api/v1/agents/:id/files?path=... - Read file from agent
pub async fn agent_read_file(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let path = params.get("path").ok_or_else(|| ApiError { error: "Missing 'path' param".into(), code: "BAD_REQUEST".into() })?;

    let cmd = AgentCommand::ReadFile {
        id: Uuid::new_v4().to_string(),
        path: path.clone(),
    };

    match state.agent_manager.send_command(&agent_id, cmd, None).await {
        Ok(alesys_core::agent::protocol::AgentResponse::FileContent { content, .. }) => {
            Ok(Json(serde_json::json!({ "content": content })))
        }
        Ok(_) => Err(ApiError { error: "Unexpected response".into(), code: "UNEXPECTED".into() }),
        Err(e) => Err(ApiError { error: e, code: "AGENT_ERROR".into() }),
    }
}

/// POST /api/v1/agents/:id/files - Write file on agent
pub async fn agent_write_file(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(req): Json<AgentWriteFileRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cmd = AgentCommand::WriteFile {
        id: Uuid::new_v4().to_string(),
        path: req.path,
        content: req.content,
    };

    match state.agent_manager.send_command(&agent_id, cmd, None).await {
        Ok(alesys_core::agent::protocol::AgentResponse::ExecuteResult { exit_code, .. }) => {
            if exit_code == 0 {
                Ok(Json(serde_json::json!({ "success": true })))
            } else {
                Err(ApiError { error: "Write failed on agent".into(), code: "AGENT_WRITE_ERROR".into() })
            }
        }
        Ok(_) => Err(ApiError { error: "Unexpected response".into(), code: "UNEXPECTED".into() }),
        Err(e) => Err(ApiError { error: e, code: "AGENT_ERROR".into() }),
    }
}

/// GET /api/v1/agents/:id/files/list?path=... - List directory on agent
pub async fn agent_list_dir(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let path = params.get("path").cloned().unwrap_or_else(|| ".".to_string());

    let cmd = AgentCommand::ListDirectory {
        id: Uuid::new_v4().to_string(),
        path,
    };

    match state.agent_manager.send_command(&agent_id, cmd, None).await {
        Ok(alesys_core::agent::protocol::AgentResponse::DirectoryList { entries, .. }) => {
            Ok(Json(serde_json::json!({ "entries": entries })))
        }
        Ok(_) => Err(ApiError { error: "Unexpected response".into(), code: "UNEXPECTED".into() }),
        Err(e) => Err(ApiError { error: e, code: "AGENT_ERROR".into() }),
    }
}

// =============================================================================
// Authentication
// =============================================================================

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub role: String,
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: In production, validate against database with bcrypt
    // For now, simple hardcoded validation for development
    let (user_id, role) = match (req.username.as_str(), req.password.as_str()) {
        ("admin", "alesys") => ("admin", Role::Admin),
        ("user", "alesys") => ("user", Role::User),
        _ => {
            return Err(ApiError {
                error: "Invalid credentials".into(),
                code: "UNAUTHORIZED".into(),
            });
        }
    };

    let jwt_config = auth::JwtConfig::from_env();
    let token = auth::create_token(user_id, role.clone(), &jwt_config)
        .map_err(|e| ApiError { error: e.to_string(), code: "TOKEN_ERROR".into() })?;

    Ok(Json(serde_json::json!({
        "token": token,
        "role": format!("{:?}", role).to_lowercase(),
    })))
}

/// GET /api/v1/auth/me
pub async fn get_current_user(
    claims: Claims,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "user_id": claims.sub,
        "role": format!("{:?}", claims.role).to_lowercase(),
    })))
}

// ===== Plugin Endpoints =====

/// GET /api/v1/plugins - List installed plugins
pub async fn list_plugins(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let plugins = state.plugin_manager.list_plugins().await;
    Ok(Json(serde_json::json!({
        "plugins": plugins,
        "count": plugins.len(),
    })))
}

/// POST /api/v1/plugins/:id/execute - Execute a plugin command
#[derive(Deserialize)]
pub struct PluginExecuteRequest {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

pub async fn execute_plugin(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
    Json(req): Json<PluginExecuteRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let context = alesys_core::plugin::PluginContext {
        work_dir: std::env::temp_dir(),
        allowed_paths: vec![],
        config: std::collections::HashMap::new(),
        request_id: Uuid::new_v4().to_string(),
    };

    match state
        .plugin_manager
        .execute(&plugin_id, &req.command, &req.args, &context)
        .await
    {
        Ok(result) => Ok(Json(serde_json::json!({
            "success": result.success,
            "output": result.output,
            "error": result.error,
            "metadata": result.metadata,
        }))),
        Err(e) => Err(ApiError {
            error: e,
            code: "PLUGIN_ERROR".into(),
        }),
    }
}

/// POST /api/v1/plugins/:id/enable - Enable a plugin
pub async fn enable_plugin(
    State(_state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Update database and reload plugin
    Ok(Json(serde_json::json!({
        "plugin_id": plugin_id,
        "enabled": true,
        "message": "Plugin enabled",
    })))
}

/// POST /api/v1/plugins/:id/disable - Disable a plugin
pub async fn disable_plugin(
    State(_state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Update database and unload plugin
    Ok(Json(serde_json::json!({
        "plugin_id": plugin_id,
        "enabled": false,
        "message": "Plugin disabled",
    })))
}

// ===== Marketplace Endpoints =====

/// GET /api/v1/marketplace/plugins - List available plugins
pub async fn marketplace_list(
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Fetch from remote marketplace API
    let plugins = vec![
        serde_json::json!({
            "id": "git-integration",
            "name": "Git Integration",
            "version": "0.1.0",
            "author": "ALEsys",
            "description": "Git integration for ALEsys",
            "installed": true,
        }),
        serde_json::json!({
            "id": "test-runner",
            "name": "Test Runner",
            "version": "0.1.0",
            "author": "ALEsys",
            "description": "Run tests automatically",
            "installed": true,
        }),
        serde_json::json!({
            "id": "docker-runner",
            "name": "Docker Runner",
            "version": "0.1.0",
            "author": "ALEsys",
            "description": "Run code in Docker containers",
            "installed": true,
        }),
    ];

    Ok(Json(serde_json::json!({
        "plugins": plugins,
        "count": plugins.len(),
    })))
}

/// POST /api/v1/marketplace/install/:id - Install a plugin
pub async fn marketplace_install(
    State(_state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Download and install plugin from marketplace
    Ok(Json(serde_json::json!({
        "plugin_id": plugin_id,
        "status": "installed",
        "message": "Plugin installed successfully",
    })))
}

/// DELETE /api/v1/marketplace/uninstall/:id - Uninstall a plugin
pub async fn marketplace_uninstall(
    State(_state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Remove plugin and cleanup
    Ok(Json(serde_json::json!({
        "plugin_id": plugin_id,
        "status": "uninstalled",
        "message": "Plugin uninstalled successfully",
    })))
}

pub async fn pair_programmer_analyze(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let file_path = payload.get("file_path").and_then(|v| v.as_str()).unwrap_or("unknown");

    let mut suggestions = Vec::new();

    if code.contains("TODO") {
        suggestions.push(serde_json::json!({
            "id": "todo-1",
            "suggestion_type": "CodeSmell",
            "file_path": file_path,
            "line": code.lines().position(|l| l.contains("TODO")).map(|i| i + 1).unwrap_or(1),
            "description": "Found TODO comment",
            "severity": "Low",
            "auto_fixable": false,
        }));
    }
    if code.contains("unwrap()") {
        suggestions.push(serde_json::json!({
            "id": "unwrap-1",
            "suggestion_type": "CodeSmell",
            "file_path": file_path,
            "line": code.lines().position(|l| l.contains("unwrap()")).map(|i| i + 1).unwrap_or(1),
            "description": "Using unwrap() instead of proper error handling",
            "severity": "Medium",
            "auto_fixable": false,
        }));
    }
    if code.contains("println!") {
        suggestions.push(serde_json::json!({
            "id": "println-1",
            "suggestion_type": "CodeSmell",
            "file_path": file_path,
            "line": code.lines().position(|l| l.contains("println!")).map(|i| i + 1).unwrap_or(1),
            "description": "Found println! - consider using logging instead",
            "severity": "Low",
            "auto_fixable": false,
        }));
    }

    Ok(Json(serde_json::json!({
        "suggestions": suggestions,
        "total": suggestions.len(),
    })))
}

pub async fn pair_programmer_refactor(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let refactor_type = payload.get("refactor_type").and_then(|v| v.as_str()).unwrap_or("remove_whitespace");

    let refactored = match refactor_type {
        "remove_whitespace" => {
            let lines: Vec<&str> = code.lines().collect();
            lines
                .join("\n")
                .split("\n")
                .map(|l| l.trim_end())
                .collect::<Vec<_>>()
                .join("\n")
        }
        "sort_imports" => {
            let mut lines: Vec<&str> = code.lines().collect();
            let mut import_lines: Vec<&str> = Vec::new();
            let mut other_lines: Vec<&str> = Vec::new();

            for &line in &lines {
                if line.trim_start().starts_with("use ") || line.trim_start().starts_with("import ") {
                    import_lines.push(line);
                } else {
                    other_lines.push(line);
                }
            }

            import_lines.sort();
            lines = import_lines;
            lines.extend(other_lines);
            lines.join("\n")
        }
        _ => code.to_string(),
    };

    Ok(Json(serde_json::json!({
        "code": refactored,
        "refactor_type": refactor_type,
    })))
}

pub async fn pair_programmer_project(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "total_files": 0,
        "total_lines": 0,
        "file_types": {},
        "message": "Project analysis not yet implemented",
    })))
}

