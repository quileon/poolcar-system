mod dashboard_routes;
mod models;

use anyhow::Context;
use axum::{routing::get, Router};
use dotenvy;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;

use dashboard_routes::get_dashboard_data;

pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env
    dotenvy::dotenv().ok();

    // Setup database pool
    let db_url = std::env::var("DATABASE_URL").context("Failed to read DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .context("Failed to connect to Postgres")?;
    println!("Database OK");

    // App state
    let app_state = Arc::new(AppState { db: pool });

    // Port binding
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .context("Failed to bind to port 3000")?;
    println!("Listening on {}", listener.local_addr()?);

    // Axum
    let app = Router::new()
        .route("/", get(root))
        .route("/dashboard", get(get_dashboard_data))
        .with_state(app_state);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello world!"
}
