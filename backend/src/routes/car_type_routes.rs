use crate::{
    error::AppError,
    models::car_type::{CarTypeBody, CarTypeDetails, GetCarTypesResponse},
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

pub async fn get_car_types(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let car_types: Vec<CarTypeDetails> = sqlx::query_as(
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
                    WHEN ? = 'active' THEN car_types.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN car_types.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE car_types.deleted_at IS NULL
                END
            GROUP BY car_types.car_type_id, car_types.name
            ORDER BY car_types.car_type_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
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
    let car_type: CarTypeDetails = sqlx::query_as(
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
            WHERE car_types.car_type_id = ?
            GROUP BY car_types.car_type_id, car_types.name
        "#,
    )
    .bind(car_type_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(car_type))
}

pub async fn create_car_type(
    State(state): State<Arc<AppState>>,
    Json(car_type): Json<CarTypeBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO car_types (name)
            VALUES (?)
        "#,
    )
    .bind(&car_type.name)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car type created successfully")))
}

pub async fn update_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
    Json(car_type): Json<CarTypeBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE car_types
            SET name = ?
            WHERE car_type_id = ?
        "#,
    )
    .bind(&car_type.name)
    .bind(car_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car type updated successfully")))
}

pub async fn delete_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE car_types
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE car_type_id = ?
        "#,
    )
    .bind(car_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car type deleted successfully")))
}

pub async fn restore_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE car_types
            SET deleted_at = NULL
            WHERE car_type_id = ?
        "#,
    )
    .bind(car_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Car type restored successfully")))
}

pub async fn export_car_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let car_types: Vec<CarTypeDetails> = sqlx::query_as(
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
