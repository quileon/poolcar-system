mod auth_utils;
pub mod config;
mod error;
mod handlers;
mod middleware;
pub mod models;
mod redis;
mod routes;
mod state;
mod tasks;
mod types;
mod websocket;

use crate::{
    routes::{
        activity_routes, auth_routes, car_routes, chart_routes, contact_routes,
        live_tracking_routes, tracker_routes, user_routes,
    },
    state::AppState,
};
use axum::{http::Method, routing::get, Router};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn create_app(
    db_pool: sqlx::MySqlPool,
    redis_pool: deadpool_redis::Pool,
    mqtt_options: Option<rumqttc::MqttOptions>,
    config: config::Config,
) -> Router {
    let (tx, _) = broadcast::channel::<String>(100);

    let app_state = Arc::new(AppState {
        db: db_pool,
        redis: redis_pool,
        tx,
        config,
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Only spawn MQTT task if mqtt_options are provided (for testing)
    if let Some(mqtt_options) = mqtt_options {
        let mqtt_state = app_state.clone();
        tokio::spawn(async move { tasks::mqtt::mqtt_loop(mqtt_state, mqtt_options).await });
    }

    // Spawn distance handler background task
    let distance_state = app_state.clone();
    tokio::spawn(async move { tasks::distance::distance_loop(distance_state).await });

    let public_routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/auth", auth_routes::routes());

    let websocket_routes = Router::new()
        .nest("/chart", websocket::chart_websocket::routes())
        .nest("/live", websocket::live_tracking_websocket::routes());

    let protected_routes = Router::new()
        .nest("/cars", car_routes::routes())
        .nest("/chart", chart_routes::routes())
        .nest("/contacts", contact_routes::routes())
        .nest("/activities", activity_routes::routes())
        .nest("/live", live_tracking_routes::routes())
        .nest("/trackers", tracker_routes::routes())
        .nest("/users", user_routes::routes())
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ));

    Router::new()
        .nest("/api", public_routes)
        .nest("/api", protected_routes)
        .nest("/ws", websocket_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state)
}
