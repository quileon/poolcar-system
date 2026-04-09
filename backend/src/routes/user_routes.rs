use crate::{
    auth_utils,
    error::AppError,
    models::user::{GetUsersResponse, UserBody, UserDetails},
    routes::user_role_routes,
    types::{PaginationParams, SuccessResponse},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use std::sync::Arc;

pub async fn get_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let users: Vec<UserDetails> = sqlx::query_as(
        r#"
            SELECT
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id,
                user_roles.name AS user_role_name,
                users.created_at,
                users.updated_at,
                users.deleted_at
            FROM users
            LEFT JOIN user_roles ON users.user_role_id = user_roles.user_role_id
            WHERE
                CASE
                    WHEN ? = 'active' THEN users.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN users.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE users.deleted_at IS NULL
                END
            ORDER BY users.user_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
    .fetch_all(&state.db)
    .await?;

    let response = GetUsersResponse {
        user_count: users.len(),
        users,
    };
    Ok(Json(response))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let user: UserDetails = sqlx::query_as(
        r#"
            SELECT
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id,
                user_roles.name AS user_role_name,
                users.created_at,
                users.updated_at,
                users.deleted_at
            FROM users
            LEFT JOIN user_roles ON users.user_role_id = user_roles.user_role_id
            WHERE users.user_id = ?
        "#,
    )
    .bind(tracker_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    let password = payload.password.ok_or(AppError::MissingField)?;
    let hashed_password = auth_utils::hash_password(&password)?;

    sqlx::query(
        r#"
            INSERT INTO users (username, email, password, full_name, user_role_id)
            VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&hashed_password)
    .bind(&payload.full_name)
    .bind(payload.user_role_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("User created successfully")))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Json(payload): Json<UserBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    let hashed_password = match &payload.password {
        Some(pw) => Some(auth_utils::hash_password(pw)?),
        None => None,
    };

    sqlx::query(
        r#"
            UPDATE users
            SET
                username = ?,
                email = ?,
                password = COALESCE(?, password),
                full_name = ?,
                user_role_id = ?
            WHERE user_id = ?
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&hashed_password)
    .bind(&payload.full_name)
    .bind(payload.user_role_id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("User updated successfully")))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE users
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE user_id = ?
        "#,
    )
    .bind(user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("User deleted successfully")))
}

pub async fn restore_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE users
            SET deleted_at = NULL
            WHERE user_id = ?
        "#,
    )
    .bind(user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("User restored successfully")))
}

pub async fn export_users(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let users: Vec<UserDetails> = sqlx::query_as(
        r#"
            SELECT
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id,
                user_roles.name AS user_role_name,
                users.created_at,
                users.updated_at,
                users.deleted_at
            FROM users
            LEFT JOIN user_roles ON users.user_role_id = user_roles.user_role_id
            ORDER BY users.user_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "User ID",
            "Username",
            "Email",
            "Full Name",
            "User Role ID",
            "User Role Name",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for user in users {
            writer.write_record(&[
                user.user_id.to_string(),
                user.username,
                user.email,
                user.full_name,
                user.user_role_id.to_string(),
                user.user_role_name,
                user.created_at.to_string(),
                user.updated_at.to_string(),
                user.deleted_at
                    .map(|date| date.to_string())
                    .unwrap_or_default(),
            ])?;
        }
        writer.flush()?;
    }

    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "text/csv"),
            ("Content-Disposition", "attachment; filename=\"users.csv\""),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_users).post(create_user))
        .route("/export", get(export_users))
        .nest("/roles", user_role_routes::routes())
        .route(
            "/{user_id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route("/{user_id}/restore", put(restore_user))
}
