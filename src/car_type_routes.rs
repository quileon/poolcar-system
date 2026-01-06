use crate::{
    models::{CarType, PaginationParams},
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

#[derive(Debug, FromRow, Serialize)]
struct CarTypeWithCount {
    car_type_id: i32,
    name: String,
    car_count: i64,
}

#[derive(Debug, Serialize)]
struct GetCarTypesResponse {
    car_types: Vec<CarTypeWithCount>,
    car_type_count: usize,
}

#[derive(Debug, FromRow, Deserialize)]
pub struct CarTypeBody {
    pub name: String,
}

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
