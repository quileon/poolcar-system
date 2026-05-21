use crate::middleware::{require_admin, require_employee};
use crate::{
    error::AppError,
    models::contact_type::{ContactTypeBody, ContactTypeDetails, GetContactTypesResponse},
    types::{PaginationParams, SuccessResponse},
    AppState,
};
use axum::middleware::from_fn;
use axum::routing::post;
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

    let contact_types: Vec<ContactTypeDetails> = sqlx::query_as(
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
                    WHEN ? = 'active' THEN contact_types.deleted_at IS NULL
                    WHEN ? = 'deleted' THEN contact_types.deleted_at IS NOT NULL
                    WHEN ? = 'all' THEN TRUE
                    ELSE contact_types.deleted_at IS NULL
                END
            GROUP BY contact_types.contact_type_id, contact_types.name
            ORDER BY contact_types.contact_type_id ASC
        "#,
    )
    .bind(&status)
    .bind(&status)
    .bind(&status)
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
    let contact_type: ContactTypeDetails = sqlx::query_as(
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
            WHERE contact_types.contact_type_id = ?
            GROUP BY contact_types.contact_type_id, contact_types.name
            ORDER BY contact_types.contact_type_id ASC
        "#,
    )
    .bind(contact_type_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(contact_type))
}

pub async fn create_contact_type(
    State(state): State<Arc<AppState>>,
    Json(contact_type): Json<ContactTypeBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            INSERT INTO contact_types (name)
            VALUES (?)
        "#,
    )
    .bind(&contact_type.name)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Contact type created successfully",
    )))
}

pub async fn update_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
    Json(contact_type): Json<ContactTypeBody>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE contact_types
            SET name = ?
            WHERE contact_type_id = ?
        "#,
    )
    .bind(&contact_type.name)
    .bind(contact_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Contact type updated successfully",
    )))
}

pub async fn delete_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE contact_types
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE contact_type_id = ?
        "#,
    )
    .bind(contact_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Contact type deleted successfully",
    )))
}

pub async fn restore_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
) -> Result<Json<SuccessResponse>, AppError> {
    sqlx::query(
        r#"
            UPDATE contact_types
            SET deleted_at = NULL
            WHERE contact_type_id = ?
        "#,
    )
    .bind(contact_type_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SuccessResponse::new(
        "Contact type restored successfully",
    )))
}

pub async fn export_contact_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let contact_types: Vec<ContactTypeDetails> = sqlx::query_as(
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
    let employee_routes = Router::new()
        .route("/", get(get_contact_types))
        .route("/{contact_type_id}", get(get_contact_type))
        .route("/export", get(export_contact_types))
        .route_layer(from_fn(require_employee));

    let admin_routes = Router::new()
        .route("/", post(create_contact_type))
        .route(
            "/{contact_type_id}",
            put(update_contact_type).delete(delete_contact_type),
        )
        .route("/{contact_type_id}/restore", put(restore_contact_type))
        .route_layer(from_fn(require_admin));

    Router::new().merge(employee_routes).merge(admin_routes)
}
