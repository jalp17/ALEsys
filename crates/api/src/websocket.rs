//! WebSocket handlers para streaming

use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
};
use crate::state::AppState;

pub async fn ws_chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket_chat(socket, state))
}

async fn handle_websocket_chat(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Manejar mensajes del cliente
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    // Parsear mensaje: {"type": "chat", "query": "...", "session_id": "..."}
                    tracing::info!("WS message: {}", text);
                    
                    // TODO: Implementar streaming de respuesta del LLM
                    // Enviar chunks: {"type": "chunk", "content": "..."}
                    // Enviar done: {"type": "done"}
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
    });
}