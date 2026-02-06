use axum::{extract::State, response::IntoResponse, Json};
use chrono::{Duration, Utc};
use std::sync::Arc;

use crate::{
    auth_utils::{encode_jwt, verify_password},
    error::AppError,
    models::{
        login::{LoginRequest, LoginResponse},
        user::UserAuth,
    },
    types::Claims,
    AppState,
};

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        UserAuth,
        r#"
            SELECT
                users.username,
                users.password,
                users.user_role_id,
                user_roles.name AS user_role_name
            FROM users
            LEFT JOIN user_roles ON users.user_role_id = user_roles.user_role_id
            WHERE users.username = $1
            AND users.deleted_at IS NULL
        "#,
        payload.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::WrongCredentials)?;

    if !verify_password(&payload.password, &user.password) {
        return Err(AppError::WrongCredentials);
    }

    let expiration = (Utc::now() + Duration::hours(1)).timestamp() as usize;

    let claims = Claims {
        username: payload.username,
        role_name: user.user_role_name,
        exp: expiration,
    };

    let token = encode_jwt(claims, &state.config.jwt_secret)?;

    Ok(Json(LoginResponse { token }))
}
