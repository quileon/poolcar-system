use deadpool_redis::redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    error::TasksError,
    models::mqtt::{MqttPayloadWithId, MqttPayloadWithTrackerCar},
    state::AppState,
};

#[derive(sqlx::FromRow)]
struct TrackerCar {
    tracker_id: i32,
    car_id: Option<i32>,
}

pub async fn audit_handler(state: Arc<AppState>) -> Result<(), TasksError> {
    let tracker_cars: Vec<TrackerCar> = sqlx::query_as(
        r#"
            SELECT
                trackers.tracker_id,
                cars.car_id as car_id
            FROM trackers
            LEFT JOIN cars ON trackers.tracker_id = cars.tracker_id
            WHERE trackers.deleted_at IS NULL
            ORDER BY trackers.tracker_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    // Get the latest payload from all of the existing tracker
    let mut tracker_payloads: Vec<(i32, Option<MqttPayloadWithTrackerCar>)> = Vec::new();
    let mut conn = state.redis.get().await?;
    for tracker_car in &tracker_cars {
        let tracker_payload_string: Option<String> = match conn
            .get(format!("tracker:{}:live", tracker_car.tracker_id))
            .await
        {
            Ok(payload) => payload,
            Err(_) => {
                tracing::warn!("No live data for tracker {}", tracker_car.tracker_id);
                None
            }
        };

        match tracker_payload_string {
            Some(tracker_payload_string) => {
                match serde_json::from_str::<MqttPayloadWithId>(&tracker_payload_string) {
                    Ok(payload) => {
                        let payload_with_car = MqttPayloadWithTrackerCar {
                            id: payload.id,
                            car_id: tracker_car.car_id.map(|id| id as u8),
                            uptime: payload.uptime,
                            connection: payload.connection,
                            location: payload.location,
                            altitude: payload.altitude,
                            speed: payload.speed,
                            course: payload.course,
                            datetime: payload.datetime,
                            satellites: payload.satellites,
                            hdop: payload.hdop,
                            stats: payload.stats,
                        };
                        tracker_payloads.push((tracker_car.tracker_id, Some(payload_with_car)));
                    }
                    Err(e) => {
                        tracing::error!("Failed to deserialize tracker payload: {}", e);
                        continue;
                    }
                }
            }
            None => {
                tracker_payloads.push((tracker_car.tracker_id, None));
            }
        }
    }

    // Save all into database
    for payload in &tracker_payloads {
        if let (_, Some(payload)) = payload {
            sqlx::query(
                r#"
                INSERT INTO car_audit (
                    car_id,
                    tracker_id,
                    latitude,
                    longitude
                )
                VALUES (?, ?, ?, ?)
            "#,
            )
            .bind(payload.car_id.map(|id| id as i32))
            .bind(payload.id as i32)
            .bind(payload.location.latitude)
            .bind(payload.location.longitude)
            .execute(&state.db)
            .await?;
        }
    }

    let mut tracker_payloads_map: HashMap<&i32, &Option<MqttPayloadWithTrackerCar>> =
        HashMap::new();

    // Add all trackers with live data
    for (payload_id, payload) in &tracker_payloads {
        tracker_payloads_map.insert(payload_id, payload);
    }

    // Broadcast audit to WebSocket clients
    if !tracker_payloads_map.is_empty() {
        let ws_message = serde_json::json!({
            "message_type": "audit",
            "data": tracker_payloads_map
        });
        let ws_message_string = serde_json::to_string(&ws_message)?;

        match state.tx.send(ws_message_string) {
            Ok(_) => tracing::debug!("Audit broadcasted to WebSockets"),
            Err(e) => tracing::warn!("Failed to broadcast audit to WebSockets: {}", e),
        }
    }

    Ok(())
}
