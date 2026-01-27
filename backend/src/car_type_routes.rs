use crate::{
    models::{
        CarType, CarTypeBody, CarTypeExportDetails, CarTypeWithCount, GetCarTypesResponse,
        PaginationParams,
    },
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::Postgres;
use std::sync::Arc;

pub async fn get_car_types(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let car_types = sqlx::query_as::<Postgres, CarTypeWithCount>(
        r#"
            SELECT
                car_types.car_type_id,
                car_types.name,
                COUNT(cars.car_id) as car_count
            FROM car_types
            LEFT JOIN cars ON car_types.car_type_id = cars.car_type_id
            WHERE car_types.deleted_at IS NULL
            GROUP BY car_types.car_type_id, car_types.name
            ORDER BY car_types.car_type_id ASC
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

    let response = GetCarTypesResponse {
        car_type_count: car_types.len(),
        car_types,
    };

    Ok(Json(response))
}

pub async fn get_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let car_type = sqlx::query_as::<Postgres, CarTypeWithCount>(
        r#"
            SELECT
                car_types.car_type_id,
                car_types.name,
                COUNT(cars.car_id) as car_count
            FROM car_types
            LEFT JOIN cars ON car_types.car_type_id = cars.car_type_id
            WHERE car_types.car_type_id = $1 AND car_types.deleted_at IS NULL
            GROUP BY car_types.car_type_id, car_types.name
        "#,
    )
    .bind(car_type_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(car_type))
}

pub async fn create_car_type(
    State(state): State<Arc<AppState>>,
    Json(car_type): Json<CarTypeBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let created_car_type = sqlx::query_as::<Postgres, CarType>(
        r#"
            INSERT INTO car_types (name)
            VALUES ($1)
            RETURNING car_type_id, name
        "#,
    )
    .bind(car_type.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(created_car_type))
}

pub async fn update_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
    Json(car_type): Json<CarTypeBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated_car_type = sqlx::query_as::<Postgres, CarType>(
        r#"
            UPDATE car_types
            SET name = $2
            WHERE car_type_id = $1
            RETURNING car_type_id, name
        "#,
    )
    .bind(car_type_id)
    .bind(car_type.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(updated_car_type))
}

pub async fn delete_car_type(
    State(state): State<Arc<AppState>>,
    Path(car_type_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let deleted_car_type = sqlx::query_as::<Postgres, CarType>(
        r#"
            UPDATE car_types
            SET deleted_at = NOW()
            WHERE car_type_id = $1
            RETURNING car_type_id, name
        "#,
    )
    .bind(car_type_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(deleted_car_type))
}

pub async fn export_car_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let car_types = sqlx::query_as::<Postgres, CarTypeExportDetails>(
        r#"
            SELECT
                car_types.car_type_id,
                car_types.name,
                COUNT(cars.car_id) as car_count
                car_types.created_at,
                car_types.updated_at
                car_types.deleted_at
            FROM car_types
            LEFT JOIN cars ON car_types.car_type_id = cars.car_type_id
            WHERE car_types.deleted_at IS NULL
            GROUP BY car_types.car_type_id, car_types.name
            ORDER BY car_types.car_type_id ASC
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

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer
            .write_record(&[
                "Car Type ID",
                "Name",
                "Count",
                "Created At",
                "Updated At",
                "Deleted At",
            ])
            .map_err(|e| {
                eprintln!("CSV write error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("CSV error: {}", e),
                )
            })?;

        for car_type in car_types {
            writer
                .write_record(&[
                    car_type.car_type_id.to_string(),
                    car_type.name,
                    car_type.car_count.to_string(),
                    car_type.created_at.to_string(),
                    car_type.updated_at.to_string(),
                    car_type
                        .deleted_at
                        .map(|date| date.to_string())
                        .unwrap_or_default(),
                ])
                .map_err(|e| {
                    eprintln!("CSV write error: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("CSV error: {}", e),
                    )
                })?;
        }

        writer.flush().map_err(|e| {
            eprintln!("CSV flush error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("CSV error: {}", e),
            )
        })?;
    }

    Ok((
        [
            ("Content-Type", "text/csv"),
            (
                "Content-Disposition",
                "attachment; filename=\"car-type.csv\"",
            ),
        ],
        csv_buffer,
    ))
}
