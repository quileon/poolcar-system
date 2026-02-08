use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use serde::Serialize;
use sqlx::{FromRow, Postgres};
use std::sync::Arc;

#[derive(Debug, FromRow, Serialize)]
struct CarResponse {
    car_id: i32,
    name: String,
    police_number: String,
    car_type_name: String,
    tracker_id: Option<i32>,
    tracker_name: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct TrackerResponse {
    tracker_id: i32,
    name: String,
}

#[derive(Debug, FromRow, Serialize)]
struct ActivityCountResponse {
    count: i64,
}

#[derive(Debug, Serialize)]
struct DashboardResponse {
    cars: Vec<CarResponse>,
    car_count: usize,
    trackers: Vec<TrackerResponse>,
    tracker_count: usize,
    active_activity_count: i64,
}

pub async fn get_dashboard_data(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cars = sqlx::query_as::<Postgres, CarResponse>(
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                car_types.name as car_type_name,
                trackers.tracker_id as tracker_id,
                trackers.name as tracker_name
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            WHERE cars.deleted_at IS NULL
            ORDER BY cars.car_id ASC
         "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let trackers = sqlx::query_as::<Postgres, TrackerResponse>(
        r#"
            SELECT
                trackers.tracker_id,
                trackers.name
            FROM trackers
            WHERE trackers.deleted_at IS NULL
         "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let activity_count_result = sqlx::query_as::<Postgres, ActivityCountResponse>(
        r#"
            SELECT COUNT(*) as count
            FROM histories
            WHERE finished_at IS NULL
            AND deleted_at IS NULL
        "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let response = DashboardResponse {
        car_count: cars.len(),
        tracker_count: trackers.len(),
        active_activity_count: activity_count_result.count,
        cars,
        trackers,
    };

    Ok(axum::Json(response))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_dashboard_data))
}
