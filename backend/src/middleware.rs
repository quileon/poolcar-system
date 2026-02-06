use crate::{auth_utils::decode_jwt, error::AppError, AppState};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::InvalidToken)?;

    let token_data = decode_jwt(token, &state.config.jwt_secret)?;

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
