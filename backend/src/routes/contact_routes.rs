use crate::{
    error::AppError,
    models::contact::{Contact, ContactBody, ContactDetails, GetContactsResponse},
    routes::contact_type_routes,
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

pub async fn get_contacts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let contacts = sqlx::query_as!(
        ContactDetails,
        r#"
            SELECT
                contacts.contact_id,
                contacts.name,
                contacts.latitude,
                contacts.longitude,
                contact_types.contact_type_id,
                contact_types.name as contact_type_name,
                contacts.created_at,
                contacts.updated_at,
                contacts.deleted_at
            FROM contacts
            LEFT JOIN contact_types ON contacts.contact_type_id = contact_types.contact_type_id
            WHERE
                CASE
                    WHEN $1 = 'active' THEN contacts.deleted_at IS NULL
                    WHEN $1 = 'deleted' THEN contacts.deleted_at IS NOT NULL
                    WHEN $1 = 'all' THEN TRUE
                    ELSE contacts.deleted_at IS NULL
                END
            ORDER BY contacts.contact_id ASC
        "#,
        status
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetContactsResponse {
        contact_count: contacts.len(),
        contacts,
    };

    Ok(Json(response))
}

pub async fn get_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let contact = sqlx::query_as!(
        ContactDetails,
        r#"
            SELECT
                contacts.contact_id,
                contacts.name,
                contacts.latitude,
                contacts.longitude,
                contact_types.contact_type_id,
                contact_types.name as contact_type_name,
                contacts.created_at,
                contacts.updated_at,
                contacts.deleted_at
            FROM contacts
            LEFT JOIN contact_types ON contacts.contact_type_id = contact_types.contact_type_id
            AND contacts.contact_id = $1
            ORDER BY contacts.contact_id ASC
        "#,
        contact_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(contact))
}

pub async fn create_contact(
    State(state): State<Arc<AppState>>,
    Json(contact): Json<ContactBody>,
) -> Result<impl IntoResponse, AppError> {
    let created_contact = sqlx::query_as!(
        Contact,
        r#"
            INSERT INTO contacts (name, latitude, longitude, contact_type_id)
            VALUES ($1, $2, $3, $4)
            RETURNING contact_id, name, latitude, longitude, contact_type_id, created_at, updated_at, deleted_at
        "#,
        contact.name,
        contact.latitude,
        contact.longitude,
        contact.contact_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_contact))
}

pub async fn update_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
    Json(contact): Json<ContactBody>,
) -> Result<impl IntoResponse, AppError> {
    let updated_contact = sqlx::query_as!(
        Contact,
        r#"
            UPDATE contacts
            SET name = $2, latitude = $3, longitude = $4, contact_type_id = $5
            WHERE contact_id = $1
            RETURNING contact_id, name, latitude, longitude, contact_type_id, created_at, updated_at, deleted_at
        "#,
        contact_id,
        contact.name,
        contact.latitude,
        contact.longitude,
        contact.contact_type_id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_contact))
}

pub async fn delete_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let deleted_contact = sqlx::query_as!(
        Contact,
        r#"
            UPDATE contacts
            SET deleted_at = NOW()
            WHERE contact_id = $1
            RETURNING contact_id, name, latitude, longitude, contact_type_id, created_at, updated_at, deleted_at
        "#,
        contact_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(deleted_contact))
}

pub async fn restore_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let restored_contact = sqlx::query_as!(
        Contact,
        r#"
            UPDATE contacts
            SET deleted_at = NULL
            WHERE contact_id = $1
            RETURNING contact_id, name, latitude, longitude, contact_type_id, created_at, updated_at, deleted_at
        "#,
        contact_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(restored_contact))
}

pub async fn export_contacts(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let contacts = sqlx::query_as!(
        ContactDetails,
        r#"
            SELECT
                contacts.contact_id,
                contacts.name,
                contacts.latitude,
                contacts.longitude,
                contact_types.contact_type_id,
                contact_types.name as contact_type_name,
                contact_types.created_at,
                contact_types.updated_at,
                contact_types.deleted_at
            FROM contacts
            LEFT JOIN contact_types ON contacts.contact_type_id = contact_types.contact_type_id
            WHERE contacts.deleted_at IS NULL
            ORDER BY contacts.contact_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Contact ID",
            "Name",
            "Latitude",
            "Longitude",
            "Contact Type ID",
            "Contact Type Name",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for contact in contacts {
            writer.serialize(contact)?;
        }
        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (CONTENT_DISPOSITION, "attachment; filename=\"contacts.csv\""),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_contacts).post(create_contact))
        .route("/export", get(export_contacts))
        .nest("/types", contact_type_routes::routes())
        .route(
            "/{contact_id}",
            get(get_contact).put(update_contact).delete(delete_contact),
        )
        .route("/{contact_id}/restore", put(restore_contact))
}
