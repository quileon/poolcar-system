use crate::{models::TrackerPayloadWithId, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use deadpool_redis::redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct ChartHistoryPayload {
    tracker_id: i32,
    payload: Vec<TrackerPayloadWithId>,
}

pub async fn get_chart_history(
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

    let mut chart_payloads: Vec<ChartHistoryPayload> = Vec::new();

    for tracker_id in tracker_ids {
        let chart_history = conn
            .lrange(format!("tracker:{}:history", tracker_id), 0, -1)
            .await
            .map_err(|e| {
                eprintln!("Failed to get tracker history from Redis: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get tracker history from Redis: {}", e),
                )
            })?;

        if !chart_history.is_empty() {
            let mut payload: Vec<TrackerPayloadWithId> = Vec::new();

            for payload_json in chart_history {
                let parsed: TrackerPayloadWithId =
                    serde_json::from_str(&payload_json).map_err(|e| {
                        eprintln!("Failed to parse tracker payload: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to parse tracker payload: {}", e),
                        )
                    })?;
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
