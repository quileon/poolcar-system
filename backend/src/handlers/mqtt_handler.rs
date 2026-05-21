use axum::body::Bytes;
use deadpool_redis::redis::AsyncCommands;
use haversine_rs::point::Point;
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

    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        INSERT INTO hardware_test (
            tracker_id, uptime, 
            connection_interval, connection_retries, connection_sequence_id, connection_iteration_id, connection_strength,
            location_latitude, location_longitude, location_age, location_valid,
            altitude_meters, altitude_feet, altitude_age, altitude_valid,
            speed_kmph, speed_mph, speed_mps, speed_knots, speed_age, speed_valid,
            course_degrees, course_age, course_valid,
            datetime_iso8601, datetime_year, datetime_month, datetime_day, datetime_hour, datetime_minute, datetime_second, datetime_centisecond, datetime_age, datetime_valid,
            satellites_visible, satellites_used, satellites_carrier_to_noise, satellites_age, satellites_valid,
            dop_hdop, dop_pdop, dop_vdop, dop_age, dop_valid,
            stats_chars_processed, stats_sentences_with_fix, stats_failed_checksum, stats_passed_checksum,
            received_at
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )
        "#,
        tracker_payload.id, tracker_payload.uptime,
        tracker_payload.connection.interval, tracker_payload.connection.retries, tracker_payload.connection.sequence_id, tracker_payload.connection.iteration_id, tracker_payload.connection.strength,
        tracker_payload.location.latitude, tracker_payload.location.longitude, tracker_payload.location.age, tracker_payload.location.valid,
        tracker_payload.altitude.meters, tracker_payload.altitude.feet, tracker_payload.altitude.age, tracker_payload.altitude.valid,
        tracker_payload.speed.kmph, tracker_payload.speed.mph, tracker_payload.speed.mps, tracker_payload.speed.knots, tracker_payload.speed.age, tracker_payload.speed.valid,
        tracker_payload.course.degrees, tracker_payload.course.age, tracker_payload.course.valid,
        tracker_payload.datetime.iso8601, tracker_payload.datetime.year, tracker_payload.datetime.month, tracker_payload.datetime.day, tracker_payload.datetime.hour, tracker_payload.datetime.minute, tracker_payload.datetime.second, tracker_payload.datetime.centisecond, tracker_payload.datetime.age, tracker_payload.datetime.valid,
        tracker_payload.satellites.visible, tracker_payload.satellites.used, tracker_payload.satellites.carrier_to_noise, tracker_payload.satellites.age, tracker_payload.satellites.valid,
        tracker_payload.dop.hdop, tracker_payload.dop.pdop, tracker_payload.dop.vdop, tracker_payload.dop.age, tracker_payload.dop.valid,
        tracker_payload.stats.chars_processed, tracker_payload.stats.sentences_with_fix, tracker_payload.stats.failed_checksum, tracker_payload.stats.passed_checksum,
        now
    ).execute(&state.db).await?;
    tracing::debug!("MQTT payload is saved for testing into database");

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
        let contact_latitude = activity.contact_latitude;
        let contact_longitude = activity.contact_longitude;
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
                tracker_latitude,
                tracker_longitude,
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
