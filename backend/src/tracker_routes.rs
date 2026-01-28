use crate::models::{
    GetTrackerResponse, PaginationParams, Tracker, TrackerBody, TrackerExportDetails,
    TrackerWithDetails,
};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::Postgres;
use std::sync::Arc;

pub async fn get_trackers(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let trackers = sqlx::query_as::<Postgres, TrackerWithDetails>(
        r#"
            SELECT
                trackers.tracker_id,
                trackers.name,
                cars.car_id as car_id,
                cars.name as car_name,
                cars.police_number as car_police_number,
                cars.car_type_id as car_type_id,
                car_types.name as car_type_name
            FROM trackers
            LEFT JOIN cars ON trackers.tracker_id = cars.tracker_id
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            WHERE trackers.deleted_at IS NULL
            ORDER BY trackers.tracker_id ASC
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

    let response = GetTrackerResponse {
        tracker_count: trackers.len(),
        trackers,
    };

    Ok(axum::Json(response))
}

pub async fn get_tracker(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let tracker = sqlx::query_as::<Postgres, TrackerWithDetails>(
        r#"
            SELECT
                trackers.tracker_id,
                trackers.name,
                cars.car_id as car_id,
                cars.name as car_name,
                cars.police_number as car_police_number,
                cars.car_type_id as car_type_id,
                car_types.name as car_type_name
            FROM trackers
            LEFT JOIN cars ON trackers.tracker_id = cars.tracker_id
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            WHERE trackers.tracker_id = $1 AND trackers.deleted_at IS NULL
        "#,
    )
    .bind(tracker_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(axum::Json(tracker))
}

pub async fn create_tracker(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TrackerBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let new_tracker = sqlx::query_as::<Postgres, Tracker>(
        r#"
            INSERT INTO trackers (name)
            VALUES ($1)
            RETURNING tracker_id, name
        "#,
    )
    .bind(payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(axum::Json(new_tracker))
}

pub async fn update_tracker(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
    Json(payload): Json<TrackerBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated_tracker = sqlx::query_as::<Postgres, Tracker>(
        r#"
            UPDATE trackers
            SET name = $2
            WHERE tracker_id = $1
            RETURNING tracker_id, name
        "#,
    )
    .bind(tracker_id)
    .bind(payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(axum::Json(updated_tracker))
}

pub async fn delete_tracker(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let deleted_tracker = sqlx::query_as::<Postgres, Tracker>(
        r#"
            UPDATE trackers
            SET deleted_at = NOW()
            WHERE tracker_id = $1
            RETURNING tracker_id, name
        "#,
    )
    .bind(tracker_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(axum::Json(deleted_tracker))
}

pub async fn export_trackers(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let trackers = sqlx::query_as::<Postgres, TrackerExportDetails>(
        r#"
            SELECT
                trackers.tracker_id,
                trackers.name,
                cars.car_id as car_id,
                cars.name as car_name,
                cars.police_number as car_police_number,
                cars.car_type_id as car_type_id,
                car_types.name as car_type_name,
                trackers.created_at,
                trackers.updated_at,
                trackers.deleted_at
            FROM trackers
            LEFT JOIN cars ON trackers.tracker_id = cars.tracker_id
            LEFT JOIN car_types ON cars.car_type_id = car_types.car_type_id
            ORDER BY trackers.tracker_id ASC
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
                "Tracker ID",
                "Name",
                "Car ID",
                "Car Name",
                "Car Police Number",
                "Car Type ID",
                "Car Type Name",
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

        for tracker in trackers {
            writer
                .write_record(&[
                    tracker.tracker_id.to_string(),
                    tracker.name,
                    tracker.car_id.map(|id| id.to_string()).unwrap_or_default(),
                    tracker.car_name.unwrap_or_default(),
                    tracker.car_police_number.unwrap_or_default(),
                    tracker
                        .car_type_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    tracker.car_type_name.unwrap_or_default(),
                    tracker.created_at.to_string(),
                    tracker.updated_at.to_string(),
                    tracker
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
        StatusCode::OK,
        [
            ("Content-Type", "text/csv"),
            (
                "Content-Disposition",
                "attachment; filename=\"trackers.csv\"",
            ),
        ],
        csv_buffer,
    ))
}
