use crate::{
    error::AppError,
    models::contact_type::{
        ContactType, ContactTypeBody, ContactTypeDetails, GetContactTypesResponse,
    },
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

pub async fn get_contact_types(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<GetContactTypesResponse>, AppError> {
    let status = params.status.unwrap_or("active".into());

    let contact_types = sqlx::query_as!(
        ContactTypeDetails,
        r#"
            SELECT
                contact_types.contact_type_id,
                contact_types.name,
                COUNT(contacts.contact_id) as contact_count,
                contact_types.created_at,
                contact_types.updated_at,
                contact_types.deleted_at
            FROM contact_types
            LEFT JOIN contacts ON contact_types.contact_type_id = contacts.contact_type_id
            WHERE
                CASE
                    WHEN $1 = 'active' THEN contact_types.deleted_at IS NULL
                    WHEN $1 = 'deleted' THEN contact_types.deleted_at IS NOT NULL
                    WHEN $1 = 'all' THEN TRUE
                    ELSE contact_types.deleted_at IS NULL
                END
            GROUP BY contact_types.contact_type_id, contact_types.name
            ORDER BY contact_types.contact_type_id ASC
        "#,
        status
    )
    .fetch_all(&state.db)
    .await?;

    let response = GetContactTypesResponse {
        contact_type_count: contact_types.len(),
        contact_types,
    };

    Ok(Json(response))
}

pub async fn get_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
) -> Result<Json<ContactTypeDetails>, AppError> {
    let contact_type = sqlx::query_as!(
        ContactTypeDetails,
        r#"
            SELECT
                contact_types.contact_type_id,
                contact_types.name,
                COUNT(contacts.contact_id) as contact_count,
                contact_types.created_at,
                contact_types.updated_at,
                contact_types.deleted_at
            FROM contact_types
            LEFT JOIN contacts ON contact_types.contact_type_id = contacts.contact_type_id
            WHERE contact_types.contact_type_id = $1
            GROUP BY contact_types.contact_type_id, contact_types.name
            ORDER BY contact_types.contact_type_id ASC
        "#,
        contact_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(contact_type))
}

pub async fn create_contact_type(
    State(state): State<Arc<AppState>>,
    Json(contact_type): Json<ContactTypeBody>,
) -> Result<Json<ContactType>, AppError> {
    let created_contact_type = sqlx::query_as!(
        ContactType,
        r#"
            INSERT INTO contact_types (name)
            VALUES ($1)
            RETURNING contact_type_id, name, created_at, updated_at, deleted_at
        "#,
        contact_type.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(created_contact_type))
}

pub async fn update_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
    Json(contact_type): Json<ContactTypeBody>,
) -> Result<Json<ContactType>, AppError> {
    let updated_contact_type = sqlx::query_as!(
        ContactType,
        r#"
            UPDATE contact_types
            SET name = $2
            WHERE contact_type_id = $1
            RETURNING contact_type_id, name, created_at, updated_at, deleted_at
        "#,
        contact_type_id,
        contact_type.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_contact_type))
}

pub async fn delete_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
) -> Result<Json<ContactType>, AppError> {
    let delete_contact_type = sqlx::query_as!(
        ContactType,
        r#"
            UPDATE contact_types
            SET deleted_at = NOW()
            WHERE contact_type_id = $1
            RETURNING contact_type_id, name, created_at, updated_at, deleted_at
        "#,
        contact_type_id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(delete_contact_type))
}

pub async fn restore_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
) -> Result<Json<ContactType>, AppError> {
    let restored_contact_type = sqlx::query_as!(
        ContactType,
        r#"
            UPDATE contact_types
            SET deleted_at = NULL
            WHERE contact_type_id = $1
            RETURNING contact_type_id, name, created_at, updated_at, deleted_at
        "#,
        contact_type_id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(restored_contact_type))
}

pub async fn export_contact_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let contact_types = sqlx::query_as!(
        ContactTypeDetails,
        r#"
            SELECT
                contact_types.contact_type_id,
                contact_types.name,
                COUNT(contacts.contact_id) as contact_count,
                contact_types.created_at,
                contact_types.updated_at,
                contact_types.deleted_at
            FROM contact_types
            LEFT JOIN contacts ON contact_types.contact_type_id = contacts.contact_type_id
            GROUP BY contact_types.contact_type_id, contact_types.name
            ORDER BY contact_types.contact_type_id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut csv_buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut csv_buffer);
        writer.write_record(&[
            "Contact Type ID",
            "Name",
            "Contact Count",
            "Created At",
            "Updated At",
            "Deleted At",
        ])?;

        for contact_type in contact_types {
            writer.serialize(contact_type)?;
        }

        writer.flush()?;
    }

    Ok((
        [
            (CONTENT_TYPE, "text/csv"),
            (
                CONTENT_DISPOSITION,
                "attachment; filename=\"contact_types.csv\"",
            ),
        ],
        csv_buffer,
    ))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_contact_types).post(create_contact_type))
        .route("/export", get(export_contact_types))
        .route(
            "/{contact_type_id}",
            get(get_contact_type)
                .put(update_contact_type)
                .delete(delete_contact_type),
        )
        .route("/{contact_type_id}/restore", put(restore_contact_type))
}
