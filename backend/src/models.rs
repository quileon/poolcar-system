use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Car {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub tracker_id: Option<i32>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CarBody {
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub tracker_id: Option<i32>,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct CarWithTracker {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub car_type_name: String,
    pub tracker_id: Option<i32>,
    pub tracker_name: Option<String>,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct GetCarsResponse {
    pub cars: Vec<CarWithTracker>,
    pub car_count: usize,
}

#[derive(Debug, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct CarType {
    pub car_type_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct CarTypeWithCount {
    pub car_type_id: i32,
    pub name: String,
    pub car_count: i64,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetCarTypesResponse {
    pub car_types: Vec<CarTypeWithCount>,
    pub car_type_count: usize,
}

#[derive(Debug, FromRow, Deserialize, TS)]
#[ts(export)]
pub struct CarTypeBody {
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Tracker {
    pub tracker_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct TrackerWithDetails {
    pub tracker_id: i32,
    pub name: String,
    pub car_id: Option<i32>,
    pub car_name: Option<String>,
    pub car_type_id: Option<i32>,
    pub car_type_name: Option<String>,
}

#[derive(Debug, FromRow, Deserialize, TS)]
#[ts(export)]
pub struct TrackerBody {
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct GetTrackerResponse {
    pub trackers: Vec<TrackerWithDetails>,
    pub tracker_count: usize,
}

#[derive(Debug, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Contact {
    pub contact_id: i32,
    pub name: String,
    #[ts(type = "number")]
    pub latitude: Decimal,
    #[ts(type = "number")]
    pub longitude: Decimal,
    pub contact_type_id: i32,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ContactBody {
    pub name: String,
    #[ts(type = "number")]
    pub latitude: Decimal,
    #[ts(type = "number")]
    pub longitude: Decimal,
    pub contact_type_id: i32,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct ContactWithDetails {
    pub contact_id: i32,
    pub name: String,
    #[ts(type = "number")]
    pub latitude: Decimal,
    #[ts(type = "number")]
    pub longitude: Decimal,
    pub contact_type_id: i32,
    pub contact_type_name: String,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct GetContactsResponse {
    pub contacts: Vec<ContactWithDetails>,
    pub contact_count: usize,
}

#[derive(Debug, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ContactType {
    pub contact_type_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct ContactTypeWithCount {
    pub contact_type_id: i32,
    pub name: String,
    pub contact_count: i64,
}

#[derive(Debug, FromRow, Deserialize, TS)]
#[ts(export)]
pub struct ContactTypeBody {
    pub name: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetContactTypesResponse {
    pub contact_types: Vec<ContactTypeWithCount>,
    pub contact_type_count: usize,
}

#[derive(Debug, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Activity {
    pub activity_id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ActivityBody {
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct ActivityWithCount {
    pub activity_id: i32,
    pub name: String,
    pub activity_count: i64,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetActivitiesResponse {
    pub activities: Vec<ActivityWithCount>,
    pub activity_count: usize,
}

#[derive(Debug, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct History {
    pub history_id: i32,
    pub car_id: Option<i32>,
    pub contact_id: i32,
    pub activity_id: i32,
    pub tracker_id: Option<i32>,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    #[ts(type = "number")]
    pub finished_latitude: Option<Decimal>,
    #[ts(type = "number")]
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct HistoryBody {
    pub car_id: Option<i32>,
    pub contact_id: i32,
    pub activity_id: i32,
    pub tracker_id: Option<i32>,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    #[ts(type = "number")]
    pub finished_latitude: Option<Decimal>,
    #[ts(type = "number")]
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, FromRow, Serialize, TS)]
#[ts(export)]
pub struct HistoryWithDetails {
    pub history_id: i32,
    pub car_id: Option<i32>,
    pub car_name: Option<String>,
    pub contact_id: i32,
    pub contact_name: String,
    pub activity_id: i32,
    pub activity_name: String,
    pub tracker_id: Option<i32>,
    pub tracker_name: Option<String>,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    #[ts(type = "number")]
    pub finished_latitude: Option<Decimal>,
    #[ts(type = "number")]
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetHistoriesResponse {
    pub histories: Vec<HistoryWithDetails>,
    pub history_count: usize,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerConnection {
    pub interval: u32,
    pub retries: u32,
    pub sequence_id: u32,
    pub iteration_id: u32,
    pub strength: u32,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerLocation {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerAltitude {
    pub meters: Option<f32>,
    pub feet: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerSpeed {
    pub kmph: Option<f32>,
    pub mph: Option<f32>,
    pub mps: Option<f32>,
    pub knots: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerCourse {
    pub degrees: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
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

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerSatellites {
    pub count: Option<u8>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerHdop {
    pub value: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerStats {
    pub chars_processed: Option<u32>,
    pub sentences_with_fix: Option<u32>,
    pub failed_checksum: Option<u32>,
    pub passed_checksum: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
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

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TrackerPayloadWithId {
    pub id: u8,
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
