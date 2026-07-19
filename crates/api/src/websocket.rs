//! WebSocket handlers para streaming

use crate::state::AppState;
use alesys_core::agent::protocol::AgentResponse;
use alesys_core::llm::ChatMessage;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct WSMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub query: Option<String>,
    pub _session_id: Option<String>,
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

fn ws_error(msg: &str) -> WSResponse {
    WSResponse {
        msg_type: "error".to_string(),
        content: None,
        sources: None,
        error: Some(msg.to_string()),
    }
}

async fn send_ws_response(
    sender: &mut futures::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
    response: &WSResponse,
) -> bool {
    match serde_json::to_string(response) {
        Ok(json) => {
            if sender
                .send(axum::extract::ws::Message::Text(json.into()))
                .await
                .is_err()
            {
                tracing::warn!("WS send failed — client disconnected");
                false
            } else {
                true
            }
        }
        Err(e) => {
            tracing::error!("WS serialize error: {}", e);
            false
        }
    }
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
                        tracing::debug!("WS parse error: {}", e);
                        if !send_ws_response(&mut sender, &ws_error("Formato de mensaje invalido"))
                            .await
                        {
                            break;
                        }
                        continue;
                    }
                };

                if ws_msg.msg_type == "chat" {
                    if let Some(query) = ws_msg.query {
                        if !send_ws_response(
                            &mut sender,
                            &WSResponse {
                                msg_type: "start".to_string(),
                                content: None,
                                sources: None,
                                error: None,
                            },
                        )
                        .await
                        {
                            break;
                        }

                        let query_embedding = match state.embedder.encode(&query) {
                            Ok(emb) => emb,
                            Err(e) => {
                                tracing::error!("WS embedding error: {}", e);
                                if !send_ws_response(
                                    &mut sender,
                                    &ws_error("Error generando embedding"),
                                )
                                .await
                                {
                                    break;
                                }
                                continue;
                            }
                        };

                        let search_results = match state
                            .graphrag
                            .hybrid_search(&query_embedding, 5, 1)
                            .await
                        {
                            Ok(results) => results,
                            Err(e) => {
                                tracing::error!("WS search error: {}", e);
                                if !send_ws_response(&mut sender, &ws_error("Error en busqueda"))
                                    .await
                                {
                                    break;
                                }
                                continue;
                            }
                        };

                        let context =
                            alesys_core::graphrag::build_rag_context(&search_results, 2000);

                        let messages = vec![
                            ChatMessage {
                                role: "system".to_string(),
                                content: crate::CHAT_SYSTEM_PROMPT.to_string(),
                            },
                            ChatMessage {
                                role: "user".to_string(),
                                content: format!("Contexto:\n{}\n\nPregunta: {}", context, query),
                            },
                        ];

                        let mut stream = state.llm_queue.chat_stream(&messages);
                        loop {
                            match stream.next().await {
                                Some(Ok(chunk)) => {
                                    if !send_ws_response(
                                        &mut sender,
                                        &WSResponse {
                                            msg_type: "chunk".to_string(),
                                            content: Some(chunk.delta),
                                            sources: None,
                                            error: None,
                                        },
                                    )
                                    .await
                                    {
                                        break;
                                    }

                                    if chunk.finish_reason.is_some() {
                                        break;
                                    }
                                }
                                Some(Err(e)) => {
                                    tracing::error!("WS LLM error: {}", e);
                                    send_ws_response(
                                        &mut sender,
                                        &ws_error("Error generando respuesta"),
                                    )
                                    .await;
                                    break;
                                }
                                None => break,
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

                        if !send_ws_response(
                            &mut sender,
                            &WSResponse {
                                msg_type: "done".to_string(),
                                content: None,
                                sources: Some(sources),
                                error: None,
                            },
                        )
                        .await
                        {
                            break;
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

// =============================================================================
// Phase 9: Agent WebSocket Handler
// =============================================================================

#[derive(Deserialize)]
pub struct AgentRegisterMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: Option<AgentRegisterPayload>,
}

#[derive(Deserialize)]
pub struct AgentRegisterPayload {
    pub name: String,
    #[allow(dead_code)]
    pub token: String,
}

pub async fn ws_agent_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket_agent(socket, state))
}

async fn handle_websocket_agent(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let agent_id = Uuid::new_v4().to_string();

    tracing::info!("Agent WebSocket connected: {}", agent_id);

    let (tx, _rx) = tokio::sync::mpsc::channel::<Vec<u8>>(32);

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                let ws_msg: AgentRegisterMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(_) => {
                        if let Ok(response) = serde_json::from_str::<AgentResponse>(&text) {
                            state.agent_manager.handle_response(response).await;
                        }
                        continue;
                    }
                };

                if ws_msg.msg_type == "register" {
                    if let Some(payload) = ws_msg.payload {
                        let info = alesys_core::agent::AgentInfo {
                            id: agent_id.clone(),
                            name: payload.name,
                            os: "unknown".to_string(),
                            arch: "unknown".to_string(),
                            status: alesys_core::agent::AgentStatus::Connected,
                            connected_at: chrono::Utc::now(),
                        };

                        state.agent_manager.register_agent(info, tx.clone()).await;
                        tracing::info!("Agent registered: {}", agent_id);

                        let pong = serde_json::json!({"type": "pong"});
                        let _ = sender.send(axum::extract::ws::Message::Text(pong.to_string().into())).await;
                    }
                }
            }
            Ok(axum::extract::ws::Message::Binary(data)) => {
                if let Ok(response) = serde_json::from_slice::<AgentResponse>(&data) {
                    state.agent_manager.handle_response(response).await;
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                tracing::info!("Agent WS closed: {}", agent_id);
                state.agent_manager.unregister_agent(&agent_id).await;
                break;
            }
            Err(e) => {
                tracing::error!("Agent WS error: {:?}", e);
                state.agent_manager.unregister_agent(&agent_id).await;
                break;
            }
            _ => {}
        }
    }

    state.agent_manager.unregister_agent(&agent_id).await;
    tracing::info!("Agent disconnected: {}", agent_id);
}
