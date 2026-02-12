use crate::{
    error::AppError,
    models::activity::{
        Activity, ActivityBody, ActivityExport, ActivityWithCount, GetActivitiesResponse,
    },
    types::PaginationParams,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

pub async fn get_activities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let activities = sqlx::query_as!(
        ActivityWithCount,
        r#"
            SELECT
                activities.activity_id,
                activities.name,
                COUNT(histories.history_id) as activity_count
            FROM activities
            LEFT JOIN histories ON histories.activity_id = activities.activity_id
            WHERE activities.deleted_at IS NULL
            GROUP BY activities.activity_id, activities.name
            ORDER BY activities.activity_id ASC
            LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetActivitiesResponse {
        activity_count: activities.len(),
        activities,
    };

    Ok(Json(response))
}

pub async fn get_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let activity = sqlx::query_as!(
        ActivityWithCount,
        r#"
            SELECT
                activities.activity_id,
                activities.name,
                COUNT(histories.history_id) as activity_count
            FROM activities
            LEFT JOIN histories ON histories.activity_id = activities.activity_id
            WHERE activities.activity_id = $1
            GROUP BY activities.activity_id, activities.name
            ORDER BY activities.activity_id ASC
        "#,
        activity_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(activity))
}

pub async fn export_activities(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let activities = sqlx::query_as!(
        ActivityExport,
        r#"
            SELECT
                activities.activity_id,
                activities.name,
                COUNT(histories.history_id) as activity_count,
                activities.created_at,
                activities.updated_at,
                activities.deleted_at
            FROM activities
            LEFT JOIN histories ON histories.activity_id = activities.activity_id
            WHERE activities.deleted_at IS NULL
            GROUP BY activities.activity_id, activities.name
            ORDER BY activities.activity_id ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Activity ID",
            "Name",
            "Activity Count",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for activity in activities {
            writer.write_record(&[
                activity.activity_id.to_string(),
                activity.name,
                activity
                    .activity_count
                    .map(|count| count.to_string())
                    .unwrap_or_default(),
                activity.created_at.to_string(),
                activity.updated_at.to_string(),
                activity
                    .deleted_at
                    .map(|count| count.to_string())
                    .unwrap_or_default(),
            ])?;
        }
        writer.flush()?;
    }

    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "text/csv"),
            (
                "Content-Disposition",
                "attachment; filename=\"activities.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub async fn create_activity(
    State(state): State<Arc<AppState>>,
    Json(activity): Json<ActivityBody>,
) -> Result<impl IntoResponse, AppError> {
    let created_activity = sqlx::query_as!(
        Activity,
        r#"
            INSERT INTO activities (name)
            VALUES ($1)
            RETURNING activity_id, name
        "#,
        activity.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_activity))
}

pub async fn update_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
    Json(activity): Json<ActivityBody>,
) -> Result<impl IntoResponse, AppError> {
    let updated_activity = sqlx::query_as!(
        Activity,
        r#"
            UPDATE activities
            SET name = $2
            WHERE activity_id = $1
            RETURNING activity_id, name
        "#,
        activity_id,
        activity.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_activity))
}

pub async fn delete_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_activity = sqlx::query_as!(
        Activity,
        r#"
            UPDATE activities
            SET deleted_at = NOW()
            WHERE activity_id = $1
            RETURNING activity_id, name
        "#,
        activity_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_activity))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_activities).post(create_activity))
        .route("/export", get(export_activities))
        .route(
            "/{activity_id}",
            get(get_activity)
                .put(update_activity)
                .delete(delete_activity),
        )
}
