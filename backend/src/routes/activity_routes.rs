use crate::{
    error::AppError,
    models::activity::{Activity, ActivityBody, ActivityWithCount, GetActivitiesResponse},
    types::PaginationParams,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, put},
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
        .route(
            "/{activity_id}",
            put(update_activity).delete(delete_activity),
        )
}
