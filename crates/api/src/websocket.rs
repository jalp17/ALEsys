//! WebSocket handlers para streaming

use crate::state::AppState;
use alesys_core::llm::{ChatMessage, LLMEngine, StreamChunk};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WSMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub query: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct WSResponse {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub content: Option<String>,
    pub sources: Option<Vec<WSSource>>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct WSSource {
    pub fragment_id: i32,
    pub path: String,
    pub similarity: f32,
}

pub async fn ws_chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket_chat(socket, state))
}

async fn handle_websocket_chat(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                let ws_msg: WSMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        let response = WSResponse {
                            msg_type: "error".to_string(),
                            content: None,
                            sources: None,
                            error: Some(format!("Error al parsear mensaje: {}", e)),
                        };
                        let _ = sender
                            .send(axum::extract::ws::Message::Text(
                                serde_json::to_string(&response).unwrap().into(),
                            ))
                            .await;
                        continue;
                    }
                };

                if ws_msg.msg_type == "chat" {
                    if let Some(query) = ws_msg.query {
                        let start_response = WSResponse {
                            msg_type: "start".to_string(),
                            content: None,
                            sources: None,
                            error: None,
                        };
                        let _ = sender
                            .send(axum::extract::ws::Message::Text(
                                serde_json::to_string(&start_response).unwrap().into(),
                            ))
                            .await;

                        let query_embedding = match state.embedder.encode(&query) {
                            Ok(emb) => emb,
                            Err(e) => {
                                let response = WSResponse {
                                    msg_type: "error".to_string(),
                                    content: None,
                                    sources: None,
                                    error: Some(format!("Error al generar embedding: {}", e)),
                                };
                                let _ = sender
                                    .send(axum::extract::ws::Message::Text(
                                        serde_json::to_string(&response).unwrap().into(),
                                    ))
                                    .await;
                                continue;
                            }
                        };

                        let search_results =
                            match state.graphrag.hybrid_search(&query_embedding, 5, 1).await {
                                Ok(results) => results,
                                Err(e) => {
                                    let response = WSResponse {
                                        msg_type: "error".to_string(),
                                        content: None,
                                        sources: None,
                                        error: Some(format!("Error en búsqueda: {}", e)),
                                    };
                                    let _ = sender
                                        .send(axum::extract::ws::Message::Text(
                                            serde_json::to_string(&response).unwrap().into(),
                                        ))
                                        .await;
                                    continue;
                                }
                            };

                        let context =
                            alesys_core::graphrag::build_rag_context(&search_results, 2000);

                        let messages = vec![
                            ChatMessage {
                                role: "system".to_string(),
                                content: "Eres un asistente de IA experto en programación y análisis de documentos. Responde de forma clara y concisa basándote en el contexto proporcionado.".to_string(),
                            },
                            ChatMessage {
                                role: "user".to_string(),
                                content: format!("Contexto:\n{}\n\nPregunta: {}", context, query),
                            },
                        ];

                        match state.llm_engine.chat_stream(&messages) {
                            Ok(iterator) => {
                                for chunk_result in iterator {
                                    match chunk_result {
                                        Ok(chunk) => {
                                            let chunk_response = WSResponse {
                                                msg_type: "chunk".to_string(),
                                                content: Some(chunk.delta),
                                                sources: None,
                                                error: None,
                                            };
                                            if sender
                                                .send(axum::extract::ws::Message::Text(
                                                    serde_json::to_string(&chunk_response)
                                                        .unwrap()
                                                        .into(),
                                                ))
                                                .await
                                                .is_err()
                                            {
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            let response = WSResponse {
                                                msg_type: "error".to_string(),
                                                content: None,
                                                sources: None,
                                                error: Some(format!("Error en streaming: {}", e)),
                                            };
                                            let _ = sender
                                                .send(axum::extract::ws::Message::Text(
                                                    serde_json::to_string(&response)
                                                        .unwrap()
                                                        .into(),
                                                ))
                                                .await;
                                            break;
                                        }
                                    }
                                }

                                let sources: Vec<WSSource> = search_results
                                    .iter()
                                    .map(|r| WSSource {
                                        fragment_id: r.fragment_id,
                                        path: r
                                            .doc_path
                                            .clone()
                                            .unwrap_or_else(|| "desconocido".to_string()),
                                        similarity: r.similarity,
                                    })
                                    .collect();

                                let done_response = WSResponse {
                                    msg_type: "done".to_string(),
                                    content: None,
                                    sources: Some(sources),
                                    error: None,
                                };
                                let _ = sender
                                    .send(axum::extract::ws::Message::Text(
                                        serde_json::to_string(&done_response).unwrap().into(),
                                    ))
                                    .await;
                            }
                            Err(e) => {
                                let response = WSResponse {
                                    msg_type: "error".to_string(),
                                    content: None,
                                    sources: None,
                                    error: Some(format!("Error en LLM: {}", e)),
                                };
                                let _ = sender
                                    .send(axum::extract::ws::Message::Text(
                                        serde_json::to_string(&response).unwrap().into(),
                                    ))
                                    .await;
                            }
                        }
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                tracing::info!("WS closed");
                break;
            }
            Err(e) => {
                tracing::error!("WS error: {:?}", e);
                break;
            }
            _ => {}
        }
    }
}
