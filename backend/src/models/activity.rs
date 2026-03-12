use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct Activity {
    pub activity_id: i32,
    pub car_id: Option<i32>,
    pub contact_id: i32,
    pub activity_type_id: i32,
    pub tracker_id: Option<i32>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    #[ts(type = "number | null")]
    pub finished_latitude: Option<Decimal>,
    #[ts(type = "number | null")]
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ActivityBody {
    pub car_id: Option<i32>,
    pub contact_id: i32,
    pub activity_type_id: i32,
    pub tracker_id: Option<i32>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    #[ts(type = "number | null")]
    pub finished_latitude: Option<Decimal>,
    #[ts(type = "number | null")]
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ActivityDetails {
    pub activity_id: i32,
    pub car_id: Option<i32>,
    pub car_name: Option<String>,
    pub car_police_number: Option<String>,
    pub contact_id: i32,
    pub contact_name: String,
    #[ts(type = "number")]
    pub contact_latitude: Decimal,
    #[ts(type = "number")]
    pub contact_longitude: Decimal,
    pub activity_type_id: i32,
    pub activity_type_name: String,
    pub tracker_id: Option<i32>,
    pub tracker_name: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    #[ts(type = "number | null")]
    pub finished_latitude: Option<Decimal>,
    #[ts(type = "number | null")]
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetActivitiesResponse {
    pub activities: Vec<ActivityDetails>,
    pub activity_count: usize,
}
