use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadConnection {
    pub interval: u32,
    pub retries: u32,
    pub sequence_id: u32,
    pub iteration_id: u32,
    pub strength: u32,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadLocation {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadAltitude {
    pub meters: Option<f32>,
    pub feet: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadSpeed {
    pub kmph: Option<f32>,
    pub mph: Option<f32>,
    pub mps: Option<f32>,
    pub knots: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadCourse {
    pub degrees: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
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

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadSatellites {
    pub count: Option<u8>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadHdop {
    pub value: Option<f32>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PayloadStats {
    pub chars_processed: Option<u32>,
    pub sentences_with_fix: Option<u32>,
    pub failed_checksum: Option<u32>,
    pub passed_checksum: Option<u32>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct MqttPayload {
    pub uptime: u32,
    pub connection: PayloadConnection,
    pub location: PayloadLocation,
    pub altitude: PayloadAltitude,
    pub speed: PayloadSpeed,
    pub course: PayloadCourse,
    pub datetime: PayloadDateTime,
    pub satellites: PayloadSatellites,
    pub hdop: PayloadHdop,
    pub stats: PayloadStats,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct MqttPayloadWithId {
    pub id: u8,
    pub uptime: u32,
    pub connection: PayloadConnection,
    pub location: PayloadLocation,
    pub altitude: PayloadAltitude,
    pub speed: PayloadSpeed,
    pub course: PayloadCourse,
    pub datetime: PayloadDateTime,
    pub satellites: PayloadSatellites,
    pub hdop: PayloadHdop,
    pub stats: PayloadStats,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct MqttPayloadWithTrackerCar {
    pub id: u8,
    pub car_id: Option<u8>,
    pub uptime: u32,
    pub connection: PayloadConnection,
    pub location: PayloadLocation,
    pub altitude: PayloadAltitude,
    pub speed: PayloadSpeed,
    pub course: PayloadCourse,
    pub datetime: PayloadDateTime,
    pub satellites: PayloadSatellites,
    pub hdop: PayloadHdop,
    pub stats: PayloadStats,
}
