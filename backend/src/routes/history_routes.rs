use crate::{
    error::AppError,
    models::history::{
        GetHistoriesResponse, History, HistoryBody, HistoryExport, HistoryWithDetails,
    },
    types::PaginationParams,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

pub async fn get_histories(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let histories = sqlx::query_as!(
        HistoryWithDetails,
        r#"
            SELECT
                histories.history_id,
                histories.car_id,
                cars.name AS car_name,
                histories.contact_id,
                contacts.name AS contact_name,
                histories.activity_id,
                activities.name AS activity_name,
                histories.tracker_id,
                trackers.name AS tracker_name,
                histories.finished_at,
                histories.started_at,
                histories.finished_latitude,
                histories.finished_longitude,
                histories.description
            FROM histories
            LEFT JOIN cars ON cars.car_id = histories.car_id
            LEFT JOIN contacts ON contacts.contact_id = histories.contact_id
            LEFT JOIN activities ON activities.activity_id = histories.activity_id
            LEFT JOIN trackers ON trackers.tracker_id = histories.tracker_id
            WHERE histories.deleted_at IS NULL
            ORDER BY histories.history_id ASC
            LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64,
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetHistoriesResponse {
        history_count: histories.len(),
        histories,
    };

    Ok(Json(response))
}

pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Path(history_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let history = sqlx::query_as!(
        HistoryWithDetails,
        r#"
            SELECT
                histories.history_id,
                histories.car_id,
                cars.name AS car_name,
                histories.contact_id,
                contacts.name AS contact_name,
                histories.activity_id,
                activities.name AS activity_name,
                histories.tracker_id,
                trackers.name AS tracker_name,
                histories.finished_at,
                histories.started_at,
                histories.finished_latitude,
                histories.finished_longitude,
                histories.description
            FROM histories
            LEFT JOIN cars ON cars.car_id = histories.car_id
            LEFT JOIN contacts ON contacts.contact_id = histories.contact_id
            LEFT JOIN activities ON activities.activity_id = histories.activity_id
            LEFT JOIN trackers ON trackers.tracker_id = histories.tracker_id
            WHERE histories.history_id = $1
            ORDER BY histories.history_id ASC
        "#,
        history_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(history))
}

pub async fn create_history(
    State(state): State<Arc<AppState>>,
    Json(history): Json<HistoryBody>,
) -> Result<impl IntoResponse, AppError> {
    let created_history = sqlx::query_as!(
        History,
        r#"
            INSERT INTO histories (car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING history_id, car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description
        "#,
        history.car_id,
        history.contact_id,
        history.activity_id,
        history.tracker_id,
        history.finished_at,
        history.started_at,
        history.finished_latitude,
        history.finished_longitude,
        history.description
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_history))
}

pub async fn update_history(
    State(state): State<Arc<AppState>>,
    Path(history_id): Path<i32>,
    Json(history): Json<HistoryBody>,
) -> Result<impl IntoResponse, AppError> {
    let updated_history = sqlx::query_as!(
        History,
        r#"
            UPDATE histories
            SET car_id = $2, contact_id = $3, activity_id = $4, tracker_id = $5, finished_at = $6, started_at = $7, finished_latitude = $8, finished_longitude = $9, description = $10
            WHERE history_id = $1
            RETURNING history_id, car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description
        "#,
        history_id,
        history.car_id,
        history.contact_id,
        history.activity_id,
        history.tracker_id,
        history.finished_at,
        history.started_at,
        history.finished_latitude,
        history.finished_longitude,
        history.description
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_history))
}

pub async fn delete_history(
    State(state): State<Arc<AppState>>,
    Path(history_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_history = sqlx::query_as!(
        History,
        r#"
            UPDATE histories
            SET deleted_at = NOW()
            WHERE history_id = $1
            RETURNING history_id, car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description
        "#,
        history_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_history))
}

pub async fn export_histories(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let histories = sqlx::query_as!(
        HistoryExport,
        r#"
            SELECT
                histories.history_id,
                histories.car_id,
                cars.name AS car_name,
                histories.contact_id,
                contacts.name AS contact_name,
                histories.activity_id,
                activities.name AS activity_name,
                histories.tracker_id,
                trackers.name AS tracker_name,
                histories.finished_at,
                histories.started_at,
                histories.finished_latitude,
                histories.finished_longitude,
                histories.description,
                histories.created_at,
                histories.updated_at,
                histories.deleted_at
            FROM histories
            LEFT JOIN cars ON cars.car_id = histories.car_id
            LEFT JOIN contacts ON contacts.contact_id = histories.contact_id
            LEFT JOIN activities ON activities.activity_id = histories.activity_id
            LEFT JOIN trackers ON trackers.tracker_id = histories.tracker_id
            WHERE histories.deleted_at IS NULL
            ORDER BY histories.history_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "History ID",
            "Car ID",
            "Car Name",
            "Contact ID",
            "Contact Name",
            "Activity ID",
            "Activity Name",
            "Tracker ID",
            "Tracker Name",
            "Started At",
            "Finished At",
            "Finished Latitude",
            "Finished Longitude",
            "Description",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for history in histories {
            writer
                .serialize(history)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        writer.flush()?;
    }

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"histories.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_histories).post(create_history))
        .route("/export", get(export_histories))
        .route(
            "/{history_id}",
            get(get_history).put(update_history).delete(delete_history),
        )
}
