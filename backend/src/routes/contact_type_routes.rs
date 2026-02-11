use crate::{
    error::AppError,
    models::contact_type::{
        ContactType, ContactTypeBody, ContactTypeWithCount, GetContactTypesResponse,
    },
    types::PaginationParams,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, put},
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
        .route(
            "/{contact_type_id}",
            put(update_contact_type).delete(delete_contact_type),
        )
}
