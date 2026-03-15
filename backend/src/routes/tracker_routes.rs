use crate::{
    error::AppError,
    models::tracker::{GetTrackerResponse, TrackerBody, TrackerDetails},
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

pub async fn get_trackers(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let trackers: Vec<TrackerDetails> = sqlx::query_as(
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
            WHERE
                CASE
                    WHEN ? = 'active' THEN trackers.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN trackers.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE trackers.deleted_at IS NULL
                END
            ORDER BY trackers.tracker_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
    .fetch_all(&state.db)
    .await?;

    let response = GetTrackerResponse {
        tracker_count: trackers.len(),
        trackers,
    };

    Ok(Json(response))
}

pub async fn get_tracker(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let tracker: TrackerDetails = sqlx::query_as(
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
            WHERE trackers.tracker_id = ?
        "#,
    )
    .bind(tracker_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(tracker))
}

pub async fn create_tracker(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TrackerBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO trackers (name)
            VALUES (?)
        "#,
    )
    .bind(&payload.name)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Tracker created successfully")))
}

pub async fn update_tracker(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
    Json(payload): Json<TrackerBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE trackers
            SET name = ?
            WHERE tracker_id = ?
        "#,
    )
    .bind(&payload.name)
    .bind(tracker_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Tracker updated successfully")))
}

pub async fn delete_tracker(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE trackers
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE tracker_id = ?
        "#,
    )
    .bind(tracker_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Tracker deleted successfully")))
}

pub async fn restore_tracker(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE trackers
            SET deleted_at = NULL
            WHERE tracker_id = ?
        "#,
    )
    .bind(tracker_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Tracker restored successfully")))
}

pub async fn export_trackers(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let trackers: Vec<TrackerDetails> = sqlx::query_as(
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
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
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
        ])?;

        for tracker in trackers {
            writer.serialize(tracker)?;
        }

        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (CONTENT_DISPOSITION, "attachment; filename=\"trackers.csv\""),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_trackers).post(create_tracker))
        .route("/export", get(export_trackers))
        .route(
            "/{tracker_id}",
            get(get_tracker).put(update_tracker).delete(delete_tracker),
        )
        .route("/{tracker_id}/restore", put(restore_tracker))
}
