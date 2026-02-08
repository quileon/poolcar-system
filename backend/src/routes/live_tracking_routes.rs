use crate::{models::mqtt::MqttPayloadWithId, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use deadpool_redis::redis::AsyncTypedCommands;
use std::sync::Arc;

pub async fn get_live_tracking_history(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
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
    .map_err(|e| {
        eprintln!("Failed to fetch tracker IDs: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch tracker IDs: {}", e),
        )
    })?;

    let mut conn = state.redis.get().await.map_err(|e| {
        eprintln!("Failed to get Redis connection: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get Redis connection: {}", e),
        )
    })?;

    let mut tracker_payloads: Vec<MqttPayloadWithId> = Vec::new();

    for tracker_id in tracker_ids {
        let tracker_payload = conn
            .get(format!("tracker:{}:live", tracker_id))
            .await
            .map_err(|e| {
                eprintln!("Failed to get tracker payload from Redis: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get tracker payload from Redis: {}", e),
                )
            })?;

        if let Some(payload_json) = tracker_payload {
            let payload: MqttPayloadWithId = serde_json::from_str(&payload_json).map_err(|e| {
                eprintln!("Failed to parse tracker payload: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse tracker payload: {}", e),
                )
            })?;
            tracker_payloads.push(payload);
        }
    }

    Ok(axum::Json(tracker_payloads))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_live_tracking_history))
}
