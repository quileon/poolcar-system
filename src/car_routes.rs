use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CarPaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

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
struct GetCarsResponse {
    cars: Vec<CarResponse>,
    car_count: usize,
}

pub async fn get_cars(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CarPaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let cars = sqlx::query_as::<Postgres, CarResponse>(
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                car_types.name as car_type_name,
                trackers.tracker_id,
                trackers.name as tracker_name
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            WHERE cars.deleted_at IS NULL
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

    let response = GetCarsResponse {
        car_count: cars.len(),
        cars,
    };

    Ok(axum::Json(response))
}
