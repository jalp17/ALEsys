//! Research API Handlers - TICKET-31.7
//!
//! Endpoints:
//! - GET    /api/v1/research/projects           -> List projects
//! - POST   /api/v1/research/projects           -> Create project
//! - GET    /api/v1/research/projects/:id       -> Get project
//! - PUT    /api/v1/research/projects/:id       -> Update project
//! - DELETE /api/v1/research/projects/:id       -> Delete project
//! - GET    /api/v1/research/notes              -> List notes (filtered by project/chapter/citation)
//! - POST   /api/v1/research/notes              -> Create note
//! - PUT    /api/v1/research/notes/:id          -> Update note
//! - DELETE /api/v1/research/notes/:id          -> Delete note
//! - GET    /api/v1/research/synthesis/:project_id -> Get synthesis
//! - PUT    /api/v1/research/synthesis/:project_id -> Update synthesis
//! - GET    /api/v1/research/export/:project_id/markdown -> Export synthesis as markdown

use crate::auth::{Claims, Permission};
use crate::state::AppState;
use crate::handlers::ApiError;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use once_cell::sync::Lazy;

// ============================================================================
// Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Draft,
    Active,
    Archived,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectStatus::Draft => write!(f, "draft"),
            ProjectStatus::Active => write!(f, "active"),
            ProjectStatus::Archived => write!(f, "archived"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchNote {
    pub id: String,
    pub project_id: String,
    pub chapter_id: Option<String>,
    pub citation_id: Option<String>,
    pub title: String,
    pub content: String,
    pub note_type: NoteType,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteType {
    Summary,
    Critique,
    Question,
    Idea,
}

impl std::fmt::Display for NoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoteType::Summary => write!(f, "summary"),
            NoteType::Critique => write!(f, "critique"),
            NoteType::Question => write!(f, "question"),
            NoteType::Idea => write!(f, "idea"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisDocument {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub content: String,
    pub citation_style: CitationStyle,
    pub citations: Vec<CitationRef>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRef {
    pub id: String,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<u32>,
    pub style: CitationStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CitationStyle {
    Apa,
    Mla,
    Chicago,
    Ieee,
}

impl std::fmt::Display for CitationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CitationStyle::Apa => write!(f, "apa"),
            CitationStyle::Mla => write!(f, "mla"),
            CitationStyle::Chicago => write!(f, "chicago"),
            CitationStyle::Ieee => write!(f, "ieee"),
        }
    }
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
    pub status: Option<ProjectStatus>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub project_id: String,
    pub chapter_id: Option<String>,
    pub citation_id: Option<String>,
    pub title: String,
    pub content: String,
    pub note_type: Option<NoteType>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub note_type: Option<NoteType>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSynthesisRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub citation_style: Option<CitationStyle>,
}

#[derive(Debug, Deserialize)]
pub struct ListNotesQuery {
    pub project_id: Option<String>,
    pub chapter_id: Option<String>,
    pub citation_id: Option<String>,
    pub note_type: Option<String>,
}

// ============================================================================
// In-memory storage (replace with DB in production)
// ============================================================================

static RESEARCH_PROJECTS: Lazy<RwLock<Vec<ResearchProject>>> = Lazy::new(|| RwLock::new(vec![]));
static RESEARCH_NOTES: Lazy<RwLock<Vec<ResearchNote>>> = Lazy::new(|| RwLock::new(vec![]));
static RESEARCH_SYNTHESIS: Lazy<RwLock<Vec<SynthesisDocument>>> = Lazy::new(|| RwLock::new(vec![]));

// ============================================================================
// Project Handlers
// ============================================================================

pub async fn list_projects_handler(
    _claims: Claims,
) -> Result<Json<Vec<ResearchProject>>, ApiError> {
    let projects = RESEARCH_PROJECTS.read().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;
    Ok(Json(projects.clone()))
}

pub async fn create_project_handler(
    _claims: Claims,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<ResearchProject>, ApiError> {
    if !crate::auth::has_permission(&_claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let now = chrono::Utc::now().to_rfc3339();
    let project = ResearchProject {
        id: Uuid::new_v4().to_string(),
        name: payload.name,
        description: payload.description,
        status: payload.status.unwrap_or(ProjectStatus::Draft),
        created_at: now.clone(),
        updated_at: now,
    };

    RESEARCH_PROJECTS.write().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?.push(project.clone());

    Ok(Json(project))
}

pub async fn get_project_handler(
    _claims: Claims,
    Path(project_id): Path<String>,
) -> Result<Json<ResearchProject>, ApiError> {
    let projects = RESEARCH_PROJECTS.read().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    projects
        .iter()
        .find(|p| p.id == project_id)
        .cloned()
        .ok_or_else(|| ApiError {
            error: "Project not found".to_string(),
            code: "NOT_FOUND".to_string(),
        })
        .map(Json)
}

pub async fn update_project_handler(
    _claims: Claims,
    Path(project_id): Path<String>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<ResearchProject>, ApiError> {
    if !crate::auth::has_permission(&_claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let mut projects = RESEARCH_PROJECTS.write().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let project = projects
        .iter_mut()
        .find(|p| p.id == project_id)
        .ok_or_else(|| ApiError {
            error: "Project not found".to_string(),
            code: "NOT_FOUND".to_string(),
        })?;

    if let Some(name) = payload.name {
        project.name = name;
    }
    if let Some(description) = payload.description {
        project.description = description;
    }
    if let Some(status) = payload.status {
        project.status = status;
    }
    project.updated_at = chrono::Utc::now().to_rfc3339();

    Ok(Json(project.clone()))
}

pub async fn delete_project_handler(
    _claims: Claims,
    Path(project_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&_claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let mut projects = RESEARCH_PROJECTS.write().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let index = projects
        .iter()
        .position(|p| p.id == project_id)
        .ok_or_else(|| ApiError {
            error: "Project not found".to_string(),
            code: "NOT_FOUND".to_string(),
        })?;

    projects.remove(index);
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Notes Handlers
// ============================================================================

pub async fn list_notes_handler(
    _claims: Claims,
    Query(query): Query<ListNotesQuery>,
) -> Result<Json<Vec<ResearchNote>>, ApiError> {
    let notes = RESEARCH_NOTES.read().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let filtered = notes
        .iter()
        .filter(|note| {
            if let Some(project_id) = &query.project_id {
                if note.project_id != *project_id {
                    return false;
                }
            }
            if let Some(chapter_id) = &query.chapter_id {
                if note.chapter_id.as_ref() != Some(chapter_id) {
                    return false;
                }
            }
            if let Some(citation_id) = &query.citation_id {
                if note.citation_id.as_ref() != Some(citation_id) {
                    return false;
                }
            }
            if let Some(note_type) = &query.note_type {
                if format!("{}", note.note_type).to_lowercase() != note_type.to_lowercase() {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    Ok(Json(filtered))
}

pub async fn create_note_handler(
    _claims: Claims,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<Json<ResearchNote>, ApiError> {
    if !crate::auth::has_permission(&_claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let now = chrono::Utc::now().to_rfc3339();
    let note = ResearchNote {
        id: Uuid::new_v4().to_string(),
        project_id: payload.project_id,
        chapter_id: payload.chapter_id,
        citation_id: payload.citation_id,
        title: payload.title,
        content: payload.content,
        note_type: payload.note_type.unwrap_or(NoteType::Idea),
        tags: payload.tags.unwrap_or_default(),
        created_at: now.clone(),
        updated_at: now,
    };

    RESEARCH_NOTES.write().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?.push(note.clone());

    Ok(Json(note))
}

pub async fn update_note_handler(
    _claims: Claims,
    Path(note_id): Path<String>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<ResearchNote>, ApiError> {
    if !crate::auth::has_permission(&_claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let mut notes = RESEARCH_NOTES.write().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let note = notes
        .iter_mut()
        .find(|n| n.id == note_id)
        .ok_or_else(|| ApiError {
            error: "Note not found".to_string(),
            code: "NOT_FOUND".to_string(),
        })?;

    if let Some(title) = payload.title {
        note.title = title;
    }
    if let Some(content) = payload.content {
        note.content = content;
    }
    if let Some(note_type) = payload.note_type {
        note.note_type = note_type;
    }
    if let Some(tags) = payload.tags {
        note.tags = tags;
    }
    note.updated_at = chrono::Utc::now().to_rfc3339();

    Ok(Json(note.clone()))
}

pub async fn delete_note_handler(
    _claims: Claims,
    Path(note_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&_claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let mut notes = RESEARCH_NOTES.write().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let index = notes
        .iter()
        .position(|n| n.id == note_id)
        .ok_or_else(|| ApiError {
            error: "Note not found".to_string(),
            code: "NOT_FOUND".to_string(),
        })?;

    notes.remove(index);
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Synthesis Handlers
// ============================================================================

pub async fn get_synthesis_handler(
    _claims: Claims,
    Path(project_id): Path<String>,
) -> Result<Json<Option<SynthesisDocument>>, ApiError> {
    let synthesis = RESEARCH_SYNTHESIS.read().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let doc = synthesis
        .iter()
        .find(|s| s.project_id == project_id)
        .cloned();

    Ok(Json(doc))
}

pub async fn update_synthesis_handler(
    _claims: Claims,
    Path(project_id): Path<String>,
    Json(payload): Json<UpdateSynthesisRequest>,
) -> Result<Json<SynthesisDocument>, ApiError> {
    if !crate::auth::has_permission(&_claims.role, Permission::IngestionWrite) {
        return Err(ApiError {
            error: "Forbidden: requires ingestion:write permission".to_string(),
            code: "FORBIDDEN".to_string(),
        });
    }

    let mut synthesis = RESEARCH_SYNTHESIS.write().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let doc = synthesis
        .iter_mut()
        .find(|s| s.project_id == project_id)
        .ok_or_else(|| ApiError {
            error: "Synthesis not found".to_string(),
            code: "NOT_FOUND".to_string(),
        })?;

    if let Some(title) = payload.title {
        doc.title = title;
    }
    if let Some(content) = payload.content {
        doc.content = content;
    }
    if let Some(citation_style) = payload.citation_style {
        doc.citation_style = citation_style;
    }
    doc.updated_at = chrono::Utc::now().to_rfc3339();

    Ok(Json(doc.clone()))
}

pub async fn export_synthesis_markdown_handler(
    _claims: Claims,
    Path(project_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let synthesis = RESEARCH_SYNTHESIS.read().map_err(|_| ApiError {
        error: "Failed to acquire lock".to_string(),
        code: "INTERNAL_ERROR".to_string(),
    })?;

    let doc = synthesis
        .iter()
        .find(|s| s.project_id == project_id)
        .ok_or_else(|| ApiError {
            error: "Synthesis not found".to_string(),
            code: "NOT_FOUND".to_string(),
        })?;

    let markdown = format!(
        "# {}\n\n{}\n\n## Referencias\n\n{}",
        doc.title,
        doc.content,
        doc.citations
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, format_citation_markdown(c)))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/markdown; charset=utf-8")
        .header("Content-Disposition", format!("attachment; filename=\"{}.md\"", doc.title))
        .body(axum::body::Body::from(markdown))
        .unwrap())
}

fn format_citation_markdown(citation: &CitationRef) -> String {
    let authors = citation.authors.join(", ");
    let year = citation.year.map(|y| y.to_string()).unwrap_or_default();
    let title = citation.title.clone().unwrap_or_default();

    match citation.style {
        CitationStyle::Apa => format!("{} ({}) {}.", authors, year, title),
        CitationStyle::Mla => format!("{}. \"{}\". {}.", authors, title, year),
        CitationStyle::Chicago => format!("{}. \"{}\". {}.", authors, title, year),
        CitationStyle::Ieee => format!("{}, \"{}\", {}.", authors, title, year),
    }
}
