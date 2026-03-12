use crate::{
    error::AppError,
    models::{
        activity::{Activity, ActivityBody, ActivityDetails, GetActivitiesResponse},
        contact::Contact,
        live_tracking::ActivityMarker,
    },
    redis::reload_redis_activities,
    routes::activity_type_routes,
    types::PaginationParams,
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
    let status = params.status.unwrap_or("active".into());

    let activities = sqlx::query_as!(
        ActivityDetails,
        r#"
            SELECT
                activities.activity_id,
                activities.car_id,
                cars.name AS "car_name?",
                cars.police_number AS "car_police_number?",
                activities.contact_id,
                contacts.latitude AS contact_latitude,
                contacts.longitude AS contact_longitude,
                contacts.name AS contact_name,
                activities.activity_type_id,
                activity_types.name AS activity_type_name,
                activities.tracker_id,
                trackers.name AS "tracker_name?",
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
                CASE
                    WHEN $1 = 'active' THEN activities.deleted_at IS NULL
                    WHEN $1 = 'deleted' THEN activities.deleted_at IS NOT NULL
                    WHEN $1 = 'all' THEN TRUE
                    ELSE activities.deleted_at IS NULL
                END
            ORDER BY activities.activity_id ASC
        "#,
        status,
    )
    .fetch_all(&state.db)
    .await?;

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
    let activity = sqlx::query_as!(
        ActivityDetails,
        r#"
            SELECT
                activities.activity_id,
                activities.car_id,
                cars.name AS "car_name?",
                cars.police_number AS "car_police_number?",
                activities.contact_id,
                contacts.name AS contact_name,
                contacts.latitude AS contact_latitude,
                contacts.longitude AS contact_longitude,
                activities.activity_type_id,
                activity_types.name AS activity_type_name,
                activities.tracker_id,
                trackers.name AS "tracker_name?",
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
            WHERE activities.activity_id = $1
        "#,
        activity_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(activity))
}

pub async fn create_activity(
    State(state): State<Arc<AppState>>,
    Json(activity): Json<ActivityBody>,
) -> Result<Json<Activity>, AppError> {
    let created_activity = sqlx::query_as!(
        Activity,
        r#"
            INSERT INTO activities (car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at
        "#,
        activity.car_id,
        activity.contact_id,
        activity.activity_type_id,
        activity.tracker_id,
        activity.finished_at,
        activity.started_at,
        activity.finished_latitude,
        activity.finished_longitude,
        activity.description
    )
    .fetch_one(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    if activity.finished_at.is_some() {
        return Ok(Json(created_activity));
    }

    let contact = sqlx::query_as!(
        Contact,
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
            WHERE contact_id = $1
        "#,
        activity.contact_id
    )
    .fetch_one(&state.db)
    .await?;

    let new_marker = serde_json::to_string(&ActivityMarker {
        id: created_activity.activity_id as u8,
        action: "POST".into(),
        name: Some(contact.name),
        latitude: Some(contact.latitude),
        longitude: Some(contact.longitude),
    })?;

    match state.tx.send(new_marker) {
        Ok(_) => tracing::debug!("New activity is broadcasted to WebSockets"),
        Err(e) => tracing::warn!("Failed to broadcast new activity to WebSockets: {}", e),
    }

    Ok(Json(created_activity))
}

pub async fn update_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
    Json(activity): Json<ActivityBody>,
) -> Result<Json<Activity>, AppError> {
    let updated_activity = sqlx::query_as!(
        Activity,
        r#"
            UPDATE activities
            SET car_id = $2, contact_id = $3, activity_type_id = $4, tracker_id = $5, finished_at = $6, started_at = $7, finished_latitude = $8, finished_longitude = $9, description = $10
            WHERE activity_id = $1
            RETURNING activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at
        "#,
        activity_id,
        activity.car_id,
        activity.contact_id,
        activity.activity_type_id,
        activity.tracker_id,
        activity.finished_at,
        activity.started_at,
        activity.finished_latitude,
        activity.finished_longitude,
        activity.description
    )
    .fetch_one(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    let updated_marker = if updated_activity.finished_at.is_some() {
        serde_json::to_string(&ActivityMarker {
            id: activity_id as u8,
            action: "DELETE".into(),
            name: None,
            latitude: None,
            longitude: None,
        })?
    } else {
        let contact = sqlx::query_as!(
            Contact,
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
            WHERE contact_id = $1
        "#,
            activity.contact_id
        )
        .fetch_one(&state.db)
        .await?;

        serde_json::to_string(&ActivityMarker {
            id: activity_id as u8,
            action: "PUT".into(),
            name: Some(contact.name),
            latitude: Some(contact.latitude),
            longitude: Some(contact.longitude),
        })?
    };

    match state.tx.send(updated_marker) {
        Ok(_) => tracing::debug!("Updated activity is broadcasted to WebSockets"),
        Err(e) => tracing::warn!("Failed to broadcast updated activity to WebSockets: {}", e),
    }

    Ok(Json(updated_activity))
}

pub async fn delete_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
) -> Result<Json<Activity>, AppError> {
    let deleted_activity = sqlx::query_as!(
        Activity,
        r#"
            UPDATE activities
            SET deleted_at = NOW()
            WHERE activity_id = $1
            RETURNING activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at
        "#,
        activity_id
    )
    .fetch_one(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    if deleted_activity.finished_at.is_some() {
        return Ok(Json(deleted_activity));
    }

    let deleted_marker = serde_json::to_string(&ActivityMarker {
        id: activity_id as u8,
        action: "DELETE".into(),
        name: None,
        latitude: None,
        longitude: None,
    })?;

    match state.tx.send(deleted_marker) {
        Ok(_) => tracing::debug!("Deleted activity is broadcasted to WebSockets"),
        Err(e) => tracing::warn!("Failed to broadcast deleted activity to WebSockets: {}", e),
    }

    Ok(Json(deleted_activity))
}

pub async fn restore_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_id): Path<i32>,
) -> Result<Json<Activity>, AppError> {
    let restored_activity = sqlx::query_as!(
        Activity,
        r#"
            UPDATE activities
            SET deleted_at = NULL
            WHERE activity_id = $1
            RETURNING activity_id, car_id, contact_id, activity_type_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description, created_at, updated_at, deleted_at
        "#,
        activity_id
    )
    .fetch_one(&state.db)
    .await?;

    reload_redis_activities(&state.db, &state.redis)
        .await
        .map_err(|_| AppError::Internal("Success, but failed to reload activities cache".into()))?;

    if restored_activity.finished_at.is_some() {
        return Ok(Json(restored_activity));
    }

    let contact = sqlx::query_as!(
        Contact,
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
            WHERE a.activity_id = $1
        "#,
        activity_id
    )
    .fetch_one(&state.db)
    .await?;

    let restored_marker = serde_json::to_string(&ActivityMarker {
        id: activity_id as u8,
        action: "POST".into(),
        name: Some(contact.name),
        latitude: Some(contact.latitude),
        longitude: Some(contact.longitude),
    })?;

    match state.tx.send(restored_marker) {
        Ok(_) => tracing::debug!("Restored activity is broadcasted to WebSockets"),
        Err(e) => tracing::warn!("Failed to broadcast restored activity to WebSockets: {}", e),
    }

    Ok(Json(restored_activity))
}

pub async fn export_activities(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let activities = sqlx::query_as!(
        ActivityDetails,
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
        "#,
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
