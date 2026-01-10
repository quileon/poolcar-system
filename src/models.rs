use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Car {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub tracker_id: Option<i32>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Tracker {
    pub tracker_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CarType {
    pub car_type_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct ContactType {
    pub contact_type_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Contact {
    pub contact_id: i32,
    pub name: String,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub contact_type_id: i32,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Activity {
    pub activity_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct History {
    pub history_id: i32,
    pub car_id: Option<i32>,
    pub contact_id: i32,
    pub activity_id: i32,
    pub tracker_id: Option<i32>,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_latitude: Option<Decimal>,
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerConnection {
    pub interval: u32,
    pub retries: u32,
    pub sequence_id: u32,
    pub iteration_id: u32,
    pub strength: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerLocation {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerAltitude {
    pub meters: Option<f32>,
    pub feet: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerSpeed {
    pub kmph: Option<f32>,
    pub mph: Option<f32>,
    pub mps: Option<f32>,
    pub knots: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerCourse {
    pub degrees: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerDateTime {
    pub iso8601: Option<String>,
    pub year: Option<u32>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub millisecond: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerSatellites {
    pub count: Option<u8>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerHdop {
    pub value: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerStats {
    pub chars_processed: Option<u32>,
    pub sentences_with_fix: Option<u32>,
    pub failed_checksum: Option<u32>,
    pub passed_checksum: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerPayload {
    pub uptime: u32,
    pub connection: TrackerConnection,
    pub location: TrackerLocation,
    pub altitude: TrackerAltitude,
    pub speed: TrackerSpeed,
    pub course: TrackerCourse,
    pub datetime: TrackerDateTime,
    pub satellites: TrackerSatellites,
    pub hdop: TrackerHdop,
    pub stats: TrackerStats,
}
