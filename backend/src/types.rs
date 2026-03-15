use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct Claims {
    pub username: String,
    pub role_name: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SuccessResponse {
    pub status: String,
    pub message: String,
}

impl SuccessResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            status: "success".to_string(),
            message: message.into(),
        }
    }
}
