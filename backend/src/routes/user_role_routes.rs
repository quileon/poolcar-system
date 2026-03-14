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

    let user_roles: Vec<UserRoleWithDetails> = sqlx::query_as(
        r#"
            SELECT
                user_roles.user_role_id,
                user_roles.name,
                COUNT(users.user_id) as user_count
            FROM user_roles
            LEFT JOIN users ON users.user_role_id = user_roles.user_role_id
            WHERE
                CASE
                    WHEN ? = 'active' THEN user_roles.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN user_roles.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE user_roles.deleted_at IS NULL
                END
            GROUP BY user_roles.user_role_id, user_roles.name
            ORDER BY user_roles.user_role_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
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
    let user_role: UserRoleWithDetails = sqlx::query_as(
        r#"
            SELECT
                user_roles.user_role_id,
                user_roles.name,
                COUNT(users.user_id) as user_count
            FROM user_roles
            LEFT JOIN users ON users.user_role_id = user_roles.user_role_id
            WHERE user_roles.deleted_at IS NULL
            AND user_roles.user_role_id = ?
            GROUP BY user_roles.user_role_id, user_roles.name
            ORDER BY user_roles.user_role_id ASC
        "#,
    )
    .bind(user_role_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user_role))
}

pub async fn create_user_role(
    State(state): State<Arc<AppState>>,
    Json(user_role): Json<UserRoleBody>,
) -> Result<Json<UserRole>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO user_roles (name)
            VALUES (?)
        "#,
    )
    .bind(&user_role.name)
    .execute(&state.db)
    .await?;

    let created_user_role: UserRole = sqlx::query_as(
        "SELECT user_role_id, name FROM user_roles WHERE user_role_id = LAST_INSERT_ID()",
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
    sqlx::query(
        r#"
            UPDATE user_roles
            SET name = ?
            WHERE user_role_id = ?
        "#,
    )
    .bind(&user_role.name)
    .bind(user_role_id)
    .execute(&state.db)
    .await?;

    let updated_user_role: UserRole =
        sqlx::query_as("SELECT user_role_id, name FROM user_roles WHERE user_role_id = ?")
            .bind(user_role_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(updated_user_role))
}

pub async fn delete_user_role(
    State(state): State<Arc<AppState>>,
    Path(user_role_id): Path<i32>,
) -> Result<Json<UserRole>, AppError> {
    sqlx::query(
        r#"
            UPDATE user_roles
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE user_role_id = ?
        "#,
    )
    .bind(user_role_id)
    .execute(&state.db)
    .await?;

    let deleted_user_role: UserRole =
        sqlx::query_as("SELECT user_role_id, name FROM user_roles WHERE user_role_id = ?")
            .bind(user_role_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(deleted_user_role))
}

pub async fn export_user_roles(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let user_roles: Vec<UserRolesExport> = sqlx::query_as(
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
