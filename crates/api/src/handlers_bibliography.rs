//! Bibliography API Handlers - TICKET-30.4/30.3/30.5

use crate::auth::{Claims, Permission};
use crate::state::AppState;
use crate::handlers::ApiError;
use alesys_core::bibliography::{
    Citation, CitationFormatter, CitationStorage, CitationStyle, FormatError, CitationDeduplicator,
};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct StoreCitationRequest {
    pub citation: Citation,
}

pub async fn store_citation_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<StoreCitationRequest>,
) -> Result<Json<Citation>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let storage = CitationStorage::new(state.db.clone());
    storage.store(&payload.citation).await.map_err(|e| ApiError {
        error: format!("Failed to store citation: {}", e),
        code: "STORAGE_ERROR".to_string(),
    })?;

    Ok(Json(payload.citation))
}

pub async fn list_citations_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(chapter_id): Path<String>,
) -> Result<Json<Vec<Citation>>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionRead) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:read permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let chapter_uuid = Uuid::parse_str(&chapter_id).map_err(|_| ApiError {
        error: "Invalid chapter_id format".to_string(),
        code: "BAD_REQUEST".to_string(),
    })?;

    let storage = CitationStorage::new(state.db.clone());
    let citations = storage.list_by_chapter(chapter_uuid, 100, 0).await.map_err(|e| ApiError {
        error: format!("Failed to list citations: {}", e),
        code: "STORAGE_ERROR".to_string(),
    })?;

    Ok(Json(citations))
}

#[derive(Debug, Deserialize)]
pub struct FormatCitationRequest {
    pub style: CitationStyle,
}

pub async fn format_citation_handler(
    claims: Claims,
    Path(citation_id): Path<String>,
    Json(payload): Json<FormatCitationRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionRead) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:read permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let _citation_uuid = Uuid::parse_str(&citation_id).map_err(|_| ApiError {
        error: "Invalid citation_id format".to_string(),
        code: "BAD_REQUEST".to_string(),
    })?;

    let formatted = CitationFormatter::format(&Citation::new(String::new(), 1), payload.style.clone())
        .map_err(|e: FormatError| ApiError {
            error: format!("Formatting error: {}", e),
            code: "FORMAT_ERROR".to_string(),
        })?;

    Ok(Json(serde_json::json!({
        "citation_id": citation_id,
        "style": format!("{:?}", payload.style),
        "formatted": formatted,
    })))
}

pub async fn deduplicate_citations_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Vec<Citation>>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let citations: Vec<Citation> = serde_json::from_value(
        payload.get("citations").cloned().unwrap_or(serde_json::Value::Array(vec![]))
    ).map_err(|_| ApiError {
        error: "Invalid citations array".to_string(),
        code: "BAD_REQUEST".to_string(),
    })?;

    let threshold = payload.get("threshold")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.8);

    let deduplicator = CitationDeduplicator::with_threshold(threshold);
    let deduplicated = deduplicator.deduplicate(citations);

    Ok(Json(deduplicated))
}
