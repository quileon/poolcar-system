use anyhow::anyhow;
use bytes::Bytes;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub async fn handle_mqtt_payload(
    payload: Bytes,
    _db: &sea_orm::DatabaseConnection,
    redis: &redis::Client,
    tx: &tokio::sync::broadcast::Sender<String>,
) -> anyhow::Result<()> {
    let parsed_payload: MqttPayload = serde_json::from_slice(&payload)?;
    debug!("{:?}", payload);

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

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadConnection {
    pub interval: u32,
    pub retries: u32,
    pub sequence_id: u32,
    pub iteration_id: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadLocation {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadAltitude {
    pub meters: Option<f64>,
    pub feet: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadSpeed {
    pub kmph: Option<f64>,
    pub mph: Option<f64>,
    pub mps: Option<f64>,
    pub knots: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadCourse {
    pub degrees: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadSatellites {
    pub visible: Option<u8>,
    pub used: Option<u8>,
    pub carrier_to_noise: Option<u8>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadDop {
    pub hdop: Option<f64>,
    pub pdop: Option<f64>,
    pub vdop: Option<f64>,
    pub age: Option<u32>,
    pub valid: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadStats {
    pub chars_processed: Option<u32>,
    pub sentences_with_fix: Option<u32>,
    pub failed_checksum: Option<u32>,
    pub passed_checksum: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PayloadNetwork {
    pub rssi: Option<u32>,
    pub lac: Option<String>,
    pub ci: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MqttPayload {
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
