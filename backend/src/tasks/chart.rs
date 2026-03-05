use crate::AppState;
use deadpool_redis::redis::{self, AsyncTypedCommands};
use std::sync::Arc;
use tokio::time;

pub async fn chart_loop(state: Arc<AppState>) {
    let mut interval = time::interval(time::Duration::from_mins(1));

    loop {
        interval.tick().await;
        tracing::debug!("Started saving MQTT payload for chart data");

        let tracker_ids: Vec<i32> = match sqlx::query_scalar(
            r#"
                SELECT tracker_id
                FROM trackers
                WHERE deleted_at IS NULL
                ORDER BY tracker_id ASC
            "#,
        )
        .fetch_all(&state.db)
        .await
        {
            Ok(ids) => ids,
            Err(err) => {
                eprintln!("Failed to fetch tracker IDs: {}", err);
                continue;
            }
        };

        let mut conn = match state.redis.get().await {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("Failed to get Redis connection: {}", err);
                continue;
            }
        };

        for tracker_id in tracker_ids {
            let tracker_payload = match conn.get(format!("tracker:{}:live", tracker_id)).await {
                Ok(payload) => payload,
                Err(err) => {
                    eprintln!("Failed to get tracker payload from Redis: {}", err);
                    continue;
                }
            };

            match tracker_payload {
                Some(payload) => {
                    match redis::pipe()
                        .atomic()
                        .rpush(format!("tracker:{}:history", tracker_id), payload)
                        .ltrim(format!("tracker:{}:history", tracker_id), -13, -1)
                        .query_async::<()>(&mut conn)
                        .await
                    {
                        Ok(_) => (),
                        Err(err) => {
                            eprintln!("Failed to update tracker history into Redis: {}", err);
                        }
                    }
                }
                None => {
                    match redis::pipe()
                        .atomic()
                        .rpush(format!("tracker:{}:history", tracker_id), "null")
                        .ltrim(format!("tracker:{}:history", tracker_id), -13, -1)
                        .query_async::<()>(&mut conn)
                        .await
                    {
                        Ok(_) => (),
                        Err(err) => {
                            eprintln!("Failed to update tracker history into Redis: {}", err);
                        }
                    }
                }
            }
        }
        tracing::debug!("Finished saving MQTT payload for chart data");
    }
}
