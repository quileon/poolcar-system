use axum::extract::Request;
use axum::http::header::SET_COOKIE;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use std::sync::Arc;

use crate::auth_utils::{decode_jwt, extract_token};
use crate::types::{SuccessDataResponse, SuccessResponse};
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
    let user: UserAuth = sqlx::query_as(
        r#"
            SELECT
                users.username,
                users.password,
                users.user_role_id,
                user_roles.name AS user_role_name
            FROM users
            LEFT JOIN user_roles ON users.user_role_id = user_roles.user_role_id
            WHERE users.username = ?
            AND users.deleted_at IS NULL
        "#,
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::WrongCredentials)?;

    if !verify_password(&payload.password, &user.password) {
        return Err(AppError::WrongCredentials);
    }

    let expiration = (Utc::now() + Duration::hours(1)).timestamp() as usize;

    let claims = Claims {
        username: payload.username.clone(),
        role_name: user.user_role_name.clone(),
        exp: expiration,
    };

    let token = encode_jwt(claims, &state.config.jwt_secret)?;

    let cookie = format!(
        "auth_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600",
        token
    );

    Ok((
        [(SET_COOKIE, cookie)],
        Json(SuccessDataResponse::new(LoginResponse {
            username: payload.username,
            role: user.user_role_name,
            token,
        })?),
    ))
}

pub async fn logout_handler() -> Result<impl IntoResponse, AppError> {
    let cookie = "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string();

    Ok((
        [(SET_COOKIE, cookie)],
        Json(SuccessResponse::new("Logged out successfully")),
    ))
}

pub async fn verify_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Json<SuccessDataResponse>, AppError> {
    let token = extract_token(req.headers()).ok_or(AppError::InvalidToken)?;

    let token_data = decode_jwt(&token, &state.config.jwt_secret)?;

    Ok(Json(SuccessDataResponse::new(LoginResponse {
        username: token_data.claims.username,
        role: token_data.claims.role_name,
        token,
    })?))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/verify", get(verify_handler))
}
