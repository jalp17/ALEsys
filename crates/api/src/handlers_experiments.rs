//! Experiment Runs API Handlers (ELN)

use crate::auth::{Claims, Permission};
use crate::state::AppState;
use crate::handlers::ApiError;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRun {
    pub id: String,
    pub project_id: Option<String>,
    pub name: String,
    pub protocol: Option<String>,
    pub params: serde_json::Value,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub result_summary: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub project_id: Option<String>,
    pub name: String,
    pub protocol: Option<String>,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRunRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub result_summary: Option<String>,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub project_id: Option<String>,
    pub status: Option<String>,
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

pub async fn list_runs_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<Vec<ExperimentRun>>, ApiError> {
    let mut sql = String::from("SELECT id, project_id, name, protocol, params, status, started_at, finished_at, result_summary, created_by, created_at, updated_at FROM experiment_runs WHERE 1=1");
    let mut bind_count = 0i32;
    if q.project_id.is_some() { bind_count += 1; sql.push_str(&format!(" AND project_id = ${}", bind_count)); }
    if q.status.is_some() { bind_count += 1; sql.push_str(&format!(" AND status = ${}", bind_count)); }
    sql.push_str(" ORDER BY created_at DESC");

    let mut query = sqlx::query(&sql);
    if let Some(pid) = &q.project_id {
        query = query.bind(Uuid::parse_str(pid).unwrap_or_default());
    }
    if let Some(status) = &q.status {
        query = query.bind(status);
    }
    let rows = query.fetch_all(&state.db).await.map_err(db_error)?;
    let mut runs = Vec::with_capacity(rows.len());
    for row in rows {
        runs.push(ExperimentRun {
            id: row.get::<Uuid, _>("id").to_string(),
            project_id: row.get::<Option<Uuid>, _>("project_id").map(|u| u.to_string()),
            name: row.get("name"),
            protocol: row.get("protocol"),
            params: row.get("params"),
            status: row.get("status"),
            started_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("started_at").map(|d| d.to_rfc3339()),
            finished_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("finished_at").map(|d| d.to_rfc3339()),
            result_summary: row.get("result_summary"),
            created_by: row.get("created_by"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        });
    }
    Ok(Json(runs))
}

pub async fn create_run_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateRunRequest>,
) -> Result<Json<ExperimentRun>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let id = Uuid::new_v4();
    let project_id = payload.project_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let params = payload.params.unwrap_or(serde_json::json!({}));
    let row = sqlx::query(
        r#"INSERT INTO experiment_runs (id, project_id, name, protocol, params, status, started_at, created_by)
           VALUES ($1, $2, $3, $4, $5, 'running', NOW(), $6)
           RETURNING id, project_id, name, protocol, params, status, started_at, finished_at, result_summary, created_by, created_at, updated_at"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(&payload.name)
    .bind(&payload.protocol)
    .bind(&params)
    .bind(&claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;
    Ok(Json(ExperimentRun {
        id: row.get::<Uuid, _>("id").to_string(),
        project_id: row.get::<Option<Uuid>, _>("project_id").map(|u| u.to_string()),
        name: row.get("name"),
        protocol: row.get("protocol"),
        params: row.get("params"),
        status: row.get("status"),
        started_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("started_at").map(|d| d.to_rfc3339()),
        finished_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("finished_at").map(|d| d.to_rfc3339()),
        result_summary: row.get("result_summary"),
        created_by: row.get("created_by"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn update_run_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(payload): Json<UpdateRunRequest>,
) -> Result<Json<ExperimentRun>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let rid = Uuid::parse_str(&run_id).map_err(|_| not_found("Invalid run id"))?;
    let mut sets = Vec::new();
    let mut bind_idx = 1i32;
    if payload.name.is_some() { sets.push(format!("name = ${}", bind_idx)); bind_idx += 1; }
    if payload.status.is_some() { sets.push(format!("status = ${}", bind_idx)); bind_idx += 1; }
    if payload.result_summary.is_some() { sets.push(format!("result_summary = ${}", bind_idx)); bind_idx += 1; }
    if payload.params.is_some() { sets.push(format!("params = ${}", bind_idx)); bind_idx += 1; }
    if payload.status.as_deref() == Some("completed") || payload.status.as_deref() == Some("failed") {
        sets.push("finished_at = NOW()".to_string());
    }
    sets.push("updated_at = NOW()".to_string());
    let sql = format!(
        "UPDATE experiment_runs SET {} WHERE id = ${} RETURNING id, project_id, name, protocol, params, status, started_at, finished_at, result_summary, created_by, created_at, updated_at",
        sets.join(", "), bind_idx
    );
    let mut q = sqlx::query(&sql);
    if let Some(name) = &payload.name { q = q.bind(name); }
    if let Some(status) = &payload.status { q = q.bind(status); }
    if let Some(rs) = &payload.result_summary { q = q.bind(rs); }
    if let Some(params) = &payload.params { q = q.bind(params); }
    q = q.bind(rid);
    let row = q.fetch_optional(&state.db).await.map_err(db_error)?
        .ok_or_else(|| not_found("Run not found"))?;
    Ok(Json(ExperimentRun {
        id: row.get::<Uuid, _>("id").to_string(),
        project_id: row.get::<Option<Uuid>, _>("project_id").map(|u| u.to_string()),
        name: row.get("name"),
        protocol: row.get("protocol"),
        params: row.get("params"),
        status: row.get("status"),
        started_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("started_at").map(|d| d.to_rfc3339()),
        finished_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("finished_at").map(|d| d.to_rfc3339()),
        result_summary: row.get("result_summary"),
        created_by: row.get("created_by"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn delete_run_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let rid = Uuid::parse_str(&run_id).map_err(|_| not_found("Invalid run id"))?;
    let result = sqlx::query("DELETE FROM experiment_runs WHERE id = $1").bind(rid).execute(&state.db).await.map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(not_found("Run not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
