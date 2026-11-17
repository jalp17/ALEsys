//! Workspaces (cuadernos de temas) — AnythingLLM-style
//! Inspired by AnythingLLM workspaces/threads.

use crate::auth::{Claims, Permission};
use crate::state::AppState;
use crate::handlers::ApiError;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub llm_model: Option<String>,
    pub temperature: f64,
    pub top_p: f64,
    pub system_prompt: Option<String>,
    pub embedder_mode: String,
    pub owner_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDocument {
    pub id: String,
    pub workspace_id: String,
    pub document_id: Option<i32>,
    pub title: String,
    pub mode: String,
    pub content: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceThread {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub mode: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub llm_model: Option<String>,
    pub temperature: Option<f64>,
    pub system_prompt: Option<String>,
    pub embedder_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub llm_model: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub system_prompt: Option<String>,
    pub embedder_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddDocumentRequest {
    pub title: String,
    pub mode: Option<String>,
    pub content: Option<String>,
    pub document_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateThreadRequest {
    pub name: Option<String>,
    pub mode: Option<String>,
}

fn db_error(e: sqlx::Error) -> ApiError {
    ApiError { error: format!("Database error: {}", e), code: "DB_ERROR".to_string() }
}
fn forbidden() -> ApiError {
    ApiError { error: "Forbidden".to_string(), code: "FORBIDDEN".to_string() }
}
fn not_found(msg: &str) -> ApiError {
    ApiError { error: msg.to_string(), code: "NOT_FOUND".to_string() }
}

// === Workspaces ===

pub async fn list_workspaces_handler(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<Workspace>>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, name, description, llm_model, temperature, top_p, system_prompt, embedder_mode, owner_id, created_at, updated_at FROM workspaces ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_error)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(Workspace {
            id: row.get::<Uuid, _>("id").to_string(),
            name: row.get("name"),
            description: row.get("description"),
            llm_model: row.get("llm_model"),
            temperature: row.get("temperature"),
            top_p: row.get("top_p"),
            system_prompt: row.get("system_prompt"),
            embedder_mode: row.get("embedder_mode"),
            owner_id: row.get("owner_id"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        });
    }
    Ok(Json(out))
}

pub async fn create_workspace_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let id = Uuid::new_v4();
    let embedder_mode = payload.embedder_mode.unwrap_or_else(|| "embed".to_string());
    let temperature = payload.temperature.unwrap_or(0.7);
    let row = sqlx::query(
        r#"INSERT INTO workspaces (id, name, description, llm_model, temperature, system_prompt, embedder_mode, owner_id)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
           RETURNING id, name, description, llm_model, temperature, top_p, system_prompt, embedder_mode, owner_id, created_at, updated_at"#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(payload.description.unwrap_or_default())
    .bind(&payload.llm_model)
    .bind(temperature)
    .bind(&payload.system_prompt)
    .bind(&embedder_mode)
    .bind(&claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;
    Ok(Json(Workspace {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        description: row.get("description"),
        llm_model: row.get("llm_model"),
        temperature: row.get("temperature"),
        top_p: row.get("top_p"),
        system_prompt: row.get("system_prompt"),
        embedder_mode: row.get("embedder_mode"),
        owner_id: row.get("owner_id"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn get_workspace_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Path(ws_id): Path<String>,
) -> Result<Json<Workspace>, ApiError> {
    let wid = Uuid::parse_str(&ws_id).map_err(|_| not_found("Invalid workspace id"))?;
    let row = sqlx::query(
        "SELECT id, name, description, llm_model, temperature, top_p, system_prompt, embedder_mode, owner_id, created_at, updated_at FROM workspaces WHERE id = $1",
    )
    .bind(wid)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?
    .ok_or_else(|| not_found("Workspace not found"))?;
    Ok(Json(Workspace {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        description: row.get("description"),
        llm_model: row.get("llm_model"),
        temperature: row.get("temperature"),
        top_p: row.get("top_p"),
        system_prompt: row.get("system_prompt"),
        embedder_mode: row.get("embedder_mode"),
        owner_id: row.get("owner_id"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn update_workspace_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(ws_id): Path<String>,
    Json(payload): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let wid = Uuid::parse_str(&ws_id).map_err(|_| not_found("Invalid workspace id"))?;
    let mut sets = Vec::new();
    let mut bind_idx = 1i32;
    if payload.name.is_some() { sets.push(format!("name = ${}", bind_idx)); bind_idx += 1; }
    if payload.description.is_some() { sets.push(format!("description = ${}", bind_idx)); bind_idx += 1; }
    if payload.llm_model.is_some() { sets.push(format!("llm_model = ${}", bind_idx)); bind_idx += 1; }
    if payload.temperature.is_some() { sets.push(format!("temperature = ${}", bind_idx)); bind_idx += 1; }
    if payload.top_p.is_some() { sets.push(format!("top_p = ${}", bind_idx)); bind_idx += 1; }
    if payload.system_prompt.is_some() { sets.push(format!("system_prompt = ${}", bind_idx)); bind_idx += 1; }
    if payload.embedder_mode.is_some() { sets.push(format!("embedder_mode = ${}", bind_idx)); bind_idx += 1; }
    sets.push("updated_at = NOW()".to_string());
    let sql = format!(
        "UPDATE workspaces SET {} WHERE id = ${} RETURNING id, name, description, llm_model, temperature, top_p, system_prompt, embedder_mode, owner_id, created_at, updated_at",
        sets.join(", "), bind_idx
    );
    let mut q = sqlx::query(&sql);
    if let Some(v) = &payload.name { q = q.bind(v); }
    if let Some(v) = &payload.description { q = q.bind(v); }
    if let Some(v) = &payload.llm_model { q = q.bind(v); }
    if let Some(v) = payload.temperature { q = q.bind(v); }
    if let Some(v) = payload.top_p { q = q.bind(v); }
    if let Some(v) = &payload.system_prompt { q = q.bind(v); }
    if let Some(v) = &payload.embedder_mode { q = q.bind(v); }
    q = q.bind(wid);
    let row = q.fetch_optional(&state.db).await.map_err(db_error)?
        .ok_or_else(|| not_found("Workspace not found"))?;
    Ok(Json(Workspace {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        description: row.get("description"),
        llm_model: row.get("llm_model"),
        temperature: row.get("temperature"),
        top_p: row.get("top_p"),
        system_prompt: row.get("system_prompt"),
        embedder_mode: row.get("embedder_mode"),
        owner_id: row.get("owner_id"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn delete_workspace_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(ws_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let wid = Uuid::parse_str(&ws_id).map_err(|_| not_found("Invalid workspace id"))?;
    let result = sqlx::query("DELETE FROM workspaces WHERE id = $1").bind(wid).execute(&state.db).await.map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(not_found("Workspace not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// === Documents (attach/embed) ===

pub async fn list_workspace_documents_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Path(ws_id): Path<String>,
) -> Result<Json<Vec<WorkspaceDocument>>, ApiError> {
    let wid = Uuid::parse_str(&ws_id).map_err(|_| not_found("Invalid workspace id"))?;
    let rows = sqlx::query(
        "SELECT id, workspace_id, document_id, title, mode, content, created_at FROM workspace_documents WHERE workspace_id = $1 ORDER BY created_at DESC",
    )
    .bind(wid)
    .fetch_all(&state.db)
    .await
    .map_err(db_error)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(WorkspaceDocument {
            id: row.get::<Uuid, _>("id").to_string(),
            workspace_id: row.get::<Uuid, _>("workspace_id").to_string(),
            document_id: row.get("document_id"),
            title: row.get("title"),
            mode: row.get("mode"),
            content: row.get("content"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        });
    }
    Ok(Json(out))
}

pub async fn add_workspace_document_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(ws_id): Path<String>,
    Json(payload): Json<AddDocumentRequest>,
) -> Result<Json<WorkspaceDocument>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let wid = Uuid::parse_str(&ws_id).map_err(|_| not_found("Invalid workspace id"))?;
    let id = Uuid::new_v4();
    let mode = payload.mode.unwrap_or_else(|| "embed".to_string());
    let row = sqlx::query(
        r#"INSERT INTO workspace_documents (id, workspace_id, document_id, title, mode, content)
           VALUES ($1,$2,$3,$4,$5,$6)
           RETURNING id, workspace_id, document_id, title, mode, content, created_at"#,
    )
    .bind(id)
    .bind(wid)
    .bind(payload.document_id)
    .bind(&payload.title)
    .bind(&mode)
    .bind(&payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;
    Ok(Json(WorkspaceDocument {
        id: row.get::<Uuid, _>("id").to_string(),
        workspace_id: row.get::<Uuid, _>("workspace_id").to_string(),
        document_id: row.get("document_id"),
        title: row.get("title"),
        mode: row.get("mode"),
        content: row.get("content"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
    }))
}

pub async fn delete_workspace_document_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path((ws_id, doc_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let did = Uuid::parse_str(&doc_id).map_err(|_| not_found("Invalid doc id"))?;
    let result = sqlx::query("DELETE FROM workspace_documents WHERE id = $1").bind(did).execute(&state.db).await.map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(not_found("Document not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// === Threads ===

pub async fn list_workspace_threads_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Path(ws_id): Path<String>,
) -> Result<Json<Vec<WorkspaceThread>>, ApiError> {
    let wid = Uuid::parse_str(&ws_id).map_err(|_| not_found("Invalid workspace id"))?;
    let rows = sqlx::query(
        "SELECT id, workspace_id, name, mode, created_at, updated_at FROM workspace_threads WHERE workspace_id = $1 ORDER BY created_at DESC",
    )
    .bind(wid)
    .fetch_all(&state.db)
    .await
    .map_err(db_error)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(WorkspaceThread {
            id: row.get::<Uuid, _>("id").to_string(),
            workspace_id: row.get::<Uuid, _>("workspace_id").to_string(),
            name: row.get("name"),
            mode: row.get("mode"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        });
    }
    Ok(Json(out))
}

pub async fn create_workspace_thread_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(ws_id): Path<String>,
    Json(payload): Json<CreateThreadRequest>,
) -> Result<Json<WorkspaceThread>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let wid = Uuid::parse_str(&ws_id).map_err(|_| not_found("Invalid workspace id"))?;
    let id = Uuid::new_v4();
    let name = payload.name.unwrap_or_else(|| "New Thread".to_string());
    let mode = payload.mode.unwrap_or_else(|| "conversation".to_string());
    let row = sqlx::query(
        r#"INSERT INTO workspace_threads (id, workspace_id, name, mode)
           VALUES ($1,$2,$3,$4)
           RETURNING id, workspace_id, name, mode, created_at, updated_at"#,
    )
    .bind(id)
    .bind(wid)
    .bind(&name)
    .bind(&mode)
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;
    Ok(Json(WorkspaceThread {
        id: row.get::<Uuid, _>("id").to_string(),
        workspace_id: row.get::<Uuid, _>("workspace_id").to_string(),
        name: row.get("name"),
        mode: row.get("mode"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn delete_workspace_thread_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path((ws_id, thread_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let tid = Uuid::parse_str(&thread_id).map_err(|_| not_found("Invalid thread id"))?;
    let result = sqlx::query("DELETE FROM workspace_threads WHERE id = $1").bind(tid).execute(&state.db).await.map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(not_found("Thread not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
