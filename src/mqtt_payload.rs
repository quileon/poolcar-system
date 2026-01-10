use axum::body::Bytes;
use deadpool_redis::redis::AsyncCommands;
use std::sync::Arc;

use crate::{models::TrackerPayload, AppState};

pub async fn save_latest_payload(
    state: Arc<AppState>,
    topic: &str,
    payload: Bytes,
) -> anyhow::Result<()> {
    // Extract tracker_id from topic
    let tracker_id = topic[8..]
        .parse::<u8>()
        .map_err(|e| anyhow::anyhow!("Failed to parse tracker_id: {}", e))?;

    // Parse payload to JSON
    let tracker_payload = serde_json::from_slice::<TrackerPayload>(&payload)
        .map_err(|e| anyhow::anyhow!("Failed to parse payload: {}", e))?;

    // Get Redis connection
    let mut conn = state
        .redis
        .get()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;

    // Serialize payload to JSON string
    let payload_json = serde_json::to_string(&tracker_payload)
        .map_err(|e| anyhow::anyhow!("Failed to serialize payload: {}", e))?;

    // Save to Redis (Latest)
    conn.set::<u8, String, ()>(tracker_id, payload_json)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save to Redis: {}", e))?;

    Ok(())
}
