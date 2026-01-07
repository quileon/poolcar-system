mod car_routes;
mod car_type_routes;
mod contact_type_routes;
mod dashboard_routes;
mod models;
mod tracker_routes;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

pub struct AppState {
    pub db: PgPool,
}

pub fn create_app(pool: PgPool) -> Router {
    let app_state = Arc::new(AppState { db: pool });

    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/dashboard", get(dashboard_routes::get_dashboard_data))
        .route("/cars", get(car_routes::get_cars))
        .route("/cars", post(car_routes::create_car))
        .route("/cars/{car_id}", put(car_routes::update_car))
        .route("/cars/{car_id}", delete(car_routes::delete_car))
        .route("/trackers", get(tracker_routes::get_trackers))
        .route("/trackers", post(tracker_routes::create_tracker))
        .route(
            "/trackers/{tracker_id}",
            put(tracker_routes::update_tracker),
        )
        .route(
            "/trackers/{tracker_id}",
            delete(tracker_routes::delete_tracker),
        )
        .route("/cars/types", get(car_type_routes::get_car_types))
        .route("/cars/types", post(car_type_routes::create_car_type))
        .route(
            "/cars/types/{car_type_id}",
            put(car_type_routes::update_car_type),
        )
        .route(
            "/cars/types/{car_type_id}",
            delete(car_type_routes::delete_car_type),
        )
        .route(
            "/contacts/types",
            get(contact_type_routes::get_contact_types),
        )
        .route(
            "/contacts/types",
            post(contact_type_routes::create_contact_type),
        )
        .route(
            "/contacts/types/{contact_type_id}",
            put(contact_type_routes::update_contact_type),
        )
        .route(
            "/contacts/types/{contact_type_id}",
            delete(contact_type_routes::delete_contact_type),
        )
        .with_state(app_state)
}
