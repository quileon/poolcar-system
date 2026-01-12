use axum::body::Bytes;
use deadpool_redis::redis::AsyncCommands;
use std::sync::Arc;

use crate::{
    models::{TrackerPayload, TrackerPayloadWithId},
    AppState,
};

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
    let tracker_payload_with_id = TrackerPayloadWithId {
        id: tracker_id,
        uptime: tracker_payload.uptime,
        connection: tracker_payload.connection,
        location: tracker_payload.location,
        altitude: tracker_payload.altitude,
        speed: tracker_payload.speed,
        course: tracker_payload.course,
        datetime: tracker_payload.datetime,
        satellites: tracker_payload.satellites,
        hdop: tracker_payload.hdop,
        stats: tracker_payload.stats,
    };

    // Get Redis connection
    let mut conn = state
        .redis
        .get()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;

    // Serialize payload to JSON string
    let payload_json = serde_json::to_string(&tracker_payload_with_id)
        .map_err(|e| anyhow::anyhow!("Failed to serialize payload: {}", e))?;

    // Save to Redis (live)
    conn.set::<String, String, ()>(format!("tracker:{}:live", tracker_id), payload_json.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save to Redis (live): {}", e))?;

    // Broadcast to websocket clients
    state
        .tx
        .send(payload_json)
        .map_err(|e| anyhow::anyhow!("Failed to broadcast payload: {}", e))?;

    Ok(())
}
