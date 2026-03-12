use crate::{
    error::AppError,
    models::user_role::{
        GetUserRolesResponse, UserRole, UserRoleBody, UserRoleWithDetails, UserRolesExport,
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

pub async fn get_user_roles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<GetUserRolesResponse>, AppError> {
    let status = params.status.unwrap_or("active".into());

    let user_roles = sqlx::query_as!(
        UserRoleWithDetails,
        r#"
            SELECT
                user_roles.user_role_id,
                user_roles.name,
                COUNT(users.user_id) as user_count
            FROM user_roles
            LEFT JOIN users ON users.user_role_id = user_roles.user_role_id
            WHERE
                CASE
                    WHEN $1 = 'active' THEN user_roles.deleted_at IS NULL
                    WHEN $1 = 'deleted' THEN user_roles.deleted_at IS NOT NULL
                    WHEN $1 = 'all' THEN TRUE
                    ELSE user_roles.deleted_at IS NULL
                END
            GROUP BY user_roles.user_role_id, user_roles.name
            ORDER BY user_roles.user_role_id ASC
        "#,
        status
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetUserRolesResponse {
        user_role_count: user_roles.len(),
        user_roles,
    };

    Ok(Json(response))
}

pub async fn get_user_role(
    State(state): State<Arc<AppState>>,
    Path(user_role_id): Path<i32>,
) -> Result<Json<UserRoleWithDetails>, AppError> {
    let user_role = sqlx::query_as!(
        UserRoleWithDetails,
        r#"
            SELECT
                user_roles.user_role_id,
                user_roles.name,
                COUNT(users.user_id) as user_count
            FROM user_roles
            LEFT JOIN users ON users.user_role_id = user_roles.user_role_id
            WHERE user_roles.deleted_at IS NULL
            AND user_roles.user_role_id = $1
            GROUP BY user_roles.user_role_id, user_roles.name
            ORDER BY user_roles.user_role_id ASC
        "#,
        user_role_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user_role))
}

pub async fn create_user_role(
    State(state): State<Arc<AppState>>,
    Json(user_role): Json<UserRoleBody>,
) -> Result<Json<UserRole>, AppError> {
    let created_user_role = sqlx::query_as!(
        UserRole,
        r#"
            INSERT INTO user_roles (name)
            VALUES ($1)
            RETURNING user_role_id, name
        "#,
        user_role.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_user_role))
}

pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    Path(user_role_id): Path<i32>,
    Json(user_role): Json<UserRoleBody>,
) -> Result<Json<UserRole>, AppError> {
    let updated_user_role = sqlx::query_as!(
        UserRole,
        r#"
            UPDATE user_roles
            SET name = $2
            WHERE user_role_id = $1
            RETURNING user_role_id, name
        "#,
        user_role_id,
        user_role.name,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_user_role))
}

pub async fn delete_user_role(
    State(state): State<Arc<AppState>>,
    Path(user_role_id): Path<i32>,
) -> Result<Json<UserRole>, AppError> {
    let deleted_user_role = sqlx::query_as!(
        UserRole,
        r#"
            UPDATE user_roles
            SET deleted_at = NOW()
            WHERE user_role_id = $1
            RETURNING user_role_id, name
        "#,
        user_role_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_user_role))
}

pub async fn export_user_roles(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let user_roles = sqlx::query_as!(
        UserRolesExport,
        r#"
            SELECT
                user_roles.user_role_id,
                user_roles.name,
                COUNT(users.user_id) as user_count,
                user_roles.created_at,
                user_roles.updated_at,
                user_roles.deleted_at
            FROM user_roles
            LEFT JOIN users ON users.user_role_id = user_roles.user_role_id
            GROUP BY user_roles.user_role_id, user_roles.name
            ORDER BY user_roles.user_role_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "User Role ID",
            "Name",
            "User Count",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for user_role in user_roles {
            writer
                .serialize(user_role)
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
                "attachment; filename=\"user_roles.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_user_roles).post(create_user_role))
        .route("/export", get(export_user_roles))
        .route(
            "/{user_role_id}",
            get(get_user_role)
                .put(update_user_role)
                .delete(delete_user_role),
        )
}
