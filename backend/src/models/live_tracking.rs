use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

use crate::models::mqtt::{
    PayloadAltitude, PayloadConnection, PayloadCourse, PayloadDateTime, PayloadHdop,
    PayloadLocation, PayloadSatellites, PayloadSpeed, PayloadStats,
};

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetMqttPayloadHistory {
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
pub struct ActivityMarker {
    pub id: u8,
    #[ts(type = "POST | PUT | DELETE")]
    pub action: String,
    #[ts(type = "number | null")]
    pub latitude: Option<Decimal>,
    #[ts(type = "number | null")]
    pub longitude: Option<Decimal>,
}
