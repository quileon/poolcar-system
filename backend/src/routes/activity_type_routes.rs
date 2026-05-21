use crate::middleware::{require_admin, require_employee};
use crate::{
    error::AppError,
    models::activity_type::{ActivityTypeBody, ActivityTypeDetails, GetActivityTypesResponse},
    types::{PaginationParams, SuccessResponse},
    AppState,
};
use axum::middleware::from_fn;
use axum::routing::post;
use axum::{
    extract::{Path, Query, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE}
    ,
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

    let activity_types: Vec<ActivityTypeDetails> = sqlx::query_as(
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
                    WHEN ? = 'active' THEN activity_types.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN activity_types.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE activity_types.deleted_at IS NULL
                END
            GROUP BY activity_types.activity_type_id, activity_types.name
            ORDER BY activity_types.activity_type_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
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
    let activity_type: ActivityTypeDetails = sqlx::query_as(
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
            WHERE activity_types.activity_type_id = ?
            GROUP BY activity_types.activity_type_id, activity_types.name
            ORDER BY activity_types.activity_type_id ASC
        "#,
    )
    .bind(activity_type_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(activity_type))
}

pub async fn create_activity_type(
    State(state): State<Arc<AppState>>,
    Json(activity_type): Json<ActivityTypeBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO activity_types (name)
            VALUES (?)
        "#,
    )
    .bind(&activity_type.name)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Activity type created successfully",
    )))
}

pub async fn update_activity_type(
    State(state): State<Arc<AppState>>,
    Path(activity_type_id): Path<i32>,
    Json(activity_type): Json<ActivityTypeBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE activity_types
            SET name = ?
            WHERE activity_type_id = ?
        "#,
    )
    .bind(&activity_type.name)
    .bind(activity_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Activity type updated successfully",
    )))
}

pub async fn delete_activity_type(
    State(state): State<Arc<AppState>>,
    Path(activity_type_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE activity_types
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE activity_type_id = ?
        "#,
    )
    .bind(activity_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Activity type deleted successfully",
    )))
}

pub async fn restore_activity_type(
    State(state): State<Arc<AppState>>,
    Path(activity_type_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE activity_types
            SET deleted_at = NULL
            WHERE activity_type_id = ?
        "#,
    )
    .bind(activity_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Activity type restored successfully",
    )))
}

pub async fn export_activities(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let activity_types: Vec<ActivityTypeDetails> = sqlx::query_as(
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
    let employee_routes = Router::new()
        .route("/", get(get_activity_types))
        .route("/{activity_id}", get(get_activity_type))
        .route("/export", get(export_activities))
        .route_layer(from_fn(require_employee));

    let admin_routes = Router::new()
        .route("/", post(create_activity_type))
        .route(
            "/{activity_id}",
            put(update_activity_type).delete(delete_activity_type),
        )
        .route("/{activity_id}/restore", put(restore_activity_type))
        .route_layer(from_fn(require_admin));

    Router::new().merge(employee_routes).merge(admin_routes)
}
