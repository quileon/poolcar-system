use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::{
    models::{LoginRequest, UserAuth},
    AppState,
};

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = sqlx::query_as!(
        UserAuth,
        r#"
            SELECT
                users.username,
                users.password,
                users.user_role_id
            FROM users
            WHERE users.username = $1
            AND users.deleted_at IS NULL
        "#,
        payload.username
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(axum::Json(user))
}
