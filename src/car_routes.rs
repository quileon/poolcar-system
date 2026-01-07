use crate::{
    models::{Car, PaginationParams},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CarBody {
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub tracker_id: Option<i32>,
}

#[derive(Debug, FromRow, Serialize)]
struct CarWithTracker {
    car_id: i32,
    name: String,
    police_number: String,
    car_type_name: String,
    tracker_id: Option<i32>,
    tracker_name: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct GetCarsResponse {
    cars: Vec<CarWithTracker>,
    car_count: usize,
}

pub async fn get_cars(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let cars = sqlx::query_as::<Postgres, CarWithTracker>(
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

pub async fn create_car(
    State(state): State<Arc<AppState>>,
    Json(car): Json<CarBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let created_car = sqlx::query_as::<Postgres, Car>(
        r#"
            INSERT INTO cars (name, police_number, active, car_type_id, tracker_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING car_id, name, police_number, active, car_type_id, tracker_id
        "#,
    )
    .bind(car.name)
    .bind(car.police_number)
    .bind(car.active)
    .bind(car.car_type_id)
    .bind(car.tracker_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(created_car))
}

pub async fn update_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
    Json(car): Json<CarBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated_car = sqlx::query_as::<Postgres, Car>(
        r#"
            UPDATE cars
            SET name = $2, police_number = $3, active = $4, car_type_id = $5, tracker_id = $6
            WHERE car_id = $1
            RETURNING car_id, name, police_number, active, car_type_id, tracker_id
        "#,
    )
    .bind(car_id)
    .bind(car.name)
    .bind(car.police_number)
    .bind(car.active)
    .bind(car.car_type_id)
    .bind(car.tracker_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(updated_car))
}

pub async fn delete_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let deleted_car = sqlx::query_as::<Postgres, Car>(
        r#"
            UPDATE cars
            SET deleted_at = NOW()
            WHERE car_id = $1
            RETURNING car_id, name, police_number, active, car_type_id, tracker_id
        "#,
    )
    .bind(car_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(deleted_car))
}
