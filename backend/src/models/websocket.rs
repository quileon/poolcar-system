use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

/// Format that's used for WebSocket messages
/// There will be three format, each representing different messages.
///
/// No. 1, live Tracker location (real-time)
/// ```rust
/// {
///     "message_type": "tracker_location",
///     "data": MqttPayloadWithId
/// }
/// ```
///
/// No. 2, new or update destination (when created or updated)
/// ```rust
/// {
///     "message_type": "update_destination",
///     "data": UpdateActivity
/// }
/// ```
///
/// No. 3, delete destination (when deleted or finished)
/// ```rust
/// {
///     "message_type": "remove_destination",
///     "data": DeleteActivity
/// }
/// ```
///
/// No. 4, closest tracker distances to destination
/// ```rust
/// {
///     "message_type": "distances",
///     "data": [
///         Distances, ...
///     ]
/// }
/// ```
#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct DeleteActivity {
    pub activity_id: u8,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct UpdateActivity {
    pub activity_id: u8,
    pub contact_name: String,
    #[ts(type = "number")]
    pub contact_latitude: Decimal,
    #[ts(type = "number")]
    pub contact_longitude: Decimal,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct Distances {
    pub activity_id: u8,
    pub tracker_id: u8,
    pub tracker_name: String,
    pub car_id: u8,
    pub car_name: String,
    pub car_police_number: String,
    #[ts(type = "number")]
    pub distance: Decimal,
}
