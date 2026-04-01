use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct Contact {
    pub contact_id: i32,
    pub name: String,
    #[ts(type = "number")]
    pub latitude: f64,
    #[ts(type = "number")]
    pub longitude: f64,
    pub contact_type_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ContactBody {
    pub name: String,
    #[ts(type = "number")]
    pub latitude: f64,
    #[ts(type = "number")]
    pub longitude: f64,
    pub contact_type_id: i32,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ContactDetails {
    pub contact_id: i32,
    pub name: String,
    #[ts(type = "number")]
    pub latitude: f64,
    #[ts(type = "number")]
    pub longitude: f64,
    pub contact_type_id: i32,
    pub contact_type_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetContactsResponse {
    pub contacts: Vec<ContactDetails>,
    pub contact_count: usize,
}
