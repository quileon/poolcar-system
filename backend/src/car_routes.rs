use crate::{
    models::car::{Car, CarBody, CarExportDetails, CarWithTracker, GetCarsResponse},
    types::PaginationParams,
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
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id,
                trackers.name as tracker_name
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            WHERE cars.deleted_at IS NULL
            ORDER BY cars.car_id ASC
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

pub async fn get_car(
    State(state): State<Arc<AppState>>,
    Path(car_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let car = sqlx::query_as::<Postgres, CarWithTracker>(
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id,
                trackers.name as tracker_name
            FROM cars
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            LEFT JOIN trackers ON cars.tracker_id = trackers.tracker_id
            WHERE cars.car_id = $1 AND cars.deleted_at IS NULL
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

    Ok(axum::Json(car))
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
            SET deleted_at = NOW(), tracker_id = NULL
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

pub async fn export_cars(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cars = sqlx::query_as::<Postgres, CarExportDetails>(
        r#"
            SELECT
                cars.car_id,
                cars.name,
                cars.police_number,
                cars.active,
                car_types.car_type_id,
                car_types.name as car_type_name,
                trackers.tracker_id,
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
            ])
            .map_err(|e| {
                eprintln!("CSV write error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("CSV error: {}", e),
                )
            })?;

        for car in cars {
            writer
                .write_record(&[
                    car.car_id.to_string(),
                    car.name,
                    car.police_number,
                    car.active.to_string(),
                    car.car_type_id.to_string(),
                    car.car_type_name,
                    car.tracker_id.map(|id| id.to_string()).unwrap_or_default(),
                    car.tracker_name.unwrap_or_default(),
                    car.created_at.to_string(),
                    car.updated_at.to_string(),
                    car.deleted_at
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
            ("Content-Disposition", "attachment; filename=\"cars.csv\""),
        ],
        csv_buffer,
    ))
}
