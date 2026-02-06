use crate::{models::mqtt::MqttPayloadWithId, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use deadpool_redis::redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct ChartPayload {
    tracker_id: i32,
    payload: Option<MqttPayloadWithId>,
}

pub async fn chart_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_chart_connection(socket, state))
}

async fn handle_chart_connection(mut socket: WebSocket, state: Arc<AppState>) {
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(_text))) => {
                // Client is requesting chart data
                match get_latest_chart_history(&state).await {
                    Ok(chart_data) => {
                        let json = match serde_json::to_string(&chart_data) {
                            Ok(json) => json,
                            Err(err) => {
                                eprintln!("Failed to serialize chart data: {}", err);
                                continue;
                            }
                        };

                        if socket.send(Message::Text(json.into())).await.is_err() {
                            eprintln!("Failed to send chart data to client");
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to get chart data: {}", err);
                        let error_msg = serde_json::json!({
                            "error": err
                        });
                        if let Ok(error_json) = serde_json::to_string(&error_msg) {
                            let _ = socket.send(Message::Text(error_json.into())).await;
                        }
                    }
                }
            }
            Some(Ok(Message::Close(_))) => {
                eprintln!("Websocket client on `/ws/chart` sent close frame");
                break;
            }
            Some(Ok(Message::Ping(data))) => {
                if socket.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Some(Ok(_)) => {
                continue;
            }
            Some(Err(err)) => {
                eprintln!(
                    "Error receiving message from websocket client on `/ws/chart`: {}",
                    err
                );
                break;
            }
            None => {
                eprintln!("Websocket client on `/ws/chart` disconnected");
                break;
            }
        }
    }
}

async fn get_latest_chart_history(state: &Arc<AppState>) -> Result<Vec<ChartPayload>, String> {
    // Fetch all tracker IDs from database
    let tracker_ids: Vec<i32> = sqlx::query_scalar(
        r#"
            SELECT tracker_id
            FROM trackers
            WHERE deleted_at IS NULL
            ORDER BY tracker_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to fetch tracker IDs: {}", e))?;

    // Get Redis connection
    let mut conn = state
        .redis
        .get()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let mut chart_payloads: Vec<ChartPayload> = Vec::new();

    for tracker_id in tracker_ids {
        // Get the latest entry from the history list
        let latest_chart_history = conn
            .lindex(format!("tracker:{}:history", tracker_id), -1)
            .await
            .map_err(|e| format!("Failed to get tracker history from Redis: {}", e))?;

        let payload = match latest_chart_history {
            Some(payload_json) => match serde_json::from_str::<MqttPayloadWithId>(&payload_json) {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    eprintln!(
                        "Failed to parse tracker payload for tracker {}: {}",
                        tracker_id, e
                    );
                    None
                }
            },
            None => None,
        };

        chart_payloads.push(ChartPayload {
            tracker_id,
            payload,
        });
    }

    Ok(chart_payloads)
}
