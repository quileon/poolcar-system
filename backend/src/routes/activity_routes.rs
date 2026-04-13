use crate::{
    error::AppError,
    models::{
        activity::{Activity, ActivityBody, ActivityDetails, GetActivitiesResponse},
        contact::Contact,
        websocket::{DeleteActivity, UpdateActivity},
    },
    redis::reload_redis_activities,
    routes::activity_type_routes,
    types::{PaginationParams, SuccessResponse},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use std::sync::Arc;

pub async fn get_activities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<GetActivitiesResponse>, AppError> {
    let status = params.status.as_deref().unwrap_or("active");

    let mut query = sqlx::QueryBuilder::new(
        r#"
            SELECT
                activities.activity_id,
                activities.car_id,
                cars.name AS car_name,
                cars.police_number AS car_police_number,
                activities.contact_id,
                contacts.latitude AS contact_latitude,
                contacts.longitude AS contact_longitude,
                contacts.name AS contact_name,
                activities.activity_type_id,
                activity_types.name AS activity_type_name,
                activities.tracker_id,
                trackers.name AS tracker_name,
                activities.started_at,
                activities.finished_at,
                activities.finished_latitude,
                activities.finished_longitude,
                activities.description,
                activities.created_at,
                activities.updated_at,
                activities.deleted_at
            FROM activities
            LEFT JOIN cars ON cars.car_id = activities.car_id
            LEFT JOIN contacts ON contacts.contact_id = activities.contact_id
            LEFT JOIN activity_types ON activity_types.activity_type_id = activities.activity_type_id
            LEFT JOIN trackers ON trackers.tracker_id = activities.tracker_id
            WHERE
        "#,
    );

    match status {
        "active" => query.push("activities.deleted_at IS NULL"),
        "deleted" => query.push("activities.deleted_at IS NOT NULL"),
        "all" => query.push("TRUE"),
        _ => query.push("activities.deleted_at IS NULL"),
    };

    if let Some(start) = params.start_date {
        query.push(" AND DATE(activities.started_at) >= ");
        query.push_bind(start);
    }

    query.push(" ORDER BY activities.activity_id ASC");

    let activities: Vec<ActivityDetails> = query.build_query_as().fetch_all(&state.db).await?;

    let response = GetActivitiesResponse {
        activity_count: activities.len(),
        activities,
    };

    Ok(Json(response))
}

pub async fn get_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
) -> Result<Json<ActivityDetails>, AppError> {
    let activity: ActivityDetails = sqlx::query_as(
        r#"
            SELECT
                activities.activity_id,
                activities.car_id,
                cars.name AS car_name,
                cars.police_number AS car_police_number,
                activities.contact_id,
                contacts.name AS contact_name,
                contacts.latitude AS contact_latitude,
                contacts.longitude AS contact_longitude,
                activities.activity_type_id,
                activity_types.name AS activity_type_name,
                activities.tracker_id,
                trackers.name AS tracker_name,
                activities.started_at,
                activities.finished_at,
                activities.finished_latitude,
                activities.finished_longitude,
                activities.description,
                activities.created_at,
                activities.updated_at,
                activities.deleted_at
            FROM activities
            LEFT JOIN cars ON cars.car_id = activities.car_id
            LEFT JOIN contacts ON contacts.contact_id = activities.contact_id
            LEFT JOIN activity_types ON activity_types.activity_type_id = activities.activity_type_id
            LEFT JOIN trackers ON trackers.tracker_id = activities.tracker_id
            WHERE activities.activity_id = ?
        "#
    )
    .bind(activity_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(activity))
}

pub async fn create_activity(
    State(state): State<Arc<AppState>>,
    Json(activity): Json<ActivityBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO activities (car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(activity.car_id)
    .bind(activity.contact_id)
    .bind(activity.activity_type_id)
    .bind(activity.tracker_id)
    .bind(activity.finished_at)
    .bind(activity.started_at)
    .bind(activity.finished_latitude)
    .bind(activity.finished_longitude)
    .bind(&activity.description)
    .execute(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    if activity.finished_at.is_none() {
        let created_activity: Activity = sqlx::query_as(
            "SELECT activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at FROM activities ORDER BY activity_id DESC LIMIT 1"
        )
        .fetch_one(&state.db)
        .await?;

        let contact: Contact = sqlx::query_as(
            r#"
                SELECT
                    contact_id,
                    name,
                    latitude,
                    longitude,
                    contact_type_id,
                    created_at,
                    updated_at,
                    deleted_at
                FROM contacts
                WHERE contact_id = ?
            "#,
        )
        .bind(activity.contact_id)
        .fetch_one(&state.db)
        .await?;

        let ws_message = serde_json::json!({
            "message_type": "update_destination",
            "data": UpdateActivity {
                activity_id: created_activity.activity_id as u8,
                contact_name: contact.name,
                contact_latitude: contact.latitude,
                contact_longitude: contact.longitude,
            }
        });
        let new_marker = serde_json::to_string(&ws_message)?;

        match state.tx.send(new_marker) {
            Ok(_) => tracing::debug!("New activity is broadcasted to WebSockets"),
            Err(e) => tracing::warn!("Failed to broadcast new activity to WebSockets: {}", e),
        }
    }

    Ok(Json(SuccessResponse::new("Activity created successfully")))
}

pub async fn update_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
    Json(activity): Json<ActivityBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE activities
            SET car_id = ?, contact_id = ?, activity_type_id = ?, tracker_id = ?, finished_at = ?, started_at = ?, finished_latitude = ?, finished_longitude = ?, description = ?
            WHERE activity_id = ?
        "#
    )
    .bind(activity.car_id)
    .bind(activity.contact_id)
    .bind(activity.activity_type_id)
    .bind(activity.tracker_id)
    .bind(activity.finished_at)
    .bind(activity.started_at)
    .bind(activity.finished_latitude)
    .bind(activity.finished_longitude)
    .bind(&activity.description)
    .bind(activity_id)
    .execute(&state.db)
    .await?;

    let updated_activity: Activity = sqlx::query_as(
        "SELECT activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at FROM activities WHERE activity_id = ?"
    )
    .bind(activity_id)
    .fetch_one(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    let updated_marker = if updated_activity.finished_at.is_some() {
        let ws_message = serde_json::json!({
            "message_type": "remove_destination",
            "data": DeleteActivity {
                activity_id: activity_id as u8,
            }
        });
        serde_json::to_string(&ws_message)?
    } else {
        let contact: Contact = sqlx::query_as(
            r#"
            SELECT
                contact_id,
                name,
                latitude,
                longitude,
                contact_type_id,
                created_at,
                updated_at,
                deleted_at
            FROM contacts
            WHERE contact_id = ?
        "#,
        )
        .bind(activity.contact_id)
        .fetch_one(&state.db)
        .await?;

        let ws_message = serde_json::json!({
            "message_type": "update_destination",
            "data": UpdateActivity {
                activity_id: activity_id as u8,
                contact_name: contact.name,
                contact_latitude: contact.latitude,
                contact_longitude: contact.longitude,
            }
        });
        serde_json::to_string(&ws_message)?
    };

    match state.tx.send(updated_marker) {
        Ok(_) => tracing::debug!("Updated activity is broadcasted to WebSockets"),
        Err(e) => tracing::warn!("Failed to broadcast updated activity to WebSockets: {}", e),
    }

    Ok(Json(SuccessResponse::new("Activity updated successfully")))
}

pub async fn delete_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE activities
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE activity_id = ?
        "#,
    )
    .bind(activity_id)
    .execute(&state.db)
    .await?;

    let deleted_activity: Activity = sqlx::query_as(
        "SELECT activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at FROM activities WHERE activity_id = ?"
    )
    .bind(activity_id)
    .fetch_one(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    if deleted_activity.finished_at.is_none() {
        let ws_message = serde_json::json!({
            "message_type": "remove_destination",
            "data": DeleteActivity {
                activity_id: activity_id as u8,
            }
        });
        let deleted_marker = serde_json::to_string(&ws_message)?;

        match state.tx.send(deleted_marker) {
            Ok(_) => tracing::debug!("Deleted activity is broadcasted to WebSockets"),
            Err(e) => tracing::warn!("Failed to broadcast deleted activity to WebSockets: {}", e),
        }
    }

    Ok(Json(SuccessResponse::new("Activity deleted successfully")))
}

pub async fn restore_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE activities
            SET deleted_at = NULL
            WHERE activity_id = ?
        "#,
    )
    .bind(activity_id)
    .execute(&state.db)
    .await?;

    let restored_activity: Activity = sqlx::query_as(
        "SELECT activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at FROM activities WHERE activity_id = ?"
    )
    .bind(activity_id)
    .fetch_one(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    if restored_activity.finished_at.is_none() {
        let contact: Contact = sqlx::query_as(
            r#"
                SELECT
                    c.contact_id,
                    c.name,
                    c.latitude,
                    c.longitude,
                    c.contact_type_id,
                    c.created_at,
                    c.updated_at,
                    c.deleted_at
                FROM contacts c
                JOIN activities a ON a.contact_id = c.contact_id
                WHERE a.activity_id = ?
            "#,
        )
        .bind(activity_id)
        .fetch_one(&state.db)
        .await?;

        let ws_message = serde_json::json!({
            "message_type": "update_destination",
            "data": UpdateActivity {
                activity_id: activity_id as u8,
                contact_name: contact.name,
                contact_latitude: contact.latitude,
                contact_longitude: contact.longitude,
            }
        });
        let restored_marker = serde_json::to_string(&ws_message)?;

        match state.tx.send(restored_marker) {
            Ok(_) => tracing::debug!("Restored activity is broadcasted to WebSockets"),
            Err(e) => tracing::warn!("Failed to broadcast restored activity to WebSockets: {}", e),
        }
    }

    Ok(Json(SuccessResponse::new("Activity restored successfully")))
}

pub async fn export_activities(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let activities: Vec<ActivityDetails> = sqlx::query_as(
        r#"
            SELECT
                activities.activity_id,
                activities.car_id,
                cars.name AS car_name,
                cars.police_number AS car_police_number,
                activities.contact_id,
                contacts.name AS contact_name,
                contacts.latitude AS contact_latitude,
                contacts.longitude AS contact_longitude,
                activities.activity_type_id,
                activity_types.name AS activity_type_name,
                activities.tracker_id,
                trackers.name AS tracker_name,
                activities.started_at,
                activities.finished_at,
                activities.finished_latitude,
                activities.finished_longitude,
                activities.description,
                activities.created_at,
                activities.updated_at,
                activities.deleted_at
            FROM activities
            LEFT JOIN cars ON cars.car_id = activities.car_id
            LEFT JOIN contacts ON contacts.contact_id = activities.contact_id
            LEFT JOIN activity_types ON activity_types.activity_type_id = activities.activity_type_id
            LEFT JOIN trackers ON trackers.tracker_id = activities.tracker_id
            ORDER BY activities.activity_id ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Activity ID",
            "Car ID",
            "Car Name",
            "Car Police Number",
            "Contact ID",
            "Contact Name",
            "Activity Type ID",
            "Activity Type Name",
            "Tracker ID",
            "Tracker Name",
            "Started At",
            "Finished At",
            "Finished Latitude",
            "Finished Longitude",
            "Description",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for activity in activities {
            writer.serialize(activity)?;
        }
        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (
                CONTENT_DISPOSITION,
                "attachment; filename=\"activities.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_activities).post(create_activity))
        .route("/export", get(export_activities))
        .nest("/types", activity_type_routes::routes())
        .route(
            "/{activity_id}",
            get(get_activity)
                .put(update_activity)
                .delete(delete_activity),
        )
        .route("/{activity_id}/restore", put(restore_activity))
}
