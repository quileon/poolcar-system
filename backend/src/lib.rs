mod activity_routes;
mod auth_routes;
mod auth_utils;
mod car_routes;
mod car_type_routes;
mod chart_handler;
mod chart_routes;
mod chart_websocket;
mod contact_routes;
mod contact_type_routes;
mod dashboard_routes;
mod error;
mod history_routes;
mod live_tracking_routes;
mod live_tracking_websocket;
mod models;
mod mqtt_handlers;
mod mqtt_payload_handler;
mod tracker_routes;
mod user_routes;

use axum::{
    http::Method,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
    pub tx: broadcast::Sender<String>,
}

pub fn create_app(
    db_pool: sqlx::PgPool,
    redis_pool: deadpool_redis::Pool,
    mqtt_options: Option<rumqttc::MqttOptions>,
) -> Router {
    let (tx, _) = broadcast::channel::<String>(100);

    let app_state = Arc::new(AppState {
        db: db_pool,
        redis: redis_pool,
        tx,
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Only spawn MQTT task if mqtt_options are provided (for testing)
    if let Some(mqtt_options) = mqtt_options {
        let mqtt_state = app_state.clone();
        tokio::spawn(
            async move { mqtt_handlers::handle_mqtt_loop(mqtt_state, mqtt_options).await },
        );
    }

    // Spawn chart handler background task
    let chart_state = app_state.clone();
    tokio::spawn(async move { chart_handler::chart_handler(chart_state).await });

    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/dashboard", get(dashboard_routes::get_dashboard_data))
        .route("/cars", get(car_routes::get_cars))
        .route("/cars/export", get(car_routes::export_cars))
        .route("/cars/{car_id}", get(car_routes::get_car))
        .route("/cars", post(car_routes::create_car))
        .route("/cars/{car_id}", put(car_routes::update_car))
        .route("/cars/{car_id}", delete(car_routes::delete_car))
        .route("/trackers", get(tracker_routes::get_trackers))
        .route("/trackers/export", get(tracker_routes::export_trackers))
        .route("/trackers/{tracker_id}", get(tracker_routes::get_tracker))
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
        .route("/cars/types/export", get(car_type_routes::export_car_types))
        .route(
            "/cars/types/{car_type_id}",
            get(car_type_routes::get_car_type),
        )
        .route("/cars/types", post(car_type_routes::create_car_type))
        .route(
            "/cars/types/{car_type_id}",
            put(car_type_routes::update_car_type),
        )
        .route(
            "/cars/types/{car_type_id}",
            delete(car_type_routes::delete_car_type),
        )
        .route("/contacts", get(contact_routes::get_contacts))
        .route("/contacts", post(contact_routes::create_contact))
        .route(
            "/contacts/{contact_id}",
            put(contact_routes::update_contact),
        )
        .route(
            "/contacts/{contact_id}",
            delete(contact_routes::delete_contact),
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
        .route("/activities", get(activity_routes::get_activities))
        .route("/activities", post(activity_routes::create_activity))
        .route(
            "/activities/{activity_id}",
            put(activity_routes::update_activity),
        )
        .route(
            "/activities/{activity_id}",
            delete(activity_routes::delete_activity),
        )
        .route("/histories", get(history_routes::get_histories))
        .route("/histories", post(history_routes::create_history))
        .route(
            "/histories/{history_id}",
            put(history_routes::update_history),
        )
        .route(
            "/histories/{history_id}",
            delete(history_routes::delete_history),
        )
        .route(
            "/ws/live",
            get(live_tracking_websocket::live_tracking_handler),
        )
        .route(
            "/live",
            get(live_tracking_routes::get_live_tracking_history),
        )
        .route("/auth/login", post(auth_routes::login_handler))
        .route("/users", get(user_routes::get_users))
        .route("/users", post(user_routes::create_user))
        .route("/users/{user_id}", get(user_routes::get_user))
        .route("/users/{user_id}", put(user_routes::update_user))
        .route("/users/{user_id}", delete(user_routes::delete_user))
        .route("/ws/chart", get(chart_websocket::chart_handler))
        .route("/chart", get(chart_routes::get_chart_history))
        .with_state(app_state)
        .layer(cors)
}
