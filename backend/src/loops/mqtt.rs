use anyhow::anyhow;
use bytes::Bytes;
use haversine_rs::{point::Point, units::Unit};
use redis::AsyncCommands;
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{entities::hardware_test, pages::trips};

pub async fn handle_mqtt_payload(
    payload: Bytes,
    db: &sea_orm::DatabaseConnection,
    redis: &redis::Client,
    tx: &tokio::sync::broadcast::Sender<String>,
) -> anyhow::Result<()> {
    let parsed_payload: MqttPayload = serde_json::from_slice(&payload)?;
    // For testing purposes
    let p = parsed_payload.clone();
    let new_test = hardware_test::ActiveModel {
        tracker_id: Set(p.id as i32),
        uptime: Set(p.uptime),
        connection_interval: Set(p.connection.interval),
        connection_retries: Set(p.connection.retries),
        connection_sequence_id: Set(p.connection.sequence_id),
        connection_iteration_id: Set(p.connection.iteration_id),
        network_rssi: Set(p.network.rssi),
        network_lac: Set(p.network.lac),
        network_ci: Set(p.network.ci),
        location_latitude: Set(p.location.latitude),
        location_longitude: Set(p.location.longitude),
        location_age: Set(p.location.age),
        location_valid: Set(p.location.valid as i8),
        altitude_meters: Set(p.altitude.meters),
        altitude_feet: Set(p.altitude.feet),
        altitude_age: Set(p.altitude.age),
        altitude_valid: Set(p.altitude.valid as i8),
        speed_kmph: Set(p.speed.kmph),
        speed_mph: Set(p.speed.mph),
        speed_mps: Set(p.speed.mps),
        speed_knots: Set(p.speed.knots),
        speed_age: Set(p.speed.age),
        speed_valid: Set(p.speed.valid as i8),
        course_degrees: Set(p.course.degrees),
        course_age: Set(p.course.age),
        course_valid: Set(p.course.valid as i8),
        datetime_iso8601: Set(p.datetime.iso8601),
        datetime_year: Set(p.datetime.year),
        datetime_month: Set(p.datetime.month.map(|v| v as u32)),
        datetime_day: Set(p.datetime.day.map(|v| v as u32)),
        datetime_hour: Set(p.datetime.hour.map(|v| v as u32)),
        datetime_minute: Set(p.datetime.minute.map(|v| v as u32)),
        datetime_second: Set(p.datetime.second.map(|v| v as u32)),
        datetime_centisecond: Set(p.datetime.centisecond),
        datetime_age: Set(p.datetime.age),
        datetime_valid: Set(p.datetime.valid as i8),
        satellites_visible: Set(p.satellites.visible.map(|v| v as u32)),
        satellites_used: Set(p.satellites.used.map(|v| v as u32)),
        satellites_carrier_to_noise: Set(p.satellites.carrier_to_noise.map(|v| v as u32)),
        satellites_age: Set(p.satellites.age),
        satellites_valid: Set(p.satellites.valid as i8),
        dop_hdop: Set(p.dop.hdop),
        dop_pdop: Set(p.dop.pdop),
        dop_vdop: Set(p.dop.vdop),
        dop_age: Set(p.dop.age),
        dop_valid: Set(p.dop.valid as i8),
        stats_chars_processed: Set(p.stats.chars_processed),
        stats_sentences_with_fix: Set(p.stats.sentences_with_fix),
        stats_failed_checksum: Set(p.stats.failed_checksum),
        stats_passed_checksum: Set(p.stats.passed_checksum),
        received_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };
    hardware_test::Entity::insert(new_test).exec(db).await?;

    if !parsed_payload.location.valid {
        return Err(anyhow!("Location is not valid."));
    }

    let mut redis_conn = redis.get_multiplexed_async_connection().await?;
    redis_conn
        .set::<_, _, ()>(
            format!("tracker:{}:live", parsed_payload.id),
            payload.as_ref(),
        )
        .await?;

    let ws_message = serde_json::json!({
        "message_type": "tracker_location",
        "data": parsed_payload,
    });
    let _ = tx.send(ws_message.to_string());

    let p1 = Point::new(
        parsed_payload.location.latitude.unwrap_or(0.0),
        parsed_payload.location.longitude.unwrap_or(0.0),
    );
    let trips = trips::get_active_trips(db, redis).await?;
    for trip in trips {
        let p2 = Point::new(trip.contact_latitude, trip.contact_longitude);
        let distance = haversine_rs::distance(p1, p2, Unit::Meters);
        debug!(
            "Distance from tracker {} to trip {}: {} meters",
            parsed_payload.id, trip.trip.activity_id, distance
        );
        if distance <= 100.0 {
            debug!(
                "Tracker {} is within 100 meters of trip {}",
                parsed_payload.id, trip.trip.activity_id
            );
            trips::finish_trip(
                trip.trip.activity_id,
                parsed_payload.id as i32,
                parsed_payload.location.latitude.unwrap_or(0.0),
                parsed_payload.location.longitude.unwrap_or(0.0),
                db,
                redis,
                tx,
            )
            .await?;

            break;
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadConnection {
    pub interval: u32,
    pub retries: u32,
    pub sequence_id: u32,
    pub iteration_id: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadLocation {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadAltitude {
    pub meters: Option<f64>,
    pub feet: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadSpeed {
    pub kmph: Option<f64>,
    pub mph: Option<f64>,
    pub mps: Option<f64>,
    pub knots: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadCourse {
    pub degrees: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadDateTime {
    pub iso8601: Option<String>,
    pub year: Option<u32>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub centisecond: Option<u32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadSatellites {
    pub visible: Option<u8>,
    pub used: Option<u8>,
    pub carrier_to_noise: Option<u8>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadDop {
    pub hdop: Option<f64>,
    pub pdop: Option<f64>,
    pub vdop: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadStats {
    pub chars_processed: Option<u32>,
    pub sentences_with_fix: Option<u32>,
    pub failed_checksum: Option<u32>,
    pub passed_checksum: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PayloadNetwork {
    pub rssi: Option<u32>,
    pub lac: Option<String>,
    pub ci: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MqttPayload {
    pub id: u32,
    pub uptime: u32,
    pub connection: PayloadConnection,
    pub location: PayloadLocation,
    pub altitude: PayloadAltitude,
    pub speed: PayloadSpeed,
    pub course: PayloadCourse,
    pub datetime: PayloadDateTime,
    pub satellites: PayloadSatellites,
    pub dop: PayloadDop,
    pub stats: PayloadStats,
    pub network: PayloadNetwork,
}
