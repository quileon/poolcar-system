// use std::sync::Arc;

// use axum::{
//     Router,
//     http::{Method, header},
//     routing::get,
// };
// use tower_http::{cors::CorsLayer, trace::TraceLayer};

// use crate::types::AppState;

// pub fn create_axum(state: Arc<AppState>) -> Router {
//     let cors = CorsLayer::new()
//         .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
//         .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
//         .allow_credentials(true);

//     let public = Router::new().route("/", get(|| async { "Hello World!" }));

//     Router::new()
//         .merge(public)
//         .layer(cors)
//         .layer(TraceLayer::new_for_http())
//         .with_state(state)
// }
