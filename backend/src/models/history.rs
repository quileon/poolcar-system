use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
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

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
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

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
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

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct HistoryExport {
    #[serde(rename = "History ID")]
    pub history_id: i32,
    #[serde(rename = "Car ID")]
    pub car_id: Option<i32>,
    #[serde(rename = "Car Name")]
    pub car_name: Option<String>,
    #[serde(rename = "Contact ID")]
    pub contact_id: i32,
    #[serde(rename = "Contact Name")]
    pub contact_name: String,
    #[serde(rename = "Activity ID")]
    pub activity_id: i32,
    #[serde(rename = "Activity Name")]
    pub activity_name: String,
    #[serde(rename = "Tracker ID")]
    pub tracker_id: Option<i32>,
    #[serde(rename = "Tracker Name")]
    pub tracker_name: Option<String>,
    #[serde(rename = "Finished At")]
    pub finished_at: Option<NaiveDateTime>,
    #[serde(rename = "Started At")]
    pub started_at: Option<NaiveDateTime>,
    #[serde(rename = "Finished Latitude")]
    #[ts(type = "number")]
    pub finished_latitude: Option<Decimal>,
    #[serde(rename = "Finished Longitude")]
    #[ts(type = "number")]
    pub finished_longitude: Option<Decimal>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Created At")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "Updated At")]
    pub updated_at: NaiveDateTime,
    #[serde(rename = "Deleted At")]
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetHistoriesResponse {
    pub histories: Vec<HistoryWithDetails>,
    pub history_count: usize,
}
