use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct CarType {
    pub car_type_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct CarTypeWithCount {
    pub car_type_id: i32,
    pub name: String,
    pub car_count: Option<i64>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct CarTypeExportDetails {
    pub car_type_id: i32,
    pub name: String,
    pub car_count: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetCarTypesResponse {
    pub car_types: Vec<CarTypeWithCount>,
    pub car_type_count: usize,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct CarTypeBody {
    pub name: String,
}
