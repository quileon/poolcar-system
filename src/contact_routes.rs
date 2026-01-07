use crate::{
    models::{Contact, PaginationParams},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ContactBody {
    pub name: String,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub contact_type_id: i32,
}

#[derive(Debug, FromRow, Serialize)]
struct ContactWithContactType {
    pub contact_id: i32,
    pub name: String,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub contact_type_id: i32,
    pub contact_type_name: String,
}

#[derive(Debug, FromRow, Serialize)]
struct GetContactsResponse {
    contacts: Vec<ContactWithContactType>,
    contact_count: usize,
}

pub async fn get_contacts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(5);

    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 1 } else { limit };
    let offset = (page - 1) * 5;

    let contacts = sqlx::query_as::<Postgres, ContactWithContactType>(
        r#"
            SELECT
                contacts.contact_id,
                contacts.name,
                contacts.latitude,
                contacts.longitude,
                contact_types.contact_type_id,
                contact_types.name as contact_type_name
            FROM contacts
            LEFT JOIN contact_types ON contacts.contact_type_id = contact_types.contact_type_id
            WHERE contacts.deleted_at IS NULL
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

    let response = GetContactsResponse {
        contact_count: contacts.len(),
        contacts,
    };

    Ok(axum::Json(response))
}

pub async fn create_contact(
    State(state): State<Arc<AppState>>,
    Json(contact): Json<ContactBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let created_contact = sqlx::query_as::<Postgres, Contact>(
        r#"
            INSERT INTO contacts (name, latitude, longitude, contact_type_id)
            VALUES ($1, $2, $3, $4)
            RETURNING contact_id, name, latitude, longitude, contact_type_id
        "#,
    )
    .bind(contact.name)
    .bind(contact.latitude)
    .bind(contact.longitude)
    .bind(contact.contact_type_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(created_contact))
}

pub async fn update_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
    Json(contact): Json<ContactBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated_contact = sqlx::query_as::<Postgres, Contact>(
        r#"
            UPDATE contacts
            SET name = $2, latitude = $3, longitude = $4, contact_type_id = $5
            WHERE contact_id = $1
            RETURNING contact_id, name, latitude, longitude, contact_type_id
        "#,
    )
    .bind(contact_id)
    .bind(contact.name)
    .bind(contact.latitude)
    .bind(contact.longitude)
    .bind(contact.contact_type_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(updated_contact))
}

pub async fn delete_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let deleted_contact = sqlx::query_as::<Postgres, Contact>(
        r#"
            UPDATE contacts
            SET deleted_at = NOW()
            WHERE contact_id = $1
            RETURNING contact_id, name, latitude, longitude, contact_type_id
        "#,
    )
    .bind(contact_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(deleted_contact))
}
