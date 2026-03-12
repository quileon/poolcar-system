use crate::{
    error::AppError,
    models::activity_type::{
        ActivityType, ActivityTypeBody, ActivityTypeDetails, GetActivityTypesResponse,
    },
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

pub async fn get_activity_types(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<GetActivityTypesResponse>, AppError> {
    let status = params.status.unwrap_or("active".into());

    let activity_types = sqlx::query_as!(
        ActivityTypeDetails,
        r#"
            SELECT
                activity_types.activity_type_id,
                activity_types.name,
                COUNT(activities.activity_id) as activity_count,
                activity_types.created_at,
                activity_types.updated_at,
                activity_types.deleted_at
            FROM activity_types
            LEFT JOIN activities ON activity_types.activity_type_id = activities.activity_type_id
            WHERE
                CASE
                    WHEN $1 = 'active' THEN activity_types.deleted_at IS NULL
                    WHEN $1 = 'deleted' THEN activity_types.deleted_at IS NOT NULL
                    WHEN $1 = 'all' THEN TRUE
                    ELSE activity_types.deleted_at IS NULL
                END
            GROUP BY activity_types.activity_type_id, activity_types.name
            ORDER BY activity_types.activity_type_id ASC
        "#,
        status
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetActivityTypesResponse {
        activity_type_count: activity_types.len(),
        activity_types,
    };

    Ok(Json(response))
}

pub async fn get_activity_type(
    State(state): State<Arc<AppState>>,
    Path(activity_type_id): Path<i32>,
) -> Result<Json<ActivityTypeDetails>, AppError> {
    let activity_type = sqlx::query_as!(
        ActivityTypeDetails,
        r#"
            SELECT
                activity_types.activity_type_id,
                activity_types.name,
                COUNT(activities.activity_id) as activity_count,
                activity_types.created_at,
                activity_types.updated_at,
                activity_types.deleted_at
            FROM activity_types
            LEFT JOIN activities ON activity_types.activity_type_id = activities.activity_type_id
            WHERE activity_types.activity_type_id = $1
            GROUP BY activity_types.activity_type_id, activity_types.name
            ORDER BY activity_types.activity_type_id ASC
        "#,
        activity_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(activity_type))
}

pub async fn create_activity_type(
    State(state): State<Arc<AppState>>,
    Json(activity_type): Json<ActivityTypeBody>,
) -> Result<Json<ActivityType>, AppError> {
    let created_activity_type = sqlx::query_as!(
        ActivityType,
        r#"
            INSERT INTO activity_types (name)
            VALUES ($1)
            RETURNING activity_type_id, name, created_at, updated_at, deleted_at
        "#,
        activity_type.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_activity_type))
}

pub async fn update_activity_type(
    State(state): State<Arc<AppState>>,
    Path(activity_type_id): Path<i32>,
    Json(activity_type): Json<ActivityTypeBody>,
) -> Result<Json<ActivityType>, AppError> {
    let updated_activity_type = sqlx::query_as!(
        ActivityType,
        r#"
            UPDATE activity_types
            SET name = $2
            WHERE activity_type_id = $1
            RETURNING activity_type_id, name, created_at, updated_at, deleted_at
        "#,
        activity_type_id,
        activity_type.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_activity_type))
}

pub async fn delete_activity_type(
    State(state): State<Arc<AppState>>,
    Path(activity_type_id): Path<i32>,
) -> Result<Json<ActivityType>, AppError> {
    let deleted_activity_type = sqlx::query_as!(
        ActivityType,
        r#"
            UPDATE activity_types
            SET deleted_at = NOW()
            WHERE activity_type_id = $1
            RETURNING activity_type_id, name, created_at, updated_at, deleted_at
        "#,
        activity_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_activity_type))
}

pub async fn restore_activity_type(
    State(state): State<Arc<AppState>>,
    Path(activity_type_id): Path<i32>,
) -> Result<Json<ActivityType>, AppError> {
    let restore_activity_type = sqlx::query_as!(
        ActivityType,
        r#"
            UPDATE activity_types
            SET deleted_at = NULL
            WHERE activity_type_id = $1
            RETURNING activity_type_id, name, created_at, updated_at, deleted_at
        "#,
        activity_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(restore_activity_type))
}

pub async fn export_activities(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let activity_types = sqlx::query_as!(
        ActivityTypeDetails,
        r#"
            SELECT
                activity_types.activity_type_id,
                activity_types.name,
                COUNT(activities.activity_id) as activity_count,
                activity_types.created_at,
                activity_types.updated_at,
                activity_types.deleted_at
            FROM activity_types
            LEFT JOIN activities ON activity_types.activity_type_id = activities.activity_type_id
            GROUP BY activity_types.activity_type_id, activity_types.name
            ORDER BY activity_types.activity_type_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Activity Type ID",
            "Name",
            "Activity Count",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for activity_type in activity_types {
            writer.serialize(activity_type)?;
        }

        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (
                CONTENT_DISPOSITION,
                "attachment; filename=\"activity_types.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_activity_types).post(create_activity_type))
        .route("/export", get(export_activities))
        .route(
            "/{activity_id}",
            get(get_activity_type)
                .put(update_activity_type)
                .delete(delete_activity_type),
        )
        .route("/{activity_id}/restore", put(restore_activity_type))
}
