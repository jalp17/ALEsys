use crate::auth::{Claims, Permission};
use crate::state::AppState;
use crate::handlers::ApiError;
use axum::extract::{Json, Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Serialize)]
pub struct IngestionHistoryItem {
    pub job_id: String,
    pub pdf_path: String,
    pub topic: String,
    pub status: String,
    pub progress: f64,
    pub message: Option<String>,
    pub output_dir: Option<String>,
    pub markdown_path: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub topic: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_ingestion_history_handler(
    claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<IngestionHistoryItem>>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionRead) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:read permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0).max(0);

    let mut sql = String::from("SELECT id, pdf_path, topic, status, progress, message, output_dir, markdown_path, created_at FROM ingestion_jobs WHERE 1=1");
    let mut bind_count = 0;

    if query.topic.is_some() {
        bind_count += 1;
        sql.push_str(&format!(" AND topic = ${}", bind_count));
    }
    if query.status.is_some() {
        bind_count += 1;
        sql.push_str(&format!(" AND status = ${}", bind_count));
    }

    sql.push_str(" ORDER BY created_at DESC");
    bind_count += 1;
    sql.push_str(&format!(" LIMIT ${}", bind_count));
    bind_count += 1;
    sql.push_str(&format!(" OFFSET ${}", bind_count));

    let mut q = sqlx::query(&sql);

    if let Some(ref topic) = query.topic {
        q = q.bind(topic);
    }
    if let Some(ref status) = query.status {
        q = q.bind(status);
    }

    let rows = q
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e| ApiError {
            error: e.to_string(),
            code: "DB_ERROR".to_string(),
        })?;

    let items: Vec<IngestionHistoryItem> = rows
        .into_iter()
        .map(|r| {
            let id: uuid::Uuid = r.get("id");
            IngestionHistoryItem {
                job_id: id.to_string(),
                pdf_path: r.get("pdf_path"),
                topic: r.get("topic"),
                status: r.get("status"),
                progress: r.get("progress"),
                message: r.get("message"),
                output_dir: r.get("output_dir"),
                markdown_path: r.get("markdown_path"),
                created_at: r.get::<Option<DateTime<Utc>>, _>("created_at").map(|dt| dt.to_rfc3339()),
            }
        })
        .collect();

    Ok(Json(items))
}

#[derive(Debug, Serialize)]
pub struct DocumentFragment {
    pub fragment_id: i32,
    pub contenido: String,
    pub indice_orden: Option<i32>,
    pub creado_en: Option<String>,
}

pub async fn get_document_fragments_handler(
    claims: Claims,
    State(state): State<AppState>,
    axum::extract::Path(document_id): axum::extract::Path<i32>,
) -> Result<Json<Vec<DocumentFragment>>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionRead) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:read permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let rows = sqlx::query_as::<_, (i32, String, Option<i32>, Option<DateTime<Utc>>)>(
        "SELECT id, contenido, indice_orden, creado_en FROM fragmentos WHERE documento_id = $1 ORDER BY indice_orden ASC NULLS LAST, id ASC"
    )
    .bind(document_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError {
        error: e.to_string(),
        code: "DB_ERROR".to_string(),
    })?;

    let fragments: Vec<DocumentFragment> = rows
        .into_iter()
        .map(|(id, contenido, indice_orden, creado_en)| DocumentFragment {
            fragment_id: id,
            contenido,
            indice_orden,
            creado_en: creado_en.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    Ok(Json(fragments))
}
