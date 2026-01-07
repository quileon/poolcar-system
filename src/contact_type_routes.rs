use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Postgres};
use std::sync::Arc;

use crate::{
    models::{ContactType, PaginationParams},
    AppState,
};

#[derive(Debug, FromRow, Serialize)]
struct ContactTypeWithCount {
    contact_type_id: i32,
    name: String,
    contact_count: i64,
}

#[derive(Debug, FromRow, Deserialize)]
pub struct ContactTypeBody {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct GetContactTypesResponse {
    contact_types: Vec<ContactTypeWithCount>,
    contact_type_count: usize,
}

pub async fn get_contact_types(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let contact_types = sqlx::query_as::<Postgres, ContactTypeWithCount>(
        r#"
            SELECT
                contact_types.contact_type_id,
                contact_types.name,
                COUNT(contacts.contact_id) as contact_count
            FROM contact_types
            LEFT JOIN contacts ON contact_types.contact_type_id = contacts.contact_type_id
            WHERE contact_types.deleted_at IS NULL
            GROUP BY contact_types.contact_type_id, contact_types.name
            LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let response = GetContactTypesResponse {
        contact_type_count: contact_types.len(),
        contact_types,
    };

    Ok(Json(response))
}

pub async fn create_contact_type(
    State(state): State<Arc<AppState>>,
    Json(contact_type): Json<ContactTypeBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let created_contact_type = sqlx::query_as::<Postgres, ContactType>(
        r#"
            INSERT INTO contact_types (name)
            VALUES ($1)
            RETURNING contact_type_id, name
        "#,
    )
    .bind(contact_type.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(created_contact_type))
}

pub async fn update_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
    Json(contact_type): Json<ContactTypeBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated_contact_type = sqlx::query_as::<Postgres, ContactType>(
        r#"
            UPDATE contact_types
            SET name = $2
            WHERE contact_type_id = $1
            RETURNING contact_type_id, name
        "#,
    )
    .bind(contact_type_id)
    .bind(contact_type.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(updated_contact_type))
}

pub async fn delete_contact_type(
    State(state): State<Arc<AppState>>,
    Path(contact_type_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let delete_contact_type = sqlx::query_as::<Postgres, ContactType>(
        r#"
            UPDATE contact_types
            SET deleted_at = NOW()
            WHERE contact_type_id = $1
            RETURNING contact_type_id, name
        "#,
    )
    .bind(contact_type_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(delete_contact_type))
}
