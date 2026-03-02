use axum::body::Bytes;
use deadpool_redis::redis::AsyncCommands;
use std::sync::Arc;

use crate::{models::mqtt::MqttPayloadWithId, AppState};

pub async fn mqtt_handler(state: Arc<AppState>, payload: Bytes) -> anyhow::Result<()> {
    // Parse payload to JSON
    let tracker_payload_with_id: MqttPayloadWithId = match serde_json::from_slice(&payload) {
        Ok(p) => p,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to parse MQTT payload to JSON: {}",
                e
            ))
        }
    };

    // Get Redis connection
    let mut conn = state
        .redis
        .get()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;

    let tracker_payload_string = match serde_json::to_string(&tracker_payload_with_id) {
        Ok(s) => s,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to serialize MQTT payload to string: {}",
                e
            ))
        }
    };

    // Save to Redis (live)
    conn.set::<_, _, ()>(
        format!("tracker:{}:live", tracker_payload_with_id.id),
        &tracker_payload_string,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to save to Redis (live): {}", e))?;

    // Broadcast to websocket clients
    state
        .tx
        .send(tracker_payload_string)
        .map_err(|e| anyhow::anyhow!("Failed to broadcast payload: {}", e))?;

    Ok(())
}
