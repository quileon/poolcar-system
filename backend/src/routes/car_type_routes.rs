use crate::{
    error::AppError,
    models::car_type::{CarType, CarTypeBody, CarTypeDetails, GetCarTypesResponse},
    types::PaginationParams,
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

pub async fn get_car_types(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let car_types = sqlx::query_as!(
        CarTypeDetails,
        r#"
            SELECT
                car_types.car_type_id,
                car_types.name,
                COUNT(cars.car_id) as car_count,
                car_types.created_at,
                car_types.updated_at,
                car_types.deleted_at
            FROM car_types
            LEFT JOIN cars ON car_types.car_type_id = cars.car_type_id
            WHERE
                CASE
                    WHEN $1 = 'active' THEN car_types.deleted_at IS NULL
                    WHEN $1 = 'deleted' THEN car_types.deleted_at IS NOT NULL
                    WHEN $1 = 'all' THEN TRUE
                    ELSE car_types.deleted_at IS NULL
                END
            GROUP BY car_types.car_type_id, car_types.name
            ORDER BY car_types.car_type_id ASC
        "#,
        status
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetCarTypesResponse {
        car_type_count: car_types.len(),
        car_types,
    };

    Ok(Json(response))
}

pub async fn get_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let car_type = sqlx::query_as!(
        CarTypeDetails,
        r#"
            SELECT
                car_types.car_type_id,
                car_types.name,
                COUNT(cars.car_id) as car_count,
                car_types.created_at,
                car_types.updated_at,
                car_types.deleted_at
            FROM car_types
            LEFT JOIN cars ON car_types.car_type_id = cars.car_type_id
            WHERE car_types.car_type_id = $1
            GROUP BY car_types.car_type_id, car_types.name
        "#,
        car_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(car_type))
}

pub async fn create_car_type(
    State(state): State<Arc<AppState>>,
    Json(car_type): Json<CarTypeBody>,
) -> Result<impl IntoResponse, AppError> {
    let created_car_type = sqlx::query_as!(
        CarType,
        r#"
            INSERT INTO car_types (name)
            VALUES ($1)
            RETURNING car_type_id, name, created_at, updated_at, deleted_at
        "#,
        car_type.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_car_type))
}

pub async fn update_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
    Json(car_type): Json<CarTypeBody>,
) -> Result<impl IntoResponse, AppError> {
    let updated_car_type = sqlx::query_as!(
        CarType,
        r#"
            UPDATE car_types
            SET name = $2
            WHERE car_type_id = $1
            RETURNING car_type_id, name, created_at, updated_at, deleted_at
        "#,
        car_type_id,
        car_type.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_car_type))
}

pub async fn delete_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_car_type = sqlx::query_as!(
        CarType,
        r#"
            UPDATE car_types
            SET deleted_at = NOW()
            WHERE car_type_id = $1
            RETURNING car_type_id, name, created_at, updated_at, deleted_at
        "#,
        car_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_car_type))
}

pub async fn restore_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let restore_car_type = sqlx::query_as!(
        CarType,
        r#"
            UPDATE car_types
            SET deleted_at = NULL
            WHERE car_type_id = $1
            RETURNING car_type_id, name, created_at, updated_at, deleted_at
        "#,
        car_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(restore_car_type))
}

pub async fn export_car_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let car_types = sqlx::query_as!(
        CarTypeDetails,
        r#"
            SELECT
                car_types.car_type_id,
                car_types.name,
                COUNT(cars.car_id) as car_count,
                car_types.created_at,
                car_types.updated_at,
                car_types.deleted_at
            FROM car_types
            LEFT JOIN cars ON car_types.car_type_id = cars.car_type_id
            GROUP BY car_types.car_type_id, car_types.name
            ORDER BY car_types.car_type_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Car Type ID",
            "Name",
            "Car Count",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for car_type in car_types {
            writer.serialize(car_type)?;
        }

        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (
                CONTENT_DISPOSITION,
                "attachment; filename=\"car-types.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_car_types).post(create_car_type))
        .route("/export", get(export_car_types))
        .route(
            "/{car_type_id}",
            get(get_car_type)
                .put(update_car_type)
                .delete(delete_car_type),
        )
        .route("/{car_type_id}/restore", put(restore_car_type))
}
