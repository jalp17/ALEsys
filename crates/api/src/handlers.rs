//! Handlers de los endpoints HTTP

use crate::state::AppState;
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
// Phase 7: Execute & Editor Handlers
// =============================================================================

/// Request para POST /api/v1/execute
#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub code: String,
    pub language: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_memory")]
    pub memory_limit_mb: u64,
}

fn default_timeout() -> u64 {
    30_000
}
fn default_memory() -> u64 {
    256
}

/// Response de POST /api/v1/execute
#[derive(Serialize)]
pub struct ExecuteResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub timed_out: bool,
    pub language: String,
}

/// Handler para POST /api/v1/execute
#[cfg(feature = "sandbox")]
pub async fn execute_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use alesys_core::sandbox::{CodeSandbox, SandboxConfig};

    tracing::info!(
        "Execute request: language={}, timeout={}ms",
        payload.language,
        payload.timeout_ms
    );

    let language = payload.language.parse().map_err(|e| ApiError {
        error: format!("Unsupported language: {}", e),
        code: "VALIDATION".into(),
    })?;

    let config = SandboxConfig {
        timeout_ms: payload.timeout_ms,
        memory_limit_mb: payload.memory_limit_mb,
        ..Default::default()
    };

    let sandbox = CodeSandbox::with_config(config);
    let result = sandbox
        .execute(&payload.code, language)
        .await
        .map_err(|e| {
            tracing::error!("Error ejecutando codigo: {}", e);
            ApiError {
                error: format!("Execution failed: {}", e),
                code: "INTERNAL".into(),
            }
        })?;

    Ok(Json(ExecuteResponse {
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
        execution_time_ms: result.execution_time_ms,
        timed_out: result.timed_out,
        language: result.language.to_string(),
    }))
}

/// Request para POST /api/v1/modify
#[derive(Deserialize)]
pub struct ModifyRequest {
    pub path: String,
    pub old_content: String,
    pub new_content: String,
}

/// Request para GET /api/v1/files
#[derive(Deserialize)]
pub struct ListFilesRequest {
    #[serde(default)]
    pub path: Option<String>,
}

/// Request para POST /api/v1/files
#[derive(Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String,
}

/// Handler para GET /api/v1/files
#[cfg(feature = "editor")]
pub async fn list_files_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<ListFilesRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use alesys_core::editor::FileEditor;

    let editor = FileEditor::new(state.project_dir.clone());
    let path = params.path.unwrap_or_default();

    let entries = editor.list_files(&path).map_err(|e| ApiError {
        error: format!("Error listing files: {}", e),
        code: "INTERNAL".into(),
    })?;

    Ok(Json(serde_json::json!({ "entries": entries })))
}

/// Handler para GET /api/v1/files/:path*
#[cfg(feature = "editor")]
pub async fn read_file_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    use alesys_core::editor::FileEditor;

    let editor = FileEditor::new(state.project_dir.clone());

    let content = editor.read_file(&path).map_err(|e| ApiError {
        error: format!("Error reading file: {}", e),
        code: "NOT_FOUND".into(),
    })?;

    Ok(Json(serde_json::json!({
        "path": path,
        "content": content,
    })))
}

/// Handler para POST /api/v1/files
#[cfg(feature = "editor")]
pub async fn write_file_handler(
    State(state): State<AppState>,
    Json(payload): Json<WriteFileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use alesys_core::editor::FileEditor;

    let editor = FileEditor::new(state.project_dir.clone());

    let result = editor
        .write_file(&payload.path, &payload.content)
        .map_err(|e| ApiError {
            error: format!("Error writing file: {}", e),
            code: "INTERNAL".into(),
        })?;

    Ok(Json(result))
}

/// Handler para POST /api/v1/modify
#[cfg(feature = "editor")]
pub async fn modify_file_handler(
    State(state): State<AppState>,
    Json(payload): Json<ModifyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use alesys_core::editor::FileEditor;

    let editor = FileEditor::new(state.project_dir.clone());

    let diff = editor
        .modify_file(&payload.path, &payload.old_content, &payload.new_content)
        .map_err(|e| ApiError {
            error: format!("Error modifying file: {}", e),
            code: "INTERNAL".into(),
        })?;

    Ok(Json(diff))
}
