use crate::{error::AppError, models::mqtt::MqttPayloadWithId, AppState};
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use deadpool_redis::redis::AsyncTypedCommands;
use std::sync::Arc;

pub async fn get_mqtt_payload_history(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let tracker_ids: Vec<i32> = sqlx::query_scalar(
        r#"
            SELECT tracker_id
            FROM trackers
            WHERE deleted_at IS NULL
            ORDER BY tracker_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut conn = state.redis.get().await?;
    let mut tracker_payloads: Vec<MqttPayloadWithId> = Vec::new();

    for tracker_id in tracker_ids {
        let tracker_payload = conn.get(format!("tracker:{}:live", tracker_id)).await?;

        if let Some(payload_json) = tracker_payload {
            let payload: MqttPayloadWithId = serde_json::from_str(&payload_json)?;
            tracker_payloads.push(payload);
        }
    }

    Ok(axum::Json(tracker_payloads))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_mqtt_payload_history))
}
