use std::sync::Arc;

use axum::middleware::from_fn;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use reqwest::{header::CONTENT_TYPE, Client};

use crate::middleware::require_employee;
use crate::{
    error::AppError,
    models::google_map::{GoogleMapPayload, GoogleMapResponse, GoogleMapSearchParams},
    state::AppState,
    types::SuccessDataResponse,
};

async fn search_places(
    Query(query): Query<GoogleMapSearchParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SuccessDataResponse>, AppError> {
    let url = "https://places.googleapis.com/v1/places:searchText";
    let payload = GoogleMapPayload::new(
        query.name,
        "en".into(),
        -6.382310833,
        107.1725405,
        50000.0,
        Some(5),
    );
    let client = Client::new();

    let response: GoogleMapResponse = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header("X-Goog-Api-Key", &state.config.google_api_key)
        .header(
            "X-Goog-FieldMask",
            "places.id,places.displayName,places.formattedAddress,places.location",
        )
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    Ok(Json(SuccessDataResponse::new(response)?))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(search_places))
        .route_layer(from_fn(require_employee))
}
