mod car_routes;
mod dashboard_routes;
mod models;
mod tracker_routes;

use axum::{
    routing::{get, post, put},
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
        .route("/trackers", get(tracker_routes::get_trackers))
        .route("/trackers", post(tracker_routes::create_tracker))
        .route(
            "/trackers/{tracker_id}",
            put(tracker_routes::update_tracker),
        )
        .with_state(app_state)
}
