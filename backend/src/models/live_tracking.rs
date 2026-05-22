use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

use crate::models::mqtt::{
    PayloadAltitude, PayloadConnection, PayloadCourse, PayloadDateTime, PayloadDop,
    PayloadLocation, PayloadNetwork, PayloadSatellites, PayloadSpeed, PayloadStats,
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
    pub dop: PayloadDop,
    pub stats: PayloadStats,
    pub network: PayloadNetwork,
}
