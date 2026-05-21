use crate::{
    auth_utils::{decode_jwt, extract_token},
    error::AppError,
    AppState,
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(req.headers()).ok_or(AppError::InvalidToken)?;

    let token_data = decode_jwt(&token, &state.config.jwt_secret)?;

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

pub async fn require_admin(req: Request, next: Next) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<crate::types::Claims>()
        .ok_or(AppError::Unauthorized)?;

    if claims.role_name != "Admin" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}

pub async fn require_security(req: Request, next: Next) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<crate::types::Claims>()
        .ok_or(AppError::Unauthorized)?;

    if claims.role_name != "Admin" && claims.role_name != "Security" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}

pub async fn require_employee(req: Request, next: Next) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<crate::types::Claims>()
        .ok_or(AppError::Unauthorized)?;

    if claims.role_name != "Admin" && claims.role_name != "Employee" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}
