use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, Clone, FromRow, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct CarAudit {
    pub audit_id: i64,
    pub car_id: Option<i32>,
    pub tracker_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub recorded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetAuditResponse {
    pub total_count: usize,
    pub audit_records: Vec<CarAudit>,
}

#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    pub date: Option<String>,
    pub car_id: Option<i32>,
    pub tracker_id: Option<i32>,
}
