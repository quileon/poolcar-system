use crate::{
    error::AppError,
    models::car_status::{CarBody, CarDetails, GetCarsResponse},
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

pub async fn get_car_statuses(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let car_statuses: Vec<CarDetails> = sqlx::query_as(
        r#"
            SELECT
                car_statuses.car_status_id,
                car_statuses.car_id,
                cars.name as car_name,
                cars.police_number as car_police_number,
                car_statuses.gas_level,
                car_statuses.kilometres,
                car_statuses.recorded_at,
                car_statuses.created_at,
                car_statuses.updated_at,
                car_statuses.deleted_at
            FROM car_statuses
            LEFT JOIN cars ON car_statuses.car_id = cars.car_id
            WHERE
                CASE
                    WHEN ? = 'active' THEN car_statuses.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN car_statuses.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE car_statuses.deleted_at IS NULL
                END
            ORDER BY car_statuses.car_status_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
    .fetch_all(&state.db)
    .await?;

    let response = GetCarsResponse {
        car_status_count: car_statuses.len(),
        car_statuses,
    };

    Ok(Json(response))
}

pub async fn get_car_status(
    State(state): State<Arc<AppState>>,
    Path(car_status_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let car_status: CarDetails = sqlx::query_as(
        r#"
            SELECT
                car_statuses.car_status_id,
                car_statuses.car_id,
                cars.name as car_name,
                cars.police_number as car_police_number,
                car_statuses.gas_level,
                car_statuses.kilometres,
                car_statuses.recorded_at,
                car_statuses.created_at,
                car_statuses.updated_at,
                car_statuses.deleted_at
            FROM car_statuses
            LEFT JOIN cars ON car_statuses.car_id = cars.car_id
            WHERE car_statuses.car_status_id = ?
        "#,
    )
    .bind(car_status_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(car_status))
}

pub async fn create_car_status(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
    Json(car_status): Json<CarBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO car_statuses (car_id, gas_level, kilometres, recorded_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(car_id)
    .bind(car_status.gas_level)
    .bind(car_status.kilometres)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Car status created successfully",
    )))
}

pub async fn update_car_status(
    State(state): State<Arc<AppState>>,
    Path(car_status_id): Path<i32>,
    Json(car_status): Json<CarBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE car_statuses
            SET gas_level = ?, kilometres = ?, recorded_at = CURRENT_TIMESTAMP
            WHERE car_status_id = ?
        "#,
    )
    .bind(car_status.gas_level)
    .bind(car_status.kilometres)
    .bind(car_status_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Car status updated successfully",
    )))
}

pub async fn delete_car_status(
    State(state): State<Arc<AppState>>,
    Path(car_status_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE car_statuses
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE car_status_id = ?
        "#,
    )
    .bind(car_status_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Car status deleted successfully",
    )))
}

pub async fn restore_car_status(
    State(state): State<Arc<AppState>>,
    Path(car_status_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE car_statuses
            SET deleted_at = NULL
            WHERE car_status_id = ?
        "#,
    )
    .bind(car_status_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Car status restored successfully",
    )))
}

pub async fn export_car_statuses(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let car_statuses: Vec<CarDetails> = sqlx::query_as(
        r#"
            SELECT
                car_statuses.car_status_id,
                car_statuses.car_id,
                cars.name as car_name,
                cars.police_number as car_police_number,
                car_statuses.gas_level,
                car_statuses.kilometres,
                car_statuses.recorded_at,
                car_statuses.created_at,
                car_statuses.updated_at,
                car_statuses.deleted_at
            FROM car_statuses
            LEFT JOIN cars ON car_statuses.car_id = cars.car_id
            ORDER BY car_statuses.car_status_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Car Status ID",
            "Car ID",
            "Car Name",
            "Car Police Number",
            "Gas Level",
            "Kilometres",
            "Recorded At",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for car_status in car_statuses {
            writer.serialize(car_status)?;
        }

        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (
                CONTENT_DISPOSITION,
                "attachment; filename=\"car_statuses.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_car_statuses).post(create_car_status))
        .route("/export", get(export_car_statuses))
        .route(
            "/{car_status_id}",
            get(get_car_status)
                .put(update_car_status)
                .delete(delete_car_status),
        )
        .route("/{car_status_id}/restore", put(restore_car_status))
}
