use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct Contact {
    pub contact_id: i32,
    pub name: String,
    #[ts(type = "number")]
    pub latitude: Decimal,
    #[ts(type = "number")]
    pub longitude: Decimal,
    pub contact_type_id: i32,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ContactBody {
    pub name: String,
    #[ts(type = "number")]
    pub latitude: Decimal,
    #[ts(type = "number")]
    pub longitude: Decimal,
    pub contact_type_id: i32,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ContactWithDetails {
    pub contact_id: i32,
    pub name: String,
    #[ts(type = "number")]
    pub latitude: Decimal,
    #[ts(type = "number")]
    pub longitude: Decimal,
    pub contact_type_id: i32,
    pub contact_type_name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetContactsResponse {
    pub contacts: Vec<ContactWithDetails>,
    pub contact_count: usize,
}
