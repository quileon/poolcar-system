use crate::{
    error::AppError,
    models::contact::{ContactBody, ContactDetails, GetContactsResponse},
    routes::contact_type_routes,
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

pub async fn get_contacts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.unwrap_or("active".into());

    let contacts: Vec<ContactDetails> = sqlx::query_as(
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
                    WHEN ? = 'active' THEN contacts.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN contacts.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE contacts.deleted_at IS NULL
                END
            ORDER BY contacts.contact_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
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
    let contact: ContactDetails = sqlx::query_as(
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
            WHERE contacts.contact_id = ?
            ORDER BY contacts.contact_id ASC
        "#,
    )
    .bind(contact_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(contact))
}

pub async fn create_contact(
    State(state): State<Arc<AppState>>,
    Json(contact): Json<ContactBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO contacts (name, latitude, longitude, contact_type_id)
            VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&contact.name)
    .bind(contact.latitude)
    .bind(contact.longitude)
    .bind(contact.contact_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Contact created successfully")))
}

pub async fn update_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
    Json(contact): Json<ContactBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE contacts
            SET name = ?, latitude = ?, longitude = ?, contact_type_id = ?
            WHERE contact_id = ?
        "#,
    )
    .bind(&contact.name)
    .bind(contact.latitude)
    .bind(contact.longitude)
    .bind(contact.contact_type_id)
    .bind(contact_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Contact updated successfully")))
}

pub async fn delete_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE contacts
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE contact_id = ?
        "#,
    )
    .bind(contact_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Contact deleted successfully")))
}

pub async fn restore_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE contacts
            SET deleted_at = NULL
            WHERE contact_id = ?
        "#,
    )
    .bind(contact_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new("Contact restored successfully")))
}

pub async fn export_contacts(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let contacts: Vec<ContactDetails> = sqlx::query_as(
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
