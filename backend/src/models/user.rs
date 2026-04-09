use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub user_role_id: i32,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct UserAuth {
    pub username: String,
    pub password: String,
    pub user_role_id: i32,
    pub user_role_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct UserDetails {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub user_role_id: i32,
    pub user_role_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct UserBody {
    pub username: String,
    pub password: Option<String>,
    pub email: String,
    pub full_name: String,
    pub user_role_id: i32,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq, TS)]
#[ts(export)]
pub struct GetUsersResponse {
    pub users: Vec<UserDetails>,
    pub user_count: usize,
}
