//! Research API Handlers - TICKET-31.7
//!
//! Endpoints (persistent PostgreSQL storage):
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
use sqlx::Row;
use uuid::Uuid;

// ============================================================================
// Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub owner_id: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
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

impl std::str::FromStr for ProjectStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(ProjectStatus::Draft),
            "active" => Ok(ProjectStatus::Active),
            "archived" => Ok(ProjectStatus::Archived),
            _ => Err(format!("Unknown status: {}", s)),
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
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
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

impl std::str::FromStr for NoteType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "summary" => Ok(NoteType::Summary),
            "critique" => Ok(NoteType::Critique),
            "question" => Ok(NoteType::Question),
            "idea" => Ok(NoteType::Idea),
            _ => Err(format!("Unknown note type: {}", s)),
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
    pub updated_by: Option<String>,
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

impl std::str::FromStr for CitationStyle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "apa" => Ok(CitationStyle::Apa),
            "mla" => Ok(CitationStyle::Mla),
            "chicago" => Ok(CitationStyle::Chicago),
            "ieee" => Ok(CitationStyle::Ieee),
            _ => Err(format!("Unknown citation style: {}", s)),
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
// Helpers
// ============================================================================

fn forbidden() -> ApiError {
    ApiError {
        error: "Forbidden: requires ingestion:write permission".to_string(),
        code: "FORBIDDEN".to_string(),
    }
}

fn not_found(msg: &str) -> ApiError {
    ApiError {
        error: msg.to_string(),
        code: "NOT_FOUND".to_string(),
    }
}

fn db_error(e: sqlx::Error) -> ApiError {
    ApiError {
        error: format!("Database error: {}", e),
        code: "DB_ERROR".to_string(),
    }
}

// ============================================================================
// Project Handlers
// ============================================================================

pub async fn list_projects_handler(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<ResearchProject>>, ApiError> {
    let owner_filter = if crate::auth::has_permission(&claims.role, Permission::IngestionWrite) { None } else { Some(claims.sub.clone()) };
    let rows = sqlx::query(
        "SELECT id, name, description, status, owner_id, created_by, updated_by, created_at, updated_at FROM research_projects WHERE ($1::text IS NULL OR owner_id = $1) ORDER BY created_at DESC",
    )
    .bind(owner_filter)
    .fetch_all(&state.db)
    .await
    .map_err(db_error)?;

    let mut projects = Vec::with_capacity(rows.len());
    for row in rows {
        let status_str: String = row.get("status");
        let status = status_str.parse::<ProjectStatus>().unwrap_or(ProjectStatus::Draft);
        projects.push(ResearchProject {
            id: row.get::<Uuid, _>("id").to_string(),
            name: row.get("name"),
            description: row.get("description"),
            status,
            owner_id: row.get("owner_id"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        });
    }
    Ok(Json(projects))
}

pub async fn create_project_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<ResearchProject>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }

    let id = Uuid::new_v4();
    let status = payload.status.unwrap_or(ProjectStatus::Draft);

    let row = sqlx::query(
        r#"INSERT INTO research_projects (id, name, description, status, owner_id, created_by, updated_by)
           VALUES ($1, $2, $3, $4, $5, $5, $5)
           RETURNING id, name, description, status, owner_id, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(status.to_string())
    .bind(&claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;

    let status_str: String = row.get("status");
    let status = status_str.parse::<ProjectStatus>().unwrap_or(ProjectStatus::Draft);
    let project = ResearchProject {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        description: row.get("description"),
        status,
        owner_id: row.get("owner_id"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    };

    // Auto-create empty synthesis document
    let synth_id = Uuid::new_v4();
    let _ = sqlx::query(
        r#"INSERT INTO research_synthesis (id, project_id, title)
           VALUES ($1, $2, $3)"#,
    )
    .bind(synth_id)
    .bind(id)
    .bind(format!("Synthesis - {}", payload.name))
    .bind(&claims.sub)
    .execute(&state.db)
    .await;

    Ok(Json(project))
}

pub async fn get_project_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ResearchProject>, ApiError> {
    let pid = Uuid::parse_str(&project_id).map_err(|_| not_found("Invalid project id"))?;

    let row = sqlx::query(
        "SELECT id, name, description, status, owner_id, created_by, updated_by, created_at, updated_at FROM research_projects WHERE id = $1",
    )
    .bind(pid)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?
    .ok_or_else(|| not_found("Project not found"))?;

    let status_str: String = row.get("status");
    let status = status_str.parse::<ProjectStatus>().unwrap_or(ProjectStatus::Draft);
    let project = ResearchProject {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        description: row.get("description"),
        status,
        owner_id: row.get("owner_id"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    };
    Ok(Json(project))
}

pub async fn update_project_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<ResearchProject>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }

    let pid = Uuid::parse_str(&project_id).map_err(|_| not_found("Invalid project id"))?;

    // Build dynamic update
    let mut sets = Vec::new();
    let mut bind_idx = 1i32;
    if payload.name.is_some() { sets.push(format!("name = ${}", bind_idx)); bind_idx += 1; }
    if payload.description.is_some() { sets.push(format!("description = ${}", bind_idx)); bind_idx += 1; }
    if payload.status.is_some() { sets.push(format!("status = ${}", bind_idx)); bind_idx += 1; }
    sets.push(format!("updated_by = ${}", bind_idx)); bind_idx += 1;
    sets.push("updated_at = NOW()".to_string());

    if sets.len() == 1 {
        // only updated_at, no-op but return existing
        return get_project_handler(claims, State(state), Path(project_id)).await;
    }

    let sql = format!(
        "UPDATE research_projects SET {} WHERE id = ${} RETURNING id, name, description, status, owner_id, created_by, updated_by, created_at, updated_at",
        sets.join(", "), bind_idx
    );

    let mut q = sqlx::query(&sql);
    if let Some(name) = &payload.name { q = q.bind(name); }
    if let Some(description) = &payload.description { q = q.bind(description); }
    if let Some(status) = &payload.status { q = q.bind(status.to_string()); }
    q = q.bind(&claims.sub);
    q = q.bind(pid);

    let row = q.fetch_optional(&state.db).await.map_err(db_error)?
        .ok_or_else(|| not_found("Project not found"))?;

    let status_str: String = row.get("status");
    let status = status_str.parse::<ProjectStatus>().unwrap_or(ProjectStatus::Draft);
    let project = ResearchProject {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        description: row.get("description"),
        status,
        owner_id: row.get("owner_id"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    };
    Ok(Json(project))
}

pub async fn delete_project_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }

    let pid = Uuid::parse_str(&project_id).map_err(|_| not_found("Invalid project id"))?;

    let result = sqlx::query("DELETE FROM research_projects WHERE id = $1")
        .bind(pid)
        .execute(&state.db)
        .await
        .map_err(db_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Project not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Notes Handlers
// ============================================================================

pub async fn list_notes_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<ListNotesQuery>,
) -> Result<Json<Vec<ResearchNote>>, ApiError> {
    let mut sql = String::from(
        "SELECT id, project_id, chapter_id, citation_id, title, content, note_type, tags, created_by, updated_by, created_at, updated_at FROM research_notes WHERE 1=1",
    );
    let mut bind_count = 0i32;

    if query.project_id.is_some() { bind_count += 1; sql.push_str(&format!(" AND project_id = ${}", bind_count)); }
    if query.chapter_id.is_some() { bind_count += 1; sql.push_str(&format!(" AND chapter_id = ${}", bind_count)); }
    if query.citation_id.is_some() { bind_count += 1; sql.push_str(&format!(" AND citation_id = ${}", bind_count)); }
    if query.note_type.is_some() { bind_count += 1; sql.push_str(&format!(" AND note_type = ${}", bind_count)); }
    sql.push_str(" ORDER BY created_at DESC");

    let mut q = sqlx::query(&sql);
    if let Some(project_id) = &query.project_id {
        let pid = Uuid::parse_str(project_id).unwrap_or_default();
        q = q.bind(pid);
    }
    if let Some(chapter_id) = &query.chapter_id {
        let cid = Uuid::parse_str(chapter_id).unwrap_or_default();
        q = q.bind(cid);
    }
    if let Some(citation_id) = &query.citation_id {
        q = q.bind(citation_id);
    }
    if let Some(note_type) = &query.note_type {
        q = q.bind(note_type.to_lowercase());
    }

    let rows = q.fetch_all(&state.db).await.map_err(db_error)?;

    let mut notes = Vec::with_capacity(rows.len());
    for row in rows {
        let note_type_str: String = row.get("note_type");
        let note_type = note_type_str.parse::<NoteType>().unwrap_or(NoteType::Idea);
        let tags: Vec<String> = row.get("tags");
        notes.push(ResearchNote {
            id: row.get::<Uuid, _>("id").to_string(),
            project_id: row.get::<Uuid, _>("project_id").to_string(),
            chapter_id: row.get::<Option<Uuid>, _>("chapter_id").map(|u| u.to_string()),
            citation_id: row.get("citation_id"),
            title: row.get("title"),
            content: row.get("content"),
            note_type,
            tags,
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        });
    }
    Ok(Json(notes))
}

pub async fn create_note_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<Json<ResearchNote>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }

    let id = Uuid::new_v4();
    let project_id = Uuid::parse_str(&payload.project_id).map_err(|_| not_found("Invalid project id"))?;
    let chapter_id = payload.chapter_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let note_type = payload.note_type.unwrap_or(NoteType::Idea);
    let tags = payload.tags.unwrap_or_default();

    let row = sqlx::query(
        r#"INSERT INTO research_notes (id, project_id, chapter_id, citation_id, title, content, note_type, tags, created_by, updated_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
           RETURNING id, project_id, chapter_id, citation_id, title, content, note_type, tags, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(chapter_id)
    .bind(&payload.citation_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(note_type.to_string())
    .bind(&tags)
    .bind(&claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;

    let note_type_str: String = row.get("note_type");
    let note_type = note_type_str.parse::<NoteType>().unwrap_or(NoteType::Idea);
    let tags: Vec<String> = row.get("tags");
    let note = ResearchNote {
        id: row.get::<Uuid, _>("id").to_string(),
        project_id: row.get::<Uuid, _>("project_id").to_string(),
        chapter_id: row.get::<Option<Uuid>, _>("chapter_id").map(|u| u.to_string()),
        citation_id: row.get("citation_id"),
        title: row.get("title"),
        content: row.get("content"),
        note_type,
        tags,
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    };
    Ok(Json(note))
}

pub async fn update_note_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(note_id): Path<String>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<ResearchNote>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }

    let nid = Uuid::parse_str(&note_id).map_err(|_| not_found("Invalid note id"))?;

    let mut sets = Vec::new();
    let mut bind_idx = 1i32;
    if payload.title.is_some() { sets.push(format!("title = ${}", bind_idx)); bind_idx += 1; }
    if payload.content.is_some() { sets.push(format!("content = ${}", bind_idx)); bind_idx += 1; }
    if payload.note_type.is_some() { sets.push(format!("note_type = ${}", bind_idx)); bind_idx += 1; }
    if payload.tags.is_some() { sets.push(format!("tags = ${}", bind_idx)); bind_idx += 1; }
    sets.push(format!("updated_by = ${}", bind_idx)); bind_idx += 1;
    sets.push("updated_at = NOW()".to_string());

    if sets.len() == 1 {
        // only updated_at
        let row = sqlx::query(
            "SELECT id, project_id, chapter_id, citation_id, title, content, note_type, tags, created_by, updated_by, created_at, updated_at FROM research_notes WHERE id = $1",
        )
        .bind(nid)
        .fetch_optional(&state.db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| not_found("Note not found"))?;
        let note_type_str: String = row.get("note_type");
        let note_type = note_type_str.parse::<NoteType>().unwrap_or(NoteType::Idea);
        let tags: Vec<String> = row.get("tags");
        return Ok(Json(ResearchNote {
            id: row.get::<Uuid, _>("id").to_string(),
            project_id: row.get::<Uuid, _>("project_id").to_string(),
            chapter_id: row.get::<Option<Uuid>, _>("chapter_id").map(|u| u.to_string()),
            citation_id: row.get("citation_id"),
            title: row.get("title"),
            content: row.get("content"),
            note_type,
            tags,
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        }));
    }

    let sql = format!(
        "UPDATE research_notes SET {} WHERE id = ${} RETURNING id, project_id, chapter_id, citation_id, title, content, note_type, tags, created_by, updated_by, created_at, updated_at",
        sets.join(", "), bind_idx
    );

    let mut q = sqlx::query(&sql);
    if let Some(title) = &payload.title { q = q.bind(title); }
    if let Some(content) = &payload.content { q = q.bind(content); }
    if let Some(note_type) = &payload.note_type { q = q.bind(note_type.to_string()); }
    if let Some(tags) = &payload.tags { q = q.bind(tags); }
    q = q.bind(&claims.sub);
    q = q.bind(nid);

    let row = q.fetch_optional(&state.db).await.map_err(db_error)?
        .ok_or_else(|| not_found("Note not found"))?;

    let note_type_str: String = row.get("note_type");
    let note_type = note_type_str.parse::<NoteType>().unwrap_or(NoteType::Idea);
    let tags: Vec<String> = row.get("tags");
    Ok(Json(ResearchNote {
        id: row.get::<Uuid, _>("id").to_string(),
        project_id: row.get::<Uuid, _>("project_id").to_string(),
        chapter_id: row.get::<Option<Uuid>, _>("chapter_id").map(|u| u.to_string()),
        citation_id: row.get("citation_id"),
        title: row.get("title"),
        content: row.get("content"),
        note_type,
        tags,
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn delete_note_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(note_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }

    let nid = Uuid::parse_str(&note_id).map_err(|_| not_found("Invalid note id"))?;

    let result = sqlx::query("DELETE FROM research_notes WHERE id = $1")
        .bind(nid)
        .execute(&state.db)
        .await
        .map_err(db_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Note not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Synthesis Handlers
// ============================================================================

pub async fn get_synthesis_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Option<SynthesisDocument>>, ApiError> {
    let pid = Uuid::parse_str(&project_id).unwrap_or_default();

    let row = sqlx::query(
        "SELECT id, project_id, title, content, citation_style, citations, updated_by, created_at, updated_at FROM research_synthesis WHERE project_id = $1",
    )
    .bind(pid)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?;

    let doc = match row {
        Some(row) => {
            let citations_json: serde_json::Value = row.get("citations");
            let citations: Vec<CitationRef> = serde_json::from_value(citations_json).unwrap_or_default();
            let style_str: String = row.get("citation_style");
            let style = style_str.parse::<CitationStyle>().unwrap_or(CitationStyle::Apa);
            Some(SynthesisDocument {
                id: row.get::<Uuid, _>("id").to_string(),
                project_id: row.get::<Uuid, _>("project_id").to_string(),
                title: row.get("title"),
                content: row.get("content"),
                citation_style: style,
                citations,
                updated_by: row.get("updated_by"),
                created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
            })
        }
        None => None,
    };
    Ok(Json(doc))
}

pub async fn update_synthesis_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<UpdateSynthesisRequest>,
) -> Result<Json<SynthesisDocument>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }

    let pid = Uuid::parse_str(&project_id).map_err(|_| not_found("Invalid project id"))?;

    // Ensure synthesis exists
    let exists = sqlx::query("SELECT id FROM research_synthesis WHERE project_id = $1")
        .bind(pid)
        .fetch_optional(&state.db)
        .await
        .map_err(db_error)?;
    if exists.is_none() {
        let synth_id = Uuid::new_v4();
        let _ = sqlx::query(
            "INSERT INTO research_synthesis (id, project_id, title, updated_by) VALUES ($1, $2, $3, $4)",
        )
        .bind(synth_id)
        .bind(pid)
        .bind("Synthesis")
        .execute(&state.db)
        .await;
    }

    let mut sets = Vec::new();
    let mut bind_idx = 1i32;
    if payload.title.is_some() { sets.push(format!("title = ${}", bind_idx)); bind_idx += 1; }
    if payload.content.is_some() { sets.push(format!("content = ${}", bind_idx)); bind_idx += 1; }
    if payload.citation_style.is_some() { sets.push(format!("citation_style = ${}", bind_idx)); bind_idx += 1; }
    sets.push(format!("updated_by = ${}", bind_idx)); bind_idx += 1;
    sets.push("updated_at = NOW()".to_string());

    let sql = format!(
        "UPDATE research_synthesis SET {} WHERE project_id = ${} RETURNING id, project_id, title, content, citation_style, citations, updated_by, created_at, updated_at",
        sets.join(", "), bind_idx
    );

    let mut q = sqlx::query(&sql);
    if let Some(title) = &payload.title { q = q.bind(title); }
    if let Some(content) = &payload.content { q = q.bind(content); }
    if let Some(citation_style) = &payload.citation_style { q = q.bind(citation_style.to_string()); }
    q = q.bind(&claims.sub);
    q = q.bind(pid);

    let row = q.fetch_one(&state.db).await.map_err(db_error)?;

    let citations_json: serde_json::Value = row.get("citations");
    let citations: Vec<CitationRef> = serde_json::from_value(citations_json).unwrap_or_default();
    let style_str: String = row.get("citation_style");
    let style = style_str.parse::<CitationStyle>().unwrap_or(CitationStyle::Apa);
    let doc = SynthesisDocument {
        id: row.get::<Uuid, _>("id").to_string(),
        project_id: row.get::<Uuid, _>("project_id").to_string(),
        title: row.get("title"),
        content: row.get("content"),
        citation_style: style,
        citations,
        updated_by: row.get("updated_by"),
                created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    };
    Ok(Json(doc))
}

pub async fn export_synthesis_markdown_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let pid = Uuid::parse_str(&project_id).unwrap_or_default();

    let row = sqlx::query(
        "SELECT title, content, citations, updated_by FROM research_synthesis WHERE project_id = $1",
    )
    .bind(pid)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?
    .ok_or_else(|| not_found("Synthesis not found"))?;

    let title: String = row.get("title");
    let content: String = row.get("content");
    let citations_json: serde_json::Value = row.get("citations");
    let citations: Vec<CitationRef> = serde_json::from_value(citations_json).unwrap_or_default();

    let markdown = format!(
        "# {}\n\n{}\n\n## Referencias\n\n{}",
        title,
        content,
        citations
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, format_citation_markdown(c)))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/markdown; charset=utf-8")
        .header("Content-Disposition", format!("attachment; filename=\"{}.md\"", title))
        .body(axum::body::Body::from(markdown))
        .unwrap())
}


pub async fn export_synthesis_latex_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let pid = Uuid::parse_str(&project_id).unwrap_or_default();
    let row = sqlx::query(
        "SELECT title, content, citations, citation_style FROM research_synthesis WHERE project_id = $1",
    )
    .bind(pid)
    .fetch_optional(&state.db)
    .await
    .map_err(db_error)?
    .ok_or_else(|| not_found("Synthesis not found"))?;

    let title: String = row.get("title");
    let content: String = row.get("content");
    let citations_json: serde_json::Value = row.get("citations");
    let citations: Vec<CitationRef> = serde_json::from_value(citations_json).unwrap_or_default();
    let style: String = row.get("citation_style");

    let bib = citations.iter().enumerate().map(|(i,c)| {
        format!("\\bibitem{{ref{}}} {} ({}) {}.", i+1, c.authors.join(", "), c.year.unwrap_or(0), c.title.clone().unwrap_or_default())
    }).collect::<Vec<_>>().join("
");

    let latex = format!(
        r"\documentclass{{article}}
\usepackage[utf8]{{inputenc}}
\title{{{}}}
\begin{{document}}
\maketitle

{}

\begin{{thebibliography}}{{99}}
{}
\end{{thebibliography}}
\end{{document}}",
        title.replace('\\', "\\\\"),
        content,
        bib
    );

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-latex; charset=utf-8")
        .header("Content-Disposition", format!("attachment; filename=\"{}.tex\"", title))
        .body(axum::body::Body::from(latex))
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
