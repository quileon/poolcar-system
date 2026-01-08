use crate::{
    models::{History, PaginationParams},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct HistoryBody {
    pub car_id: i32,
    pub contact_id: i32,
    pub activity_id: i32,
    pub tracker_id: i32,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_latitude: Option<Decimal>,
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct HistoryWithDetails {
    pub history_id: i32,
    pub car_id: i32,
    pub car_name: String,
    pub contact_id: i32,
    pub contact_name: String,
    pub activity_id: i32,
    pub activity_name: String,
    pub tracker_id: i32,
    pub tracker_name: String,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_latitude: Option<Decimal>,
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
struct GetHistoriesResponse {
    histories: Vec<HistoryWithDetails>,
    history_count: usize,
}

pub async fn get_histories(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let histories = sqlx::query_as::<Postgres, HistoryWithDetails>(
        r#"
            SELECT
                histories.history_id,
                histories.car_id,
                cars.name AS car_name,
                histories.contact_id,
                contacts.name AS contact_name,
                histories.activity_id,
                activities.name AS activity_name,
                histories.tracker_id,
                trackers.name AS tracker_name,
                histories.finished_at,
                histories.started_at,
                histories.finished_latitude,
                histories.finished_longitude,
                histories.description
            FROM histories
            JOIN cars ON cars.car_id = histories.car_id
            JOIN activities ON activities.activity_id = histories.activity_id
            JOIN trackers ON trackers.tracker_id = histories.tracker_id
            WHERE histories.deleted_at IS NULL
            LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let response = GetHistoriesResponse {
        history_count: histories.len(),
        histories,
    };

    Ok(Json(response))
}

pub async fn create_history(
    State(state): State<Arc<AppState>>,
    Json(history): Json<HistoryBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let created_history = sqlx::query_as::<Postgres, History>(
        r#"
            INSERT INTO histories (car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING history_id, car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description
        "#,
    )
    .bind(history.car_id)
    .bind(history.contact_id)
    .bind(history.activity_id)
    .bind(history.tracker_id)
    .bind(history.finished_at)
    .bind(history.started_at)
    .bind(history.finished_latitude)
    .bind(history.finished_longitude)
    .bind(history.description)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(created_history))
}

pub async fn update_history(
    State(state): State<Arc<AppState>>,
    Path(history_id): Path<i32>,
    Json(history): Json<HistoryBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated_history = sqlx::query_as::<Postgres, History>(
        r#"
            UPDATE histories
            SET car_id = $2, contact_id = $3, activity_id = $4, tracker_id = $5, finished_at = $6, started_at = $7, finished_latitude = $8, finished_longitude = $9, description = $10
            WHERE history_id = $1
            RETURNING history_id, car_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description
        "#,
    )
    .bind(history_id)
    .bind(history.car_id)
    .bind(history.contact_id)
    .bind(history.activity_id)
    .bind(history.tracker_id)
    .bind(history.finished_at)
    .bind(history.started_at)
    .bind(history.finished_latitude)
    .bind(history.finished_longitude)
    .bind(history.description)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(updated_history))
}

pub async fn delete_history(
    State(state): State<Arc<AppState>>,
    Path(history_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let deleted_history = sqlx::query_as::<Postgres, History>(
        r#"
            UPDATE histories
            SET deleted_at = NOW()
            WHERE history_id = $1
            RETURNING history_id, car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description
        "#,
    )
    .bind(history_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(deleted_history))
}
