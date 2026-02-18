use crate::{
    error::AppError,
    models::car::{Car, CarBody, CarDetails, GetCarsResponse},
    routes::car_type_routes,
    types::PaginationParams,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use std::sync::Arc;

pub async fn get_cars(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let cars = sqlx::query_as!(
        CarDetails,
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id as "tracker_id?",
                trackers.name as "tracker_name?",
                cars.created_at,
                cars.updated_at,
                cars.deleted_at
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            ORDER BY cars.car_id ASC
            LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64,
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetCarsResponse {
        car_count: cars.len(),
        cars,
    };

    Ok(axum::Json(response))
}

pub async fn get_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let car = sqlx::query_as!(
        CarDetails,
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id as "tracker_id?",
                trackers.name as "tracker_name?",
                cars.created_at,
                cars.updated_at,
                cars.deleted_at
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            WHERE cars.car_id = $1
        "#,
        car_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(axum::Json(car))
}

pub async fn create_car(
    State(state): State<Arc<AppState>>,
    Json(car): Json<CarBody>,
) -> Result<impl IntoResponse, AppError> {
    let created_car = sqlx::query_as!(
        Car,
        r#"
            INSERT INTO cars (name, police_number, active, car_type_id, tracker_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING car_id, name, police_number, active, car_type_id, tracker_id, created_at, updated_at, deleted_at
        "#,
        car.name,
        car.police_number,
        car.active,
        car.car_type_id,
        car.tracker_id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_car))
}

pub async fn update_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
    Json(car): Json<CarBody>,
) -> Result<impl IntoResponse, AppError> {
    let updated_car = sqlx::query_as!(
        Car,
        r#"
            UPDATE cars
            SET
                name = $2,
                police_number = $3,
                active = $4,
                car_type_id = $5,
                tracker_id = $6
            WHERE car_id = $1
            RETURNING car_id, name, police_number, active, car_type_id, tracker_id, created_at, updated_at, deleted_at
        "#,
        car_id,
        car.name,
        car.police_number,
        car.active,
        car.car_type_id,
        car.tracker_id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_car))
}

pub async fn delete_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_car = sqlx::query_as!(
        Car,
        r#"
            UPDATE cars
            SET deleted_at = NOW(), tracker_id = NULL
            WHERE car_id = $1
            RETURNING car_id, name, police_number, active, car_type_id, tracker_id, created_at, updated_at, deleted_at
        "#,
        car_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_car))
}

pub async fn restore_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let restored_car = sqlx::query_as!(
        Car,
        r#"
            UPDATE cars
            SET deleted_at = NULL
            WHERE car_id = $1
            RETURNING car_id, name, police_number, active, car_type_id, tracker_id, created_at, updated_at, deleted_at
        "#,
        car_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(restored_car))
}

pub async fn export_cars(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let cars = sqlx::query_as!(
        CarDetails,
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id as "tracker_id?",
                trackers.name as "tracker_name?",
                cars.created_at,
                cars.updated_at,
                cars.deleted_at
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            ORDER BY cars.car_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Car ID",
            "Name",
            "Police Number",
            "Active",
            "Car Type ID",
            "Car Type Name",
            "Tracker ID",
            "Tracker Name",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for car in cars {
            writer.serialize(car)?;
        }

        writer.flush()?;
    }

    Ok((
        [
            ("Content-Type", "text/csv"),
            ("Content-Disposition", "attachment; filename=\"cars.csv\""),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_cars).post(create_car))
        .route("/export", get(export_cars))
        .nest("/types", car_type_routes::routes())
        .route("/{car_id}", get(get_car).put(update_car).delete(delete_car))
        .route("/{car_id}/restore", put(restore_car))
}
