use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    auth_utils,
    error::AppError,
    models::user::{User, UserBody, UserWithDetails},
    types::PaginationParams,
    AppState,
};

pub async fn get_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let users = sqlx::query_as!(
        UserWithDetails,
        r#"
            SELECT
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id,
                user_roles.name AS user_role_name
            FROM users
            LEFT JOIN user_roles ON users.user_role_id = user_roles.user_role_id
            WHERE users.deleted_at IS NULL
            ORDER BY users.user_id ASC
            LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.db)
    .await?;

    Ok(axum::Json(users))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        UserWithDetails,
        r#"
            SELECT
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id,
                user_roles.name AS user_role_name
            FROM users
            LEFT JOIN user_roles ON users.user_role_id = user_roles.user_role_id
            WHERE users.user_id = $1
            AND users.deleted_at IS NULL
        "#,
        tracker_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(axum::Json(user))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserBody>,
) -> Result<impl IntoResponse, AppError> {
    let password = payload.password.ok_or(AppError::MissingField)?;
    let hashed_password = auth_utils::hash_password(&password)?;

    let new_user = sqlx::query_as!(
        User,
        r#"
            INSERT INTO users (username, email, password, full_name, user_role_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id
        "#,
        payload.username,
        payload.email,
        hashed_password,
        payload.full_name,
        payload.user_role_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(new_user))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Json(payload): Json<UserBody>,
) -> Result<impl IntoResponse, AppError> {
    let hashed_password = match &payload.password {
        Some(pw) => Some(auth_utils::hash_password(pw)?),
        None => None,
    };

    let updated_user = sqlx::query_as!(
        User,
        r#"
            UPDATE users
            SET
                username = $2,
                email = $3,
                password = COALESCE($4, password),
                full_name = $5,
                user_role_id = $6
            WHERE user_id = $1
            RETURNING
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id
        "#,
        user_id,
        payload.username,
        payload.email,
        hashed_password,
        payload.full_name,
        payload.user_role_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_user))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_user = sqlx::query_as!(
        User,
        r#"
            UPDATE users
            SET deleted_at = NOW()
            WHERE user_id = $1
            RETURNING
                users.user_id,
                users.username,
                users.email,
                users.full_name,
                users.user_role_id
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_user))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_users).post(create_user))
        .route(
            "/{user_id}",
            get(get_user).put(update_user).delete(delete_user),
        )
}
