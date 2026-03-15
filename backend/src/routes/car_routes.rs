use crate::{
    error::AppError,
    models::car::{CarBody, CarDetails, GetCarsResponse},
    routes::car_type_routes,
    types::{PaginationParams, SuccessResponse},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use std::sync::Arc;

pub async fn get_cars(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let cars: Vec<CarDetails> = sqlx::query_as(
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id as tracker_id,
                trackers.name as tracker_name,
                cars.created_at,
                cars.updated_at,
                cars.deleted_at
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            WHERE
                CASE
                    WHEN ? = 'active' THEN cars.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN cars.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE cars.deleted_at IS NULL
                END
            ORDER BY cars.car_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
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
    let car: CarDetails = sqlx::query_as(
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id as tracker_id,
                trackers.name as tracker_name,
                cars.created_at,
                cars.updated_at,
                cars.deleted_at
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            WHERE cars.car_id = ?
        "#,
    )
    .bind(car_id)
    .fetch_one(&state.db)
    .await?;

    Ok(axum::Json(car))
}

pub async fn create_car(
    State(state): State<Arc<AppState>>,
    Json(car): Json<CarBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO cars (name, police_number, active, car_type_id, tracker_id)
            VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&car.name)
    .bind(&car.police_number)
    .bind(car.active)
    .bind(car.car_type_id)
    .bind(car.tracker_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car created successfully")))
}

pub async fn update_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
    Json(car): Json<CarBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE cars
            SET name = ?, police_number = ?, active = ?, car_type_id = ?, tracker_id = ?
            WHERE car_id = ?
        "#,
    )
    .bind(&car.name)
    .bind(&car.police_number)
    .bind(car.active)
    .bind(car.car_type_id)
    .bind(car.tracker_id)
    .bind(car_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car updated successfully")))
}

pub async fn delete_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE cars
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE car_id = ?
        "#,
    )
    .bind(car_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car deleted successfully")))
}

pub async fn restore_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE cars
            SET deleted_at = NULL
            WHERE car_id = ?
        "#,
    )
    .bind(car_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car restored successfully")))
}

pub async fn export_cars(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let cars: Vec<CarDetails> = sqlx::query_as(
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id as tracker_id,
                trackers.name as tracker_name,
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
            (CONTENT_TYPE, "text/csv"),
            (CONTENT_DISPOSITION, "attachment; filename=\"cars.csv\""),
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
