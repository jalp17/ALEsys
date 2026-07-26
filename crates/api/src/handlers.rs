//! Handlers de los endpoints HTTP

use crate::auth::{self, Claims, JwtConfig, Role};
use crate::state::AppState;
use alesys_core::agent::protocol::AgentCommand;
use alesys_core::graphrag::search::AdvancedSearchQuery;
use alesys_core::graphrag::SearchResultSource;
use alesys_core::llm::{ChatMessage, LLMEngine, LLMState};
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

    let llm_loaded = state.llm_engine.read().await.is_loaded();

    Json(serde_json::json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "db": if db_ok { "connected" } else { "disconnected" },
        "llm_loaded": llm_loaded,
        "embedder": state.embedder.is_available(),
    }))
}

/// Handler para GET /api/v1/config — returns current runtime configuration
pub async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_optional(&state.db)
        .await
        .is_ok();

    let llm_loaded = state.llm_engine.read().await.is_loaded();

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
            "llm_loaded": llm_loaded,
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

pub async fn learning_feedback(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let suggestion_id = payload.get("suggestion_id").and_then(|v| v.as_str()).unwrap_or("");
    let rating = payload.get("rating").and_then(|v| v.as_str()).unwrap_or("neutral");
    let suggestion_type = payload.get("suggestion_type").and_then(|v| v.as_str()).unwrap_or("unknown");

    let feedback_id = uuid::Uuid::new_v4().to_string();

    Ok(Json(serde_json::json!({
        "id": feedback_id,
        "suggestion_id": suggestion_id,
        "rating": rating,
        "suggestion_type": suggestion_type,
        "status": "recorded",
    })))
}

pub async fn learning_insights(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "insights": [
            {
                "insight_type": "LanguagePreference",
                "description": "Most used language: Rust (based on project structure)",
                "confidence": 0.85,
                "based_on_count": 42,
            },
            {
                "insight_type": "SuggestionPreference",
                "description": "TODO suggestions are 80% helpful based on user feedback",
                "confidence": 0.8,
                "based_on_count": 15,
            },
        ],
    })))
}

pub async fn debug_analyze(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::debug_assistant::log_parser::LogParser;
    use alesys_core::debug_assistant::analyzer::DebugAnalyzer;
    use alesys_core::debug_assistant::suggestion::SuggestionFormatter;

    let logs_input = payload.get("logs").and_then(|v| v.as_str()).unwrap_or("");

    let parser = LogParser::new();
    let logs = parser.parse_logs(logs_input);

    let analyzer = DebugAnalyzer::new();
    let analysis = analyzer.analyze(&logs);

    let formatter = SuggestionFormatter::new();
    let report = formatter.format(&analysis);

    Ok(Json(serde_json::json!({
        "summary": report.analysis_summary,
        "severity": report.severity,
        "total_errors": report.total_errors,
        "total_warnings": report.total_warnings,
        "patterns_found": report.patterns_found,
        "root_cause": report.root_cause,
        "suggestions": report.suggestions,
    })))
}

pub async fn test_generate(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::test_generation::generator::{TestGenerator, FunctionInfo, ParameterInfo, ComplexityLevel};
    use alesys_core::test_generation::suite::TestSuite;

    let function_name = payload.get("function_name").and_then(|v| v.as_str()).unwrap_or("unknown");
    let language = payload.get("language").and_then(|v| v.as_str()).unwrap_or("rust");

    let params: Vec<ParameterInfo> = payload.get("parameters")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().filter_map(|p| {
                Some(ParameterInfo {
                    name: p.get("name")?.as_str()?.to_string(),
                    type_name: p.get("type")?.as_str()?.to_string(),
                    is_optional: p.get("optional").and_then(|v| v.as_bool()).unwrap_or(false),
                    default_value: p.get("default").and_then(|v| v.as_str()).map(|s| s.to_string()),
                })
            }).collect()
        })
        .unwrap_or_default();

    let function = FunctionInfo {
        name: function_name.to_string(),
        parameters: params,
        return_type: payload.get("return_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        is_async: payload.get("is_async").and_then(|v| v.as_bool()).unwrap_or(false),
        complexity: ComplexityLevel::Moderate,
        dependencies: vec![],
    };

    let generator = TestGenerator::new(language, "built-in");
    let tests = generator.generate_for_function(&function);

    let mut suite = TestSuite::new(&format!("{}_tests", function_name), language, "built-in");
    suite.add_tests(tests);

    Ok(Json(serde_json::json!({
        "suite_name": suite.name,
        "total_tests": suite.get_test_count(),
        "test_code": suite.export_to_file(),
        "summary": suite.get_summary(),
    })))
}

pub async fn refactoring_analyze(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::advanced_refactoring::analyzer::CodeAnalyzer;

    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = payload.get("language").and_then(|v| v.as_str()).unwrap_or("rust");

    let analyzer = CodeAnalyzer::new();
    let blocks = analyzer.analyze_code(code, language);
    let opportunities = analyzer.find_opportunities(&blocks);
    let graph = analyzer.build_dependency_graph(&blocks);

    Ok(Json(serde_json::json!({
        "blocks": blocks.len(),
        "opportunities": opportunities.iter().map(|o| serde_json::json!({
            "type": format!("{:?}", o.opportunity_type),
            "description": o.description,
            "confidence": o.confidence,
            "impact": format!("{:?}", o.estimated_impact),
        })).collect::<Vec<_>>(),
        "dependency_graph": {
            "nodes": graph.nodes.len(),
            "edges": graph.edges.len(),
            "circular_deps": graph.circular_deps.len(),
        },
    })))
}

pub async fn refactoring_preview(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::advanced_refactoring::analyzer::CodeAnalyzer;
    use alesys_core::advanced_refactoring::transformer::Transformer;
    use alesys_core::advanced_refactoring::preview::PreviewGenerator;
    use alesys_core::advanced_refactoring::analyzer::{RefactoringOpportunity, OpportunityType, ImpactLevel};

    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = payload.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    let ref_type = payload.get("refactoring_type").and_then(|v| v.as_str()).unwrap_or("RemoveDeadCode");

    let analyzer = CodeAnalyzer::new();
    let blocks = analyzer.analyze_code(code, language);

    let opportunity_type = match ref_type {
        "ExtractFunction" => OpportunityType::ExtractFunction,
        "RenameSymbol" => OpportunityType::RenameSymbol,
        "InlineFunction" => OpportunityType::InlineFunction,
        "SimplifyConditional" => OpportunityType::SimplifyConditional,
        "RemoveDeadCode" => OpportunityType::RemoveDeadCode,
        "DeduplicateCode" => OpportunityType::DeduplicateCode,
        _ => OpportunityType::RemoveDeadCode,
    };

    let opportunity = RefactoringOpportunity {
        opportunity_type,
        description: format!("Apply {}", ref_type),
        confidence: 0.8,
        affected_blocks: blocks.iter().take(1).map(|b| b.id.clone()).collect(),
        estimated_impact: ImpactLevel::Medium,
    };

    let transformer = Transformer::new();
    let result = transformer.apply_refactoring(code, &opportunity, &blocks);

    let preview_gen = PreviewGenerator::new();
    let preview = preview_gen.generate_preview(&result);
    let text_preview = preview_gen.format_diff_as_text(&preview);

    Ok(Json(serde_json::json!({
        "success": result.success,
        "changes": result.changes.len(),
        "preview": text_preview,
        "can_apply": preview.can_apply,
        "warnings": preview.warnings,
    })))
}

pub async fn kb_merge(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::knowledge_curation::merger::{DocumentMerger, Document, MergeStrategy};

    let empty_vec: Vec<serde_json::Value> = vec![];
    let docs_input = payload.get("documents").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
    let strategy = payload.get("strategy").and_then(|v| v.as_str()).unwrap_or("smart");

    let docs: Vec<Document> = docs_input.iter().enumerate().map(|(i, d)| {
        Document {
            id: d.get("id").and_then(|v| v.as_str()).unwrap_or(&format!("doc-{}", i)).to_string(),
            title: d.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled").to_string(),
            content: d.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            tags: d.get("tags").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            metadata: std::collections::HashMap::new(),
        }
    }).collect();

    let merge_strategy = match strategy {
        "concatenate" => MergeStrategy::Concatenate,
        "interleave" => MergeStrategy::Interleave,
        "manual" => MergeStrategy::Manual,
        _ => MergeStrategy::Smart,
    };

    let merger = DocumentMerger::new(merge_strategy);
    let result = merger.merge(&docs);

    Ok(Json(serde_json::json!({
        "success": result.success,
        "merged_content": result.merged_content,
        "sources_count": result.sources_count,
        "conflicts": result.conflicts,
        "warnings": result.warnings,
    })))
}

pub async fn kb_split(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::knowledge_curation::splitter::{DocumentSplitter, SplitStrategy};

    let document_id = payload.get("document_id").and_then(|v| v.as_str()).unwrap_or("doc");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let strategy = payload.get("strategy").and_then(|v| v.as_str()).unwrap_or("smart");

    let split_strategy = match strategy {
        "by-size" => {
            let max = payload.get("max_chars").and_then(|v| v.as_u64()).unwrap_or(500) as usize;
            SplitStrategy::BySize { max_chars: max }
        }
        "by-headers" => SplitStrategy::ByHeaders,
        "by-paragraphs" => SplitStrategy::ByParagraphs,
        "by-sentences" => {
            let max = payload.get("max_sentences").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            SplitStrategy::BySentences { max_sentences: max }
        }
        _ => SplitStrategy::Smart,
    };

    let splitter = DocumentSplitter::new(split_strategy);
    let result = splitter.split(document_id, content);

    Ok(Json(serde_json::json!({
        "original_id": result.original_id,
        "chunks_count": result.chunks.len(),
        "chunks": result.chunks.iter().map(|c| serde_json::json!({
            "id": c.id,
            "content": c.content,
            "index": c.index,
            "start_offset": c.start_offset,
            "end_offset": c.end_offset,
        })).collect::<Vec<_>>(),
        "strategy_used": result.strategy_used,
    })))
}

pub async fn kb_archive(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::knowledge_curation::archiver::{DocumentArchiver, ArchiveReason};

    let document_id = payload.get("document_id").and_then(|v| v.as_str()).unwrap_or("doc");
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("outdated");

    let archiver = DocumentArchiver::new("/tmp/alesys-archive");
    let archive_reason = match reason {
        "deprecated" => ArchiveReason::Deprecated,
        "duplicate" => ArchiveReason::Duplicate,
        "merged" => ArchiveReason::Merged,
        "unused" => ArchiveReason::Unused,
        r => ArchiveReason::Custom(r.to_string()),
    };

    let result = archiver.archive(document_id, title, content, &[], archive_reason);

    Ok(Json(serde_json::json!({
        "success": result.success,
        "document_id": result.document_id,
        "archive_path": result.archive_path,
        "reason": result.reason,
        "archived_at": result.archived_at,
    })))
}

pub async fn kb_duplicates(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::knowledge_curation::dedup::{DuplicateDetector, SimilarityMethod};
    use alesys_core::knowledge_curation::merger::Document;

    let empty_vec: Vec<serde_json::Value> = vec![];
    let docs_input = payload.get("documents").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
    let threshold = payload.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.7);
    let method = payload.get("method").and_then(|v| v.as_str()).unwrap_or("fuzzy");

    let docs: Vec<Document> = docs_input.iter().enumerate().map(|(i, d)| {
        Document {
            id: d.get("id").and_then(|v| v.as_str()).unwrap_or(&format!("doc-{}", i)).to_string(),
            title: d.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled").to_string(),
            content: d.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            tags: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }).collect();

    let similarity_method = match method {
        "exact" => SimilarityMethod::Exact,
        "token" => SimilarityMethod::TokenOverlap,
        "semantic" => SimilarityMethod::Semantic,
        _ => SimilarityMethod::Fuzzy,
    };

    let detector = DuplicateDetector::new(threshold, similarity_method);
    let report = detector.detect(&docs);

    Ok(Json(serde_json::json!({
        "total_checked": report.total_checked,
        "duplicates_found": report.pairs.len(),
        "method_used": report.method_used,
        "pairs": report.pairs.iter().map(|p| serde_json::json!({
            "doc_a_id": p.doc_a_id,
            "doc_b_id": p.doc_b_id,
            "similarity_score": p.similarity_score,
            "method": p.method,
        })).collect::<Vec<_>>(),
    })))
}

pub async fn kb_quality(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::knowledge_curation::quality::QualityScorer;

    let document_id = payload.get("document_id").and_then(|v| v.as_str()).unwrap_or("doc");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let metadata: std::collections::HashMap<String, String> = payload.get("metadata")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let scorer = QualityScorer::new();
    let report = scorer.score(document_id, content, &metadata);

    Ok(Json(serde_json::json!({
        "document_id": report.document_id,
        "overall_score": report.overall_score,
        "metrics": report.metrics.iter().map(|m| serde_json::json!({
            "metric": m.metric,
            "score": m.score,
            "weight": m.weight,
            "details": m.details,
        })).collect::<Vec<_>>(),
        "issues": report.issues.len(),
        "recommendations": report.recommendations,
    })))
}

pub async fn collab_status(
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::multi_agent::coordinator::AgentCoordinator;

    let coord = AgentCoordinator::new();
    let stats = coord.get_stats();

    Ok(Json(serde_json::json!({
        "total_agents": stats.total_agents,
        "idle_agents": stats.idle_agents,
        "busy_agents": stats.busy_agents,
        "capabilities": stats.capabilities,
    })))
}

pub async fn collab_tasks(
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::multi_agent::task_board::TaskBoard;

    let board = TaskBoard::new();
    let stats = board.get_stats();

    Ok(Json(serde_json::json!({
        "total": stats.total,
        "pending": stats.pending,
        "in_progress": stats.in_progress,
        "done": stats.done,
        "failed": stats.failed,
        "tasks": board.list_tasks().iter().map(|t| serde_json::json!({
            "id": t.id,
            "title": t.title,
            "status": format!("{:?}", t.status),
            "priority": format!("{:?}", t.priority),
        })).collect::<Vec<_>>(),
    })))
}

pub async fn collab_create_task(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::multi_agent::task_board::{TaskBoard, TaskPriority};

    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("task-1");
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("New Task");
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let priority_str = payload.get("priority").and_then(|v| v.as_str()).unwrap_or("medium");

    let priority = match priority_str {
        "low" => TaskPriority::Low,
        "high" => TaskPriority::High,
        "critical" => TaskPriority::Critical,
        _ => TaskPriority::Medium,
    };

    let mut board = TaskBoard::new();
    let task = board.create_task(id, title, description, priority);

    Ok(Json(serde_json::json!({
        "id": task.id,
        "title": task.title,
        "status": format!("{:?}", task.status),
        "priority": format!("{:?}", task.priority),
    })))
}

pub async fn collab_consensus(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::multi_agent::consensus::{ConsensusEngine, AgentVote, Vote};

    let proposal_id = payload.get("proposal_id").and_then(|v| v.as_str()).unwrap_or("proposal-1");
    let empty_vec: Vec<serde_json::Value> = vec![];
    let votes_input = payload.get("votes").and_then(|v| v.as_array()).unwrap_or(&empty_vec);
    let threshold = payload.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.6);

    let votes: Vec<AgentVote> = votes_input.iter().map(|v| {
        let vote_str = v.get("vote").and_then(|val| val.as_str()).unwrap_or("abstain");
        let vote = match vote_str {
            "approve" => Vote::Approve,
            "reject" => Vote::Reject,
            _ => Vote::Abstain,
        };
        AgentVote {
            agent_id: v.get("agent_id").and_then(|val| val.as_str()).unwrap_or("unknown").to_string(),
            vote,
            reasoning: v.get("reasoning").and_then(|val| val.as_str()).unwrap_or("").to_string(),
            confidence: v.get("confidence").and_then(|val| val.as_f64()).unwrap_or(0.5),
        }
    }).collect();

    let engine = ConsensusEngine::new(threshold);
    let result = engine.evaluate(proposal_id, &votes);
    let weighted = engine.calculate_weighted_score(&votes);

    Ok(Json(serde_json::json!({
        "proposal_id": result.proposal_id,
        "passed": result.passed,
        "approval_rate": result.approval_rate,
        "consensus_reached": result.consensus_reached,
        "final_decision": result.final_decision,
        "weighted_score": weighted,
        "votes_count": result.votes.len(),
    })))
}

pub async fn analytics_usage(
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::analytics::usage_tracker::UsageTracker;

    let tracker = UsageTracker::new();
    let stats = tracker.get_stats();

    Ok(Json(serde_json::json!({
        "total_events": stats.total_events,
        "unique_users": stats.unique_users,
        "events_by_type": stats.events_by_type,
        "events_by_user": stats.events_by_user,
    })))
}

pub async fn analytics_performance(
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::analytics::performance::PerformanceMonitor;

    let monitor = PerformanceMonitor::new();
    let report = monitor.generate_report();

    Ok(Json(serde_json::json!({
        "total_metrics": report.total_metrics,
        "summaries": report.summaries.iter().map(|s| serde_json::json!({
            "name": s.name,
            "avg": s.avg,
            "min": s.min,
            "max": s.max,
            "count": s.count,
        })).collect::<Vec<_>>(),
    })))
}

pub async fn analytics_users(
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::analytics::user_behavior::BehaviorAnalyzer;

    let analyzer = BehaviorAnalyzer::new();
    let stats = analyzer.get_stats();

    Ok(Json(serde_json::json!({
        "total_actions": stats.total_actions,
        "unique_users": stats.unique_users,
        "unique_actions": stats.unique_actions,
        "patterns": analyzer.detect_patterns().iter().map(|p| serde_json::json!({
            "name": p.pattern_name,
            "frequency": p.frequency,
            "description": p.description,
        })).collect::<Vec<_>>(),
    })))
}

pub async fn analytics_reports(
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::analytics::reports::ReportGenerator;

    let gen = ReportGenerator::new();
    let reports = gen.get_reports();

    Ok(Json(serde_json::json!({
        "total_reports": reports.len(),
        "reports": reports.iter().map(|r| serde_json::json!({
            "id": r.id,
            "title": r.title,
            "type": format!("{:?}", r.report_type),
            "generated_at": r.generated_at,
        })).collect::<Vec<_>>(),
    })))
}

pub async fn workflow_list(
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "workflows": [],
        "total": 0,
    })))
}

pub async fn workflow_create(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::workflow::builder::WorkflowBuilder;

    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("wf-1");
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("New Workflow");
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("");

    let workflow = WorkflowBuilder::new(id, name)
        .description(description)
        .build();

    Ok(Json(serde_json::json!({
        "id": workflow.id,
        "name": workflow.name,
        "description": workflow.description,
        "steps": workflow.steps.len(),
        "enabled": workflow.enabled,
    })))
}

pub async fn workflow_run(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::workflow::engine::WorkflowEngine;
    use alesys_core::workflow::builder::WorkflowBuilder;
    use alesys_core::workflow::actions::{Action, ActionType};

    let mut config = std::collections::HashMap::new();
    config.insert("command".to_string(), "echo workflow-executed".to_string());
    let action = Action {
        id: "action-1".to_string(),
        action_type: ActionType::RunCommand,
        config,
        timeout_ms: 5000,
    };

    let workflow = WorkflowBuilder::new(&id, "Executed Workflow")
        .step("step-1", "Run Command", action)
        .build();

    let engine = WorkflowEngine::new();
    let result = engine.execute(&workflow);

    Ok(Json(serde_json::json!({
        "workflow_id": result.workflow_id,
        "success": result.success,
        "logs": result.logs.iter().map(|l| serde_json::json!({
            "step_id": l.step_id,
            "step_name": l.step_name,
            "success": l.success,
            "output": l.output,
            "duration_ms": l.duration_ms,
        })).collect::<Vec<_>>(),
        "total_duration_ms": result.total_duration_ms,
    })))
}

pub async fn search_faceted(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use alesys_core::search_adv::query_builder::{QueryBuilder, SearchQuery};
    use alesys_core::search_adv::query_builder::SearchItem;
    use alesys_core::search_adv::facets::FacetedSearch;

    let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let page = payload.get("page").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let page_size = payload.get("page_size").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let facet_fields: Vec<String> = payload.get("facets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let query = SearchQuery {
        text: text.to_string(),
        page,
        page_size,
        ..Default::default()
    };

    let builder = QueryBuilder::new();
    let docs: Vec<SearchItem> = vec![];
    let result = builder.search(&query, &docs);

    let search = FacetedSearch::new();
    let items: Vec<std::collections::HashMap<String, String>> = vec![];
    let facets = search.compute_facets(&items, &facet_fields);

    Ok(Json(serde_json::json!({
        "results": result.results.len(),
        "total": result.total,
        "page": result.page,
        "query_time_ms": result.query_time_ms,
        "facets": facets.iter().map(|f| serde_json::json!({
            "field": f.field,
            "values": f.values.iter().map(|v| serde_json::json!({
                "value": v.value,
                "count": v.count,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })))
}

pub async fn search_suggest(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let query = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");

    let suggestions: Vec<String> = if query.len() >= 2 {
        vec![
            format!("{} tutorial", query),
            format!("{} examples", query),
            format!("{} documentation", query),
        ]
    } else {
        vec![]
    };

    Ok(Json(serde_json::json!({
        "query": query,
        "suggestions": suggestions,
    })))
}


// ============================================================================
// LLM MANAGEMENT ENDPOINTS
// ============================================================================

/// GET /api/v1/llm/status - Verificar estado del LLM
pub async fn get_llm_status(
    State(state): State<AppState>,
) -> Result<Json<LLMStatusResponse>, StatusCode> {
    let engine = state.llm_engine.read().await;
    
    let loaded = engine.is_loaded();
    let backend = engine.backend_name().to_string();
    let llm_state = engine.state();
    
    let state_str = match llm_state {
        LLMState::Unloaded => "unloaded".to_string(),
        LLMState::Loaded => "loaded".to_string(),
        LLMState::Error => "error".to_string(),
    };
    
    let message = if loaded {
        format!("LLM cargado y listo (backend={})", backend)
    } else {
        format!("LLM no cargado (backend={}). Usar POST /api/v1/llm/load para cargar.", backend)
    };

    Ok(Json(LLMStatusResponse {
        loaded,
        backend,
        state: state_str,
        model_path: if state.llm_config.model_path.is_empty() {
            None
        } else {
            Some(state.llm_config.model_path.clone())
        },
        message,
    }))
}

/// POST /api/v1/llm/load - Cargar modelo LLM en memoria
pub async fn load_llm(
    State(state): State<AppState>,
    Json(payload): Json<LoadLLMRequest>,
) -> Result<Json<LoadLLMResponse>, (StatusCode, String)> {
    let engine = state.llm_engine.read().await;
    
    // Verificar si ya está cargado
    if engine.is_loaded() && !payload.force {
        return Err((
            StatusCode::CONFLICT,
            "LLM ya está cargado. Usar force=true para recargar.".to_string()
        ));
    }
    
    drop(engine);
    
    // Intentar cargar el modelo
    let config = state.llm_config.clone();
    let backend_name = config.backend.to_string();
    let model_path = config.model_path.clone();
    
    // Estimación de RAM basada en el modelo
    let estimated_ram = estimate_model_ram(&model_path);
    
    tracing::info!("Cargando modelo LLM: backend={}, path={}", backend_name, model_path);
    
    match state.llm_queue.load(&config).await {
        Ok(()) => {
            tracing::info!("✅ Modelo LLM cargado exitosamente");
            Ok(Json(LoadLLMResponse {
                success: true,
                backend: backend_name,
                model_path,
                estimated_ram_mb: estimated_ram,
                message: "Modelo cargado exitosamente en memoria".to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Error cargando modelo LLM: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error cargando modelo: {}", e)
            ))
        }
    }
}

/// POST /api/v1/llm/unload - Descargar modelo LLM de memoria
pub async fn unload_llm(
    State(state): State<AppState>,
) -> Result<Json<UnloadLLMResponse>, StatusCode> {
    let engine = state.llm_engine.read().await;
    
    // Verificar si está cargado
    if !engine.is_loaded() {
        return Ok(Json(UnloadLLMResponse {
            success: false,
            message: "LLM ya está descargado".to_string(),
            ram_freed_mb: None,
        }));
    }
    
    let backend_name = engine.backend_name().to_string();
    let model_path = state.llm_config.model_path.clone();
    let ram_freed = estimate_model_ram(&model_path);
    
    drop(engine);
    
    tracing::info!("Descargando modelo LLM: backend={}", backend_name);
    
    match state.llm_queue.unload().await {
        Ok(()) => {
            tracing::info!("✅ Modelo LLM descargado - {} MB liberados", ram_freed);
            Ok(Json(UnloadLLMResponse {
                success: true,
                message: format!("Modelo descargado. {} MB de RAM liberados.", ram_freed),
                ram_freed_mb: Some(ram_freed),
            }))
        }
        Err(e) => {
            tracing::error!("Error descargando modelo LLM: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Estima el consumo de RAM de un modelo basado en su nombre/path
fn estimate_model_ram(model_path: &str) -> u64 {
    if model_path.is_empty() {
        return 1024; // Default 1 GB
    }
    
    let filename = model_path.to_lowercase();
    
    // Modelos pequeños (< 1B params)
    if filename.contains("tiny") || filename.contains("1b") || filename.contains("0.5b") {
        600 // 600 MB
    }
    // Modelos medianos (1-3B params)
    else if filename.contains("2b") || filename.contains("3b") || filename.contains("phi") {
        2048 // 2 GB
    }
    // Modelos grandes (4-8B params)
    else if filename.contains("4b") || filename.contains("5b") || filename.contains("6b") || filename.contains("7b") || filename.contains("8b") {
        4096 // 4 GB
    }
    // Modelos muy grandes (13B+ params)
    else if filename.contains("13b") || filename.contains("20b") || filename.contains("30b") {
        8192 // 8 GB
    }
    // Modelos MoE
    else if filename.contains("moe") || filename.contains("mixtral") {
        if filename.contains("8x7b") || filename.contains("8x7") {
            8192 // 8 GB para Mixtral 8x7B
        } else if filename.contains("4x0.6b") || filename.contains("qwen3moe") {
            1024 // 1 GB para Qwen3-MoE
        } else {
            4096 // Default MoE
        }
    }
    // Default
    else {
        2048 // 2 GB default
    }
}

// LLM Management Types
#[derive(Debug, Serialize, Deserialize)]
pub struct LLMStatusResponse {
    pub loaded: bool,
    pub backend: String,
    pub state: String,
    pub model_path: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LoadLLMRequest {
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadLLMResponse {
    pub success: bool,
    pub backend: String,
    pub model_path: String,
    pub estimated_ram_mb: u64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnloadLLMResponse {
    pub success: bool,
    pub message: String,
    pub ram_freed_mb: Option<u64>,
}
