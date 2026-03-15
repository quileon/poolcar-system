use axum::body::Bytes;
use deadpool_redis::redis::AsyncCommands;
use haversine_rs::point::Point;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::sync::Arc;

use crate::{
    error::MqttError,
    models::{mqtt::MqttPayloadWithId, websocket::DeleteActivity},
    redis::{complete_redis_activities, get_all_redis_activities},
    AppState,
};

/// MQTT payload handler
///
/// Handles MQTT payload sended by tracker.
/// Payload that has valid location will be saved into live redis and broadcasted to WebSockets.
///
/// Each payload also get compared with all active activities.
/// If the distance between tracker location and activity destination (contact) is lower than 100 meter, the activity will be completed.
pub async fn mqtt_handler(state: Arc<AppState>, payload: Bytes) -> Result<(), MqttError> {
    let tracker_payload: MqttPayloadWithId = serde_json::from_slice(&payload)?;
    if tracker_payload.location.latitude.is_none() || tracker_payload.location.longitude.is_none() {
        return Err(MqttError::InvalidLocation);
    }
    let tracker_payload_string = serde_json::to_string(&tracker_payload)?;

    let mut conn = state.redis.get().await?;

    // Redis (latest data)
    conn.set::<_, _, ()>(
        format!("tracker:{}:live", tracker_payload.id),
        &tracker_payload_string,
    )
    .await?;
    tracing::debug!("MQTT payload is saved into Redis");

    let ws_message = serde_json::json!({
        "message_type": "tracker_location",
        "data": tracker_payload
    });
    let ws_message_string = serde_json::to_string(&ws_message)?;
    match state.tx.send(ws_message_string) {
        Ok(_) => tracing::debug!("MQTT payload is broadcasted to WebSockets"),
        Err(e) => tracing::warn!("Failed to broadcast MQTT payload to WebSockets: {}", e),
    }

    let activities = get_all_redis_activities(&state.redis).await?;
    for activity in activities {
        let contact_latitude = match activity.contact_latitude.to_f64() {
            Some(lat) => lat,
            None => continue,
        };
        let contact_longitude = match activity.contact_longitude.to_f64() {
            Some(long) => long,
            None => continue,
        };
        let tracker_latitude = tracker_payload.location.latitude.unwrap_or(0.0);
        let tracker_longitude = tracker_payload.location.longitude.unwrap_or(0.0);

        let p1 = Point::new(contact_latitude, contact_longitude);
        let p2 = Point::new(tracker_latitude, tracker_longitude);

        let distance = haversine_rs::distance(p1, p2, haversine_rs::units::Unit::Meters);
        tracing::trace!(
            "Distance to activity {}: {}",
            activity.activity_id,
            distance
        );
        if distance < 100.0 {
            tracing::debug!(
                "Completing activity {} with distance {}",
                activity.activity_id,
                distance
            );
            complete_redis_activities(
                &state.db,
                &state.redis,
                activity.activity_id,
                tracker_payload.id,
                Decimal::from_f64_retain(tracker_latitude).unwrap_or(Decimal::ZERO),
                Decimal::from_f64_retain(tracker_longitude).unwrap_or(Decimal::ZERO),
            )
            .await?;

            let ws_message = serde_json::json!({
                "message_type": "remove_destination",
                "data": DeleteActivity {
                    activity_id: activity.activity_id as u8,
                }
            });
            let deleted_marker = serde_json::to_string(&ws_message)?;

            match state.tx.send(deleted_marker) {
                Ok(_) => tracing::debug!("Completed activity is broadcasted to WebSockets"),
                Err(e) => tracing::warn!(
                    "Failed to broadcast completed activity to WebSockets: {}",
                    e
                ),
            }

            break;
        }
    }

    Ok(())
}
