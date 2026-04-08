use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::{
    error::AppError,
    models::audit::{AuditQueryParams, CarAudit, GetAuditResponse},
    state::AppState,
};

pub async fn get_audit(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuditQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    // Validate that exactly one of car_id or tracker_id is provided
    match (params.car_id, params.tracker_id) {
        (None, None) => {
            return Err(AppError::ValidationError(
                "Either car_id or tracker_id must be provided".to_string(),
            ));
        }
        (Some(_), Some(_)) => {
            return Err(AppError::ValidationError(
                "Cannot provide both car_id and tracker_id. Choose one".to_string(),
            ));
        }
        _ => {}
    }

    let target_date = if let Some(date_str) = params.date {
        if !is_valid_date(&date_str) {
            return Err(AppError::ValidationError(
                "Invalid date format. Use YYYY-MM-DD".to_string(),
            ));
        }
        if !is_within_30_days(&date_str) {
            return Err(AppError::ValidationError(
                "Date must be within the last 30 days".to_string(),
            ));
        }
        date_str
    } else {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    };

    // Build the query based on which filter is provided
    let audit_records: Vec<CarAudit> = if let Some(car_id) = params.car_id {
        // Query by car_id
        sqlx::query_as::<_, CarAudit>(
            r#"
            SELECT
                car_audit_id,
                car_id,
                tracker_id,
                latitude,
                longitude,
                recorded_at,
                created_at,
                updated_at,
                deleted_at
            FROM car_audit
            WHERE DATE(recorded_at) = ? AND car_id = ?
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(&target_date)
        .bind(car_id)
        .fetch_all(&state.db)
        .await?
    } else if let Some(tracker_id) = params.tracker_id {
        // Query by tracker_id
        sqlx::query_as::<_, CarAudit>(
            r#"
            SELECT
                car_audit_id,
                car_id,
                tracker_id,
                latitude,
                longitude,
                recorded_at,
                created_at,
                updated_at,
                deleted_at
            FROM car_audit
            WHERE DATE(recorded_at) = ? AND tracker_id = ?
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(&target_date)
        .bind(tracker_id)
        .fetch_all(&state.db)
        .await?
    } else {
        vec![]
    };

    let total_count = audit_records.len();

    let response = GetAuditResponse {
        total_count,
        audit_records,
    };

    Ok(Json(response))
}

fn is_valid_date(date_str: &str) -> bool {
    // Check format YYYY-MM-DD
    if date_str.len() != 10 {
        return false;
    }

    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    let year: Result<u32, _> = parts[0].parse();
    let month: Result<u32, _> = parts[1].parse();
    let day: Result<u32, _> = parts[2].parse();

    match (year, month, day) {
        (Ok(y), Ok(m), Ok(d)) => y > 0 && m >= 1 && m <= 12 && d >= 1 && d <= 31,
        _ => false,
    }
}

fn is_within_30_days(date_str: &str) -> bool {
    use chrono::NaiveDate;

    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(target_date) => {
            let today = chrono::Local::now().naive_local().date();
            let duration = today.signed_duration_since(target_date);

            // Allow dates from today going back 30 days
            duration.num_days() >= 0 && duration.num_days() <= 30
        }
        Err(_) => false,
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_audit))
}
