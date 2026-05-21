use crate::middleware::require_employee;
use crate::{error::AppError, models::mqtt::MqttPayloadWithId, AppState};
use axum::middleware::from_fn;
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use deadpool_redis::redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct ChartHistoryPayload {
    tracker_id: i32,
    payload: Vec<MqttPayloadWithId>,
}

pub async fn get_chart_history(
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

    let mut chart_payloads: Vec<ChartHistoryPayload> = Vec::new();

    for tracker_id in tracker_ids {
        let chart_history = conn
            .lrange(format!("tracker:{}:history", tracker_id), 0, -1)
            .await?;

        if !chart_history.is_empty() {
            let mut payload: Vec<MqttPayloadWithId> = Vec::new();

            for payload_json in chart_history {
                let parsed: MqttPayloadWithId = serde_json::from_str(&payload_json)?;
                payload.push(parsed);
            }

            chart_payloads.push(ChartHistoryPayload {
                tracker_id,
                payload,
            });
        } else {
            chart_payloads.push(ChartHistoryPayload {
                tracker_id,
                payload: Vec::new(),
            });
        }
    }

    Ok(axum::Json(chart_payloads))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_chart_history))
        .route_layer(from_fn(require_employee))
}
