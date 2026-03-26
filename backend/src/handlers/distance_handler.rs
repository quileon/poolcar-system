use deadpool_redis::redis::AsyncCommands;
use haversine_rs::point::Point;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    error::TasksError,
    models::{activity::ActivityDetails, mqtt::MqttPayloadWithId, websocket::Distances},
    state::AppState,
};

pub async fn distance_handler(state: Arc<AppState>) -> Result<(), TasksError> {
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

    // Get the latest payload from all of the existing tracker
    let mut tracker_payloads: Vec<MqttPayloadWithId> = Vec::new();
    for tracker_id in tracker_ids {
        let tracker_payload_string: String =
            match conn.get(format!("tracker:{}:live", tracker_id)).await {
                Ok(payload) => payload,
                Err(_) => {
                    tracing::warn!("No live data for tracker {}", tracker_id);
                    continue;
                }
            };

        match serde_json::from_str::<MqttPayloadWithId>(&tracker_payload_string) {
            Ok(payload) => tracker_payloads.push(payload),
            Err(e) => {
                tracing::error!("Failed to deserialize tracker payload: {}", e);
                continue;
            }
        }
    }

    // Get all activities and get the tracker with the closest distance for each activity
    let active_activities_string: String = conn.get("activities").await?;
    let active_activities: Vec<ActivityDetails> = serde_json::from_str(&active_activities_string)?;

    let mut distances_map: HashMap<i32, Distances> = HashMap::new();

    for activity in active_activities {
        if activity.finished_at.is_some() {
            continue;
        }

        let contact_latitude = match activity.contact_latitude.to_f64() {
            Some(lat) => lat,
            None => continue,
        };
        let contact_longitude = match activity.contact_longitude.to_f64() {
            Some(long) => long,
            None => continue,
        };

        let contact_point = Point::new(contact_latitude, contact_longitude);
        let mut closest_distance = f64::MAX;
        let mut closest_tracker: Option<&MqttPayloadWithId> = None;

        for tracker_payload in &tracker_payloads {
            let tracker_latitude = tracker_payload.location.latitude.unwrap_or(0.0);
            let tracker_longitude = tracker_payload.location.longitude.unwrap_or(0.0);

            let tracker_point = Point::new(tracker_latitude, tracker_longitude);
            let distance = haversine_rs::distance(
                contact_point,
                tracker_point,
                haversine_rs::units::Unit::Kilometers,
            );

            if distance < closest_distance {
                closest_distance = distance;
                closest_tracker = Some(tracker_payload);
            }
        }

        if let Some(tracker) = closest_tracker {
            let car_data: Option<(i32, String, String)> = sqlx::query_as(
                r#"
                    SELECT car_id, name, police_number
                    FROM cars
                    WHERE tracker_id = ? AND deleted_at IS NULL
                    LIMIT 1
                "#,
            )
            .bind(tracker.id as i32)
            .fetch_optional(&state.db)
            .await?;

            let (car_id, car_name, car_police_number) = match car_data {
                Some((id, name, number)) => (Some(id as u8), Some(name), Some(number)),
                None => (None, None, None),
            };

            distances_map.insert(
                activity.activity_id,
                Distances {
                    activity_id: activity.activity_id as u8,
                    tracker_id: tracker.id,
                    tracker_name: activity.tracker_name.unwrap_or_default(),
                    car_id,
                    car_name,
                    car_police_number,
                    distance: closest_distance,
                },
            );

            tracing::trace!(
                "Activity {} (tracker {}) closest distance: {}m",
                activity.activity_id,
                tracker.id,
                closest_distance
            );
        }
    }

    // Broadcast distances to WebSocket clients
    if !distances_map.is_empty() {
        let ws_message = serde_json::json!({
            "message_type": "distances",
            "data": distances_map
        });
        let ws_message_string = serde_json::to_string(&ws_message)?;

        match state.tx.send(ws_message_string) {
            Ok(_) => tracing::debug!("Distances broadcasted to WebSockets"),
            Err(e) => tracing::warn!("Failed to broadcast distances to WebSockets: {}", e),
        }
    }

    Ok(())
}
