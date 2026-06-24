use crate::middleware::require_employee;
use crate::{
    error::AppError,
    models::audit::{AuditQueryParams, CarAudit, GetAuditResponse},
    state::AppState,
    validate::{is_valid_date, is_within_30_days},
};
use axum::middleware::from_fn;
use axum::{
    extract::{Query, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

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
        let wib_now = chrono::Utc::now() + chrono::Duration::hours(7);
        wib_now.format("%Y-%m-%d").to_string()
    };

    let parsed_date = chrono::NaiveDate::parse_from_str(&target_date, "%Y-%m-%d")
        .map_err(|_| AppError::ValidationError("Invalid date parsing".to_string()))?;

    let start_wib = parsed_date.and_hms_opt(0, 0, 0).unwrap();
    let end_wib = (parsed_date + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let start_utc = start_wib - chrono::Duration::hours(7);
    let end_utc = end_wib - chrono::Duration::hours(7);

    // Build the query based on which filter is provided
    let audit_records: Vec<CarAudit> = if let Some(car_id) = params.car_id {
        // Query by car_id
        sqlx::query_as::<_, CarAudit>(
            r#"
            SELECT
                audit_id,
                car_id,
                tracker_id,
                latitude,
                longitude,
                recorded_at,
                created_at,
                updated_at,
                deleted_at
            FROM audit
            WHERE recorded_at >= ? AND recorded_at < ? AND car_id = ?
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(start_utc)
        .bind(end_utc)
        .bind(car_id)
        .fetch_all(&state.db)
        .await?
    } else if let Some(tracker_id) = params.tracker_id {
        // Query by tracker_id
        sqlx::query_as::<_, CarAudit>(
            r#"
            SELECT
                audit_id,
                car_id,
                tracker_id,
                latitude,
                longitude,
                recorded_at,
                created_at,
                updated_at,
                deleted_at
            FROM audit
            WHERE recorded_at >= ? AND recorded_at < ? AND tracker_id = ?
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(start_utc)
        .bind(end_utc)
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

pub async fn export_audit(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let audit_records: Vec<CarAudit> = sqlx::query_as(
        r#"
        SELECT
            audit_id,
            car_id,
            tracker_id,
            latitude,
            longitude,
            recorded_at,
            created_at,
            updated_at,
            deleted_at
        FROM audit
        WHERE recorded_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) AND deleted_at IS NULL
        ORDER BY recorded_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record([
            "Audit ID",
            "Car ID",
            "Tracker ID",
            "Latitude",
            "Longitude",
            "Recorded At",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for record in audit_records {
            writer.serialize(record)?;
        }

        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (
                CONTENT_DISPOSITION,
                "attachment; filename=\"audit.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    let employee_routes = Router::new()
        .route("/", get(get_audit))
        .route("/export", get(export_audit))
        .route_layer(from_fn(require_employee));

    employee_routes
}
