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

async fn handle_live_tracking_connection(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            // Receive MQTT payload from broadcast channel
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        // Send message to websocket client
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            eprintln!("Websocket client disconnected (send failed)");
                            break;
                        }
                    }
                    Err(RecvError::Lagged(skipped)) => {
                        eprintln!("Websocket receiver lagged, skipped {} message(s)", skipped);
                        continue;
                    }
                    Err(RecvError::Closed) => {
                        eprintln!("Broadcast channel closed unexpectedly");
                        break;
                    }
                }
            }
            // Receive from websocket client (detect disconnection)
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Close(_))) => {
                        eprintln!("Websocket client sent close frame");
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
                        eprintln!("Websocket client error");
                        break;
                    }
                    None => {
                        eprintln!("Websocket client disconnected");
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
