//! Handlers de los endpoints HTTP

use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;

/// Request para chat
#[derive(Deserialize)]
pub struct ChatRequest {
    pub query: String,
    pub session_id: Option<String>,
}

/// Response de chat
#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub sources: Vec<Source>,
}

#[derive(Serialize)]
pub struct Source {
    pub fragment_id: i32,
    pub path: String,
    pub similarity: f32,
}

/// Handler para POST /api/chat
pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    // TODO: Implementar lógica de chat con GraphRAG
    // 1. Generar embedding del query
    // 2. Búsqueda híbrida (vector + grafo)
    // 3. Construir contexto RAG
    // 4. Llamar al LLM
    // 5. Retornar respuesta + fuentes
    
    tracing::info!("Chat request: {}", payload.query);
    
    // Placeholder para Fase 1
    let response = ChatResponse {
        response: "Respuesta placeholder - Implementar en Fase 1".to_string(),
        sources: vec![],
    };
    
    Json(response)
}

/// Request para generación
#[derive(Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub file_path: String,
    pub language: String,
}

/// Response de generación
#[derive(Serialize)]
pub struct GenerateResponse {
    pub generated_code: String,
    pub file_path: String,
}

/// Handler para POST /api/generate (FASE 2)
pub async fn generate_handler(
    State(state): State<AppState>,
    Json(payload): Json<GenerateRequest>,
) -> impl IntoResponse {
    tracing::info!("Generate request: {} → {}", payload.prompt, payload.file_path);
    
    // TODO: Implementar en Fase 2
    // 1. Prompt engineering para generación de código
    // 2. Llamar al LLM con contexto del proyecto
    // 3. Retornar código generado
    
    let response = GenerateResponse {
        generated_code: "// Código generado - Implementar en Fase 2".to_string(),
        file_path: payload.file_path,
    };
    
    Json(response)
}

/// Handler para GET /api/sessions
pub async fn list_sessions(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // TODO: Implementar list de sesiones
    // Placeholder
    Json(serde_json::json!({
        "sessions": []
    }))
}

/// Handler para POST /api/sessions
pub async fn create_session(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // TODO: Implementar creación de sesión
    Json(serde_json::json!({
        "session_id": "placeholder"
    }))
}

// FASE AVANZADA (Fase 7+):
// pub async fn execute_handler(...) { ... }
// pub async fn modify_handler(...) { ... }