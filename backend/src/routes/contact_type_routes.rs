use crate::{
    error::AppError,
    models::contact_type::{
        ContactType, ContactTypeBody, ContactTypeExport, ContactTypeWithCount,
        GetContactTypesResponse,
    },
    types::PaginationParams,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

pub async fn get_contact_types(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let contact_types = sqlx::query_as!(
        ContactTypeWithCount,
        r#"
            SELECT
                contact_types.contact_type_id,
                contact_types.name,
                COUNT(contacts.contact_id) as contact_count
            FROM contact_types
            LEFT JOIN contacts ON contact_types.contact_type_id = contacts.contact_type_id
            WHERE contact_types.deleted_at IS NULL
            GROUP BY contact_types.contact_type_id, contact_types.name
            ORDER BY contact_types.contact_type_id ASC
            LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
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
) -> Result<impl IntoResponse, AppError> {
    let contact_type = sqlx::query_as!(
        ContactTypeWithCount,
        r#"
            SELECT
                contact_types.contact_type_id,
                contact_types.name,
                COUNT(contacts.contact_id) as contact_count
            FROM contact_types
            LEFT JOIN contacts ON contact_types.contact_type_id = contacts.contact_type_id
            WHERE contact_types.deleted_at IS NULL
            AND contact_types.contact_type_id = $1
            GROUP BY contact_types.contact_type_id, contact_types.name
            ORDER BY contact_types.contact_type_id ASC
        "#,
        contact_type_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(axum::Json(contact_type))
}

pub async fn create_contact_type(
    State(state): State<Arc<AppState>>,
    Json(contact_type): Json<ContactTypeBody>,
) -> Result<impl IntoResponse, AppError> {
    let created_contact_type = sqlx::query_as!(
        ContactType,
        r#"
            INSERT INTO contact_types (name)
            VALUES ($1)
            RETURNING contact_type_id, name
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
) -> Result<impl IntoResponse, AppError> {
    let updated_contact_type = sqlx::query_as!(
        ContactType,
        r#"
            UPDATE contact_types
            SET name = $2
            WHERE contact_type_id = $1
            RETURNING contact_type_id, name
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
) -> Result<impl IntoResponse, AppError> {
    let delete_contact_type = sqlx::query_as!(
        ContactType,
        r#"
            UPDATE contact_types
            SET deleted_at = NOW()
            WHERE contact_type_id = $1
            RETURNING contact_type_id, name
        "#,
        contact_type_id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(delete_contact_type))
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
}

pub async fn export_contact_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let contact_types = sqlx::query_as!(
        ContactTypeExport,
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
            writer.write_record(&[
                contact_type.contact_type_id.to_string(),
                contact_type.name,
                contact_type
                    .contact_count
                    .map(|count| count.to_string())
                    .unwrap_or_default(),
                contact_type.created_at.to_string(),
                contact_type.updated_at.to_string(),
                contact_type
                    .deleted_at
                    .map(|date| date.to_string())
                    .unwrap_or_default(),
            ])?;
        }
        writer.flush()?;
    }

    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "text/csv"),
            (
                "Content-Disposition",
                "attachment; filename=\"contact_types.csv\"",
            ),
        ],
        csv_buffer,
    ))
}
