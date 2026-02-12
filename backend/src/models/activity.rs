use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct Activity {
    pub activity_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ActivityBody {
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct ActivityWithCount {
    pub activity_id: i32,
    pub name: String,
    pub activity_count: Option<i64>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetActivitiesResponse {
    pub activities: Vec<ActivityWithCount>,
    pub activity_count: usize,
}
