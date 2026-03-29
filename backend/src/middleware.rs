use crate::{auth_utils::decode_jwt, error::AppError, AppState};
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
    let token = req
        .headers()
        .get("cookie")
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let cookie = cookie.trim();
                if cookie.starts_with("auth_token=") {
                    cookie.strip_prefix("auth_token=")
                } else {
                    None
                }
            })
        })
        .ok_or(AppError::InvalidToken)?;

    let token_data = decode_jwt(token, &state.config.jwt_secret)?;

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
