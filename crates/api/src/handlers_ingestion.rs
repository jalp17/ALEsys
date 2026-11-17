//! Ingestion API Handlers (Phase 29)

use alesys_core::ingestion::{IngestionJob, IngestionConfig, IngestionMode};
use alesys_core::bibliography::Citation;
use crate::auth::{Claims, Permission};
use crate::state::{AppState, IngestionSemaphore};
use crate::handlers::ApiError;
use axum::{
    extract::{Json, Path, Request, State},
    http::request::Parts,
};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct IngestPdfRequest {
    pub pdf_path: String,
    #[serde(default = "default_topic")]
    pub topic: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub mode: IngestionMode,
    #[serde(default)]
    pub force_fallback: bool,
    #[serde(default = "default_ocr_langs")]
    pub ocr_languages: Vec<String>,
    #[serde(default = "default_true")]
    pub extract_formulas: bool,
    #[serde(default = "default_true")]
    pub extract_tables: bool,
}

fn default_topic() -> String { "uncategorized".to_string() }
fn default_ocr_langs() -> Vec<String> { vec!["en".to_string(), "es".to_string()] }
fn default_true() -> bool { true }

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub job_id: String,
    pub success: bool,
    pub mode: IngestionMode,
    pub database_generated: bool,
    pub output_dir: Option<String>,
    pub markdown_path: Option<String>,
    pub images_dir: Option<String>,
    pub database_path: Option<String>,
    pub citations_count: usize,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

pub async fn ingest_pdf_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<IngestPdfRequest>,
) -> Result<Json<IngestResponse>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let _permit = state.ingestion_semaphore.acquire(&claims.sub).await;

    let job = IngestionJob {
        id: Uuid::new_v4(),
        pdf_path: PathBuf::from(&payload.pdf_path),
        topic: payload.topic,
        session_id: payload.session_id.and_then(|s| Uuid::parse_str(&s).ok()),
        mode: payload.mode,
        force_fallback: payload.force_fallback,
        ocr_languages: payload.ocr_languages,
        extract_formulas: payload.extract_formulas,
        extract_tables: payload.extract_tables,
    };
    let job_id = job.id;

    let _ = sqlx::query(
        "INSERT INTO ingestion_jobs (id, pdf_path, topic, status, progress) VALUES ($1, $2, $3, 'processing', 0.0)"
    )
    .bind(job_id)
    .bind(job.pdf_path.to_string_lossy().as_ref())
    .bind(&job.topic)
    .execute(&state.db)
    .await;

    let config = state.ingestion_config.clone();
    let processor = alesys_core::ingestion::PDFProcessor::new_with_dir(config.model_dir, config.max_parallel);

    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(32);
    let state_graphrag = state.graphrag.clone();

    let result = processor
        .process_with_progress_and_graphrag(job, progress_tx, Some(&state_graphrag), Some(&state.db))
        .await;

    if result.is_ok() {
        while let Some(_progress) = progress_rx.recv().await {}
    }

    let final_status = if result.is_ok() { "completed" } else { "failed" };
    let _ = sqlx::query(
        "UPDATE ingestion_jobs SET status = $1, progress = CASE WHEN $1 = 'completed' THEN 100.0 ELSE progress END WHERE id = $2"
    )
    .bind(final_status)
    .bind(job_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(result) => Ok(Json(IngestResponse {
            job_id: result.job_id.to_string(),
            success: result.success,
            mode: result.mode,
            database_generated: result.database_generated,
            output_dir: Some(result.output_dir.to_string_lossy().into_owned()),
            markdown_path: Some(result.markdown_path.to_string_lossy().into_owned()),
            images_dir: Some(result.images_dir.to_string_lossy().into_owned()),
            database_path: result.database_path.map(|p| p.to_string_lossy().into_owned()),
            citations_count: result.citations.len(),
            warnings: result.warnings,
            error: result.error,
        })),
        Err(e) => Err(ApiError {
            error: e.to_string(),
            code: "INGESTION_ERROR".to_string(),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct IngestBatchRequest {
    pub pdf_paths: Vec<String>,
    #[serde(default = "default_topic")]
    pub topic: String,
    #[serde(default)]
    pub mode: IngestionMode,
    #[serde(default)]
    pub parallel: usize,
}

pub async fn ingest_batch_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<IngestBatchRequest>,
) -> Result<Json<Vec<IngestResponse>>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let _permit = state.ingestion_semaphore.acquire(&claims.sub).await;

    let jobs: Vec<IngestionJob> = payload.pdf_paths.into_iter().map(|path| IngestionJob {
        id: Uuid::new_v4(),
        pdf_path: PathBuf::from(&path),
        topic: payload.topic.clone(),
        mode: payload.mode.clone(),
        ..Default::default()
    }).collect();

    let job_ids: Vec<Uuid> = jobs.iter().map(|j| j.id).collect();

    for job in &jobs {
        let _ = sqlx::query(
            "INSERT INTO ingestion_jobs (id, pdf_path, topic, status, progress) VALUES ($1, $2, $3, 'processing', 0.0)"
        )
        .bind(job.id)
        .bind(job.pdf_path.to_string_lossy().as_ref())
        .bind(&job.topic)
        .execute(&state.db)
        .await;
    }

    let config = state.ingestion_config.clone();
    let processor = alesys_core::ingestion::PDFProcessor::new_with_dir(config.model_dir, payload.parallel.max(1));

    let result = processor.process_batch(jobs, Some(&state.graphrag), Some(&state.db)).await;

    let final_status = if result.is_ok() { "completed" } else { "failed" };
    for job_id in job_ids {
        let _ = sqlx::query(
            "UPDATE ingestion_jobs SET status = $1, progress = CASE WHEN $1 = 'completed' THEN 100.0 ELSE progress END WHERE id = $2"
        )
        .bind(final_status)
        .bind(job_id)
        .execute(&state.db)
        .await;
    }

    match result {
        Ok(results) => {
            let responses: Vec<IngestResponse> = results.into_iter().map(|r| IngestResponse {
                job_id: r.job_id.to_string(),
                success: r.success,
                mode: r.mode,
                database_generated: r.database_generated,
                output_dir: Some(r.output_dir.to_string_lossy().into_owned()),
                markdown_path: Some(r.markdown_path.to_string_lossy().into_owned()),
                images_dir: Some(r.images_dir.to_string_lossy().into_owned()),
                database_path: r.database_path.map(|p| p.to_string_lossy().into_owned()),
                citations_count: r.citations.len(),
                warnings: r.warnings,
                error: r.error,
            }).collect();
            Ok(Json(responses))
        }
        Err(e) => Err(ApiError {
            error: e.to_string(),
            code: "INGESTION_BATCH_ERROR".to_string(),
        }),
    }
}

pub async fn ingest_status_handler(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let row = sqlx::query(
        "SELECT id, pdf_path, topic, status, progress, message, output_dir, markdown_path, created_at FROM ingestion_jobs WHERE id = $1"
    )
    .bind(job_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError {
        error: e.to_string(),
        code: "DB_ERROR".to_string(),
    })?;

    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "job_id": r.get::<uuid::Uuid, _>("id").to_string(),
            "pdf_path": r.get::<String, _>("pdf_path"),
            "topic": r.get::<String, _>("topic"),
            "status": r.get::<String, _>("status"),
            "progress": r.get::<f64, _>("progress"),
            "message": r.get::<Option<String>, _>("message"),
            "output_dir": r.get::<Option<String>, _>("output_dir"),
            "markdown_path": r.get::<Option<String>, _>("markdown_path"),
            "created_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")
                .map(|dt| dt.to_rfc3339()),
        }))),
        None => Err(ApiError {
            error: "Job not found".to_string(),
            code: "NOT_FOUND".to_string(),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateIngestionConfigRequest {
    pub model_dir: Option<String>,
    pub output_base_dir: Option<String>,
    pub fallback_enabled: Option<bool>,
    pub default_ocr_langs: Option<Vec<String>>,
    pub max_parallel: Option<usize>,
    pub timeout_hours: Option<u64>,
}

pub async fn get_ingestion_config_handler(
    State(state): State<AppState>,
) -> Result<Json<alesys_core::ingestion::IngestionConfig>, ApiError> {
    Ok(Json(state.ingestion_config.clone()))
}

pub async fn put_ingestion_config_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<UpdateIngestionConfigRequest>,
) -> Result<Json<alesys_core::ingestion::IngestionConfig>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires admin permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let mut config = state.ingestion_config.clone();
    if let Some(v) = payload.model_dir {
        config.model_dir = PathBuf::from(v);
    }
    if let Some(v) = payload.output_base_dir {
        config.output_base_dir = PathBuf::from(v);
    }
    if let Some(v) = payload.fallback_enabled {
        config.fallback_enabled = v;
    }
    if let Some(v) = payload.default_ocr_langs {
        config.default_ocr_langs = v;
    }
    if let Some(v) = payload.max_parallel {
        config.max_parallel = v;
    }
    if let Some(v) = payload.timeout_hours {
        config.timeout_hours = v;
    }

    Ok(Json(config))
}

// WebSocket: /ws/ingestion/:job_id
#[derive(Debug, Deserialize)]
pub struct IngestMarkdownRequest {
    /// Root directory to scan for *.md files (books/, vault/, ...)
    pub dir: String,
    /// Explicit topic. If omitted, each top-level subdir becomes its own topic.
    #[serde(default)]
    pub topic: Option<String>,
    /// Safety limit
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    /// Preview only, do not index
    #[serde(default)]
    pub dry_run: bool,
}

fn default_max_files() -> usize { 5000 }

#[derive(Debug, Serialize)]
pub struct MarkdownPreviewFile {
    pub path: String,
    pub topic: String,
    pub title: String,
    pub bytes: usize,
}

/// POST /api/v1/ingestion/markdown
///
/// Async by design: returns a `job_id` immediately and does the indexing in a
/// background tokio task, so the request timeout never kills a long corpus
/// run. Poll `/api/v1/ingestion/markdown/status/{job_id}` for progress.
pub async fn ingest_markdown_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<IngestMarkdownRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let root = std::path::PathBuf::from(&payload.dir);
    let files = alesys_core::ingestion::discover_markdown(&root, payload.max_files)
        .map_err(|e| ApiError { error: e, code: "INVALID_DIR".to_string() })?;

    let total_bytes: usize = files.iter().map(|f| f.bytes).sum();
    let mut topics: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for f in &files { topics.insert(f.topic.clone()); }

    if payload.dry_run {
        return Ok(Json(serde_json::json!({
            "dry_run": true,
            "dir": payload.dir,
            "files_found": files.len(),
            "total_bytes": total_bytes,
            "topics": topics.into_iter().collect::<Vec<_>>(),
            "sample": files.iter().take(50).map(|f| MarkdownPreviewFile {
                path: f.path.display().to_string(),
                topic: f.topic.clone(),
                title: f.title.clone(),
                bytes: f.bytes,
            }).collect::<Vec<_>>(),
        })));
    }

    let job_id = Uuid::new_v4().to_string();
    let job = crate::state::MarkdownJobState {
        job_id: job_id.clone(),
        dir: payload.dir.clone(),
        topic: payload.topic.clone(),
        status: "queued".to_string(),
        files_done: 0,
        files_total: files.len(),
        chunks_indexed: 0,
        files_indexed: 0,
        skipped_empty: 0,
        topics: vec![],
        current_file: None,
        current_topic: None,
        message: Some("En cola".to_string()),
        error: None,
        result: None,
        created_at: Utc::now(),
        cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    state.markdown_jobs.write().await.insert(job_id.clone(), job.clone());

    let files_len = files.len();

    // Spawn the actual indexing — outlives this request.
    let user_id = claims.sub.clone();
    let jobs_map = state.markdown_jobs.clone();
    let graphrag = state.graphrag.clone();
    let db = state.db.clone();
    let semaphore = state.ingestion_semaphore.clone();
    let cancelled = job.cancelled.clone();
    let topic_override = payload.topic.clone();
    let dir_label = payload.dir.clone();
    let jid = job_id.clone();

    tokio::spawn(async move {
        // Per-user concurrency gate: serialize long embedding runs.
        let _permit = semaphore.acquire(&user_id).await;

        // A cancel that arrived while we were queued stops us before we start.
        if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
            let mut guard = jobs_map.write().await;
            if let Some(j) = guard.get_mut(&jid) {
                j.status = "cancelled".to_string();
                j.message = Some("Cancelado antes de iniciar".to_string());
            }
            return;
        }

        {
            let mut guard = jobs_map.write().await;
            if let Some(j) = guard.get_mut(&jid) {
                j.status = "processing".to_string();
                j.message = Some(format!("Indexando {} archivos", files.len()));
            }
        }

        let indexer = alesys_core::ingestion::MarkdownIndexer::new(db.clone());
        let progress_jobs = jobs_map.clone();
        let progress_jid = jid.clone();
        let progress_cancel = cancelled.clone();
        let result = indexer
            .index_files_with_progress(
                &files,
                &graphrag,
                topic_override.as_deref(),
                Some(&move |p: alesys_core::ingestion::MarkdownProgress| {
                    // Non-blocking-ish update: write lock is held only briefly.
                    if progress_cancel.load(std::sync::atomic::Ordering::SeqCst) {
                        return;
                    }
                    let guard = progress_jobs.clone();
                    let gid = progress_jid.clone();
                    tokio::spawn(async move {
                        let mut g = guard.write().await;
                        if let Some(j) = g.get_mut(&gid) {
                            j.files_done = p.files_done;
                            j.files_total = p.files_total;
                            j.chunks_indexed = p.chunks_indexed;
                            j.current_file = p.current_file.clone();
                            j.current_topic = p.current_topic.clone();
                        }
                    });
                }),
                Some(cancelled.as_ref()),
            )
            .await;

        let mut guard = jobs_map.write().await;
        if let Some(j) = guard.get_mut(&jid) {
            // Determine outcome: cancelled or errors with zero indexed = failure
            let failed = result.cancelled || (!result.errors.is_empty() && result.files_indexed == 0);
            if failed {
                j.status = if result.cancelled { "cancelled" } else { "failed" }.to_string();
                let msg = if result.cancelled {
                    "Cancelado por el usuario".to_string()
                } else {
                    result.errors.join("; ")
                };
                j.error = Some(msg.clone());
                j.message = Some(msg);
            } else {
                j.status = "completed".to_string();
                j.files_indexed = result.files_indexed;
                j.chunks_indexed = result.chunks_indexed;
                j.skipped_empty = result.skipped_empty;
                j.topics = result.topics.clone();
                j.files_done = j.files_total;
                j.current_file = None;
                j.current_topic = None;
                j.message = Some(format!(
                    "Completado: {} archivos, {} chunks",
                    result.files_indexed, result.chunks_indexed
                ));
                j.result = Some(serde_json::json!({
                    "dry_run": false,
                    "job_id": jid,
                    "dir": dir_label,
                    "files_indexed": result.files_indexed,
                    "chunks_indexed": result.chunks_indexed,
                    "skipped_empty": result.skipped_empty,
                    "topics": result.topics,
                    "warnings": result.errors, // non-fatal errors as warnings
                }));
            }
        }
        drop(guard);

        // Evict terminal jobs after 1 hour so the map doesn't grow forever.
        let jobs_map_ref = jobs_map.clone();
        let jid_ref = jid.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            let mut g = jobs_map_ref.write().await;
            if let Some(j) = g.get(&jid_ref) {
                if j.status != "processing" {
                    g.remove(&jid_ref);
                }
            }
        });
    });

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "status": "queued",
        "dir": payload.dir,
        "files_found": files_len,
        "files_total": files_len,
        "total_bytes": total_bytes,
        "topics": topics.into_iter().collect::<Vec<_>>(),
        "poll_url": format!("/api/v1/ingestion/markdown/status/{}", job_id),
    })))
}

/// GET /api/v1/ingestion/markdown/status/{job_id}
pub async fn ingest_markdown_status_handler(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guard = state.markdown_jobs.read().await;
    match guard.get(&job_id) {
        Some(j) => Ok(Json(serde_json::json!({
            "job_id": j.job_id,
            "dir": j.dir,
            "topic": j.topic,
            "status": j.status,
            "progress": j.progress_pct(),
            "files_done": j.files_done,
            "files_total": j.files_total,
            "files_indexed": j.files_indexed,
            "chunks_indexed": j.chunks_indexed,
            "skipped_empty": j.skipped_empty,
            "topics": j.topics,
            "current_file": j.current_file,
            "current_topic": j.current_topic,
            "message": j.message,
            "error": j.error,
            "created_at": j.created_at.to_rfc3339(),
        }))),
        None => Err(ApiError {
            error: "Job not found (may have expired)".to_string(),
            code: "NOT_FOUND".to_string(),
        }),
    }
}

/// GET /api/v1/ingestion/markdown/jobs — list live/recent Markdown jobs.
pub async fn list_markdown_jobs_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guard = state.markdown_jobs.read().await;
    let mut jobs: Vec<_> = guard.values().collect();
    jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let payload: Vec<serde_json::Value> = jobs
        .into_iter()
        .take(50)
        .map(|j| {
            serde_json::json!({
                "job_id": j.job_id,
                "dir": j.dir,
                "status": j.status,
                "progress": j.progress_pct(),
                "files_done": j.files_done,
                "files_total": j.files_total,
                "chunks_indexed": j.chunks_indexed,
                "message": j.message,
                "created_at": j.created_at.to_rfc3339(),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "jobs": payload })))
}

/// POST /api/v1/ingestion/markdown/{job_id}/cancel
pub async fn cancel_markdown_job_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let guard = state.markdown_jobs.read().await;
    match guard.get(&job_id) {
        Some(j) => {
            j.cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(Json(serde_json::json!({
                "job_id": job_id,
                "status": "cancelling",
                "message": "La tarea se detendrá tras el archivo en curso.",
            })))
        }
        None => Err(ApiError {
            error: "Job not found (may have expired)".to_string(),
            code: "NOT_FOUND".to_string(),
        }),
    }
}

pub async fn ws_ingestion_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (mut sender, _receiver) = socket.split();
        let job_uuid = match uuid::Uuid::parse_str(&job_id) {
            Ok(u) => u,
            Err(_) => {
                let _ = sender
                    .send(axum::extract::ws::Message::Text(
                        r#"{"type":"error","message":"Invalid job_id"}"#.into(),
                    ))
                    .await;
                return;
            }
        };

        loop {
            let row = sqlx::query(
                "SELECT status, progress, message, output_dir, markdown_path FROM ingestion_jobs WHERE id = $1"
            )
            .bind(job_uuid)
            .fetch_optional(&state.db)
            .await;

            let (status, progress, message, output_dir, markdown_path) = match row {
                Ok(Some(r)) => (
                    r.get::<String, _>("status"),
                    r.get::<f64, _>("progress"),
                    r.get::<Option<String>, _>("message"),
                    r.get::<Option<String>, _>("output_dir"),
                    r.get::<Option<String>, _>("markdown_path"),
                ),
                Ok(None) => {
                    let _ = sender
                        .send(axum::extract::ws::Message::Text(
                            r#"{"type":"error","message":"Job not found"}"#.into(),
                        ))
                        .await;
                    break;
                }
                Err(e) => {
                    let _ = sender
                        .send(axum::extract::ws::Message::Text(
                            format!(r#"{{"type":"error","message":"DB error: {}"}}"#, e).into(),
                        ))
                        .await;
                    break;
                }
            };

            let payload = serde_json::json!({
                "type": "progress",
                "job_id": job_uuid.to_string(),
                "status": status,
                "progress": progress,
                "message": message,
                "output_dir": output_dir,
                "markdown_path": markdown_path,
            });

            if sender
                .send(axum::extract::ws::Message::Text(payload.to_string().into()))
                .await
                .is_err()
            {
                break;
            }

            if status == "completed" || status == "failed" || status == "cancelled" {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    })
}
