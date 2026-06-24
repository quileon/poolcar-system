use deadpool_redis::redis::AsyncCommands;
use haversine_rs::{distance, point::Point, units::Unit};
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
                            dop: payload.dop,
                            stats: payload.stats,
                            network: payload.network,
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
            // Compare with last audit location to avoid duplicate audits
            if let (Some(lat), Some(long)) = (payload.location.latitude, payload.location.longitude)
            {
                let last_location_key = format!("tracker:{}:last_audit_location", payload.id);
                let last_location: Option<String> = conn.get(&last_location_key).await?;

                if let Some(last_location) = last_location {
                    let last_location: serde_json::Value = serde_json::from_str(&last_location)?;
                    let last_lat = last_location["latitude"].as_f64().unwrap_or(0.0);
                    let last_long = last_location["longitude"].as_f64().unwrap_or(0.0);
                    let last_point = Point::new(last_lat, last_long);
                    let current_point = Point::new(lat, long);
                    let distance = distance(last_point, current_point, Unit::Meters);

                    if distance < 10.0 {
                        tracing::debug!(
                            "Tracker {} is within 10 meters of last audit location, skipping audit",
                            payload.id
                        );
                        continue;
                    }
                }

                // Update last audit location in Redis
                let new_location = serde_json::json!({
                    "latitude": lat,
                    "longitude": long
                });
                conn.set::<_, _, ()>(&last_location_key, serde_json::to_string(&new_location)?)
                    .await?;

                sqlx::query(
                    r#"
                    INSERT INTO audit (
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
                .bind(lat)
                .bind(long)
                .execute(&state.db)
                .await?;
            }
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
