use anyhow::Context;
use dotenvy;
use poolcar_tracking_system_backend_test::create_app;
use sqlx::postgres::PgPoolOptions;

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

    // Port binding
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .context("Failed to bind to port 3000")?;
    println!("Listening on {}", listener.local_addr()?);

    // Axum
    let app = create_app(pool);
    axum::serve(listener, app).await?;

    Ok(())
}
