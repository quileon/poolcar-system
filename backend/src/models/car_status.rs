use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct CarStatus {
    pub car_status_id: i32,
    pub car_id: i32,
    pub gas_level: f64,
    pub kilometres: f64,
    pub recorded_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct CarBody {
    pub gas_level: f64,
    pub kilometres: f64,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct CarDetails {
    pub car_status_id: i32,
    pub car_id: i32,
    pub car_name: String,
    pub car_police_number: String,
    pub gas_level: f64,
    pub kilometres: f64,
    pub recorded_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetCarsResponse {
    pub car_statuses: Vec<CarDetails>,
    pub car_status_count: usize,
}
