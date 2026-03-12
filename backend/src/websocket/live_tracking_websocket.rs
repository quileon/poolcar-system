use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;

pub async fn live_tracking_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_live_tracking_connection(socket, state))
}

/// Handles WebSocket connection for frontend Live Tracking Page
///
/// Creates two tasks that runs simultenaously.
/// The former waits for MQTT payload from broadcast channel then send it to websocket client.
/// The latter listen for message from websocket client.
async fn handle_live_tracking_connection(mut socket: WebSocket, state: Arc<AppState>) {
    tracing::debug!("WebSocket client connected");
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            // Receive MQTT payload from broadcast channel
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            tracing::error!("Sending message failed, websocket client disconnected");
                            break;
                        }
                    }
                    Err(RecvError::Lagged(skipped)) => {
                        tracing::debug!("Websocket client lagged, skipped {} message(s)", skipped);
                        continue;
                    }
                    Err(RecvError::Closed) => {
                        tracing::error!("Broadcast channel closed unexpectedly");
                        break;
                    }
                }
            }
            // Receive from websocket client (detect disconnection)
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("Websocket client disconnected gracefully");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {
                        // Ignore other messages
                        continue;
                    }
                    Some(Err(_)) => {
                        tracing::debug!("Websocket client error, disconnecting");
                        break;
                    }
                    None => {
                        tracing::info!("Websocket client disconnected");
                        break;
                    }
                }
            }
        }
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(live_tracking_handler))
}
