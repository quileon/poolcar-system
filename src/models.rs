use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Car {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub tracker_id: Option<i32>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Tracker {
    pub tracker_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
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
    pub car_id: i32,
    pub activity_id: i32,
    pub tracker_id: i32,
    pub finished_at: NaiveDateTime,
    pub started_at: NaiveDateTime,
    pub finished_latitude: Decimal,
    pub finished_longitude: Decimal,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
