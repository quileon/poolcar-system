use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::models::Car;
use crate::AppState;

pub async fn get_dashboard_data(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cars = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;
    Ok(axum::Json(cars))
}
