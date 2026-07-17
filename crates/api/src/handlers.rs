//! Handlers de los endpoints HTTP

use crate::state::AppState;
use alesys_core::graphrag::SearchResultSource;
use alesys_core::llm::{ChatMessage, LLMEngine};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

/// Request para chat
#[derive(Deserialize)]
pub struct ChatRequest {
    pub query: String,
    pub _session_id: Option<String>,
    pub _stream: Option<bool>,
}

/// Response de chat
#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub sources: Vec<Source>,
    pub query: String,
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
pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("Chat request: {}", payload.query);

    // 1. Generar embedding del query
    let query_embedding = state.embedder.encode(&payload.query).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error al generar embedding: {}", e),
        )
    })?;

    // 2. Búsqueda híbrida (vector + grafo)
    let search_results = state
        .graphrag
        .hybrid_search(&query_embedding, 5, 1)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error en búsqueda: {}", e),
            )
        })?;

    // 3. Construir contexto RAG
    let context = alesys_core::graphrag::build_rag_context(&search_results, 2000);

    // 4. Llamar al LLM
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "Eres un asistente de IA experto en programación y análisis de documentos. Responde de forma clara y concisa basándote en el contexto proporcionado.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!("Contexto:\n{}\n\nPregunta: {}", context, payload.query),
        },
    ];

    let llm_response = state.llm_engine.chat(&messages).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error en LLM: {}", e),
        )
    })?;

    // 5. Convertir resultados a formato de respuesta
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

/// Response de generación
#[derive(Serialize)]
pub struct GenerateResponse {
    pub file_name: String,
    pub content: String,
    pub language: String,
    pub explanation: String,
    pub suggestions: Vec<String>,
}

/// Handler para POST /api/generate
///
/// Reutiliza el LLMBackend compartido de AppState (no crea instancias nuevas).
/// Incluye validación de sintaxis post-generación via SyntaxValidator.
pub async fn generate_handler(
    State(state): State<AppState>,
    Json(payload): Json<GenerateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(
        "Generate request: '{}' → {}",
        payload.prompt,
        payload.language
    );

    let context = payload.context.map(|ctx| {
        alesys_core::generator::BuildContext {
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
        }
    });

    let gen_request = alesys_core::generator::GenerateRequest {
        prompt: payload.prompt,
        language: payload.language,
        context,
        max_tokens: payload.max_tokens.unwrap_or(2048),
    };

    let generator = alesys_core::generator::CodeGenerator::new(state.llm_engine.clone());

    let result = generator.generate(gen_request).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error al generar código: {}", e),
        )
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

/// Handler para GET /api/sessions
pub async fn list_sessions(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Implementar list de sesiones
    Json(serde_json::json!({
        "sessions": []
    }))
}

/// Handler para POST /api/sessions
pub async fn create_session(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Implementar creación de sesión
    Json(serde_json::json!({
        "session_id": "placeholder"
    }))
}

/// Handler para GET /api/graph/stats
pub async fn graph_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.graphrag.graph_stats();
    Json(serde_json::json!({
        "nodes": stats.nodes,
        "edges": stats.edges,
    }))
}

/// Health check endpoint
pub async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
