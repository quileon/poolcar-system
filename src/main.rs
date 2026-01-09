use anyhow::Context;
use deadpool_redis::Runtime;
use dotenvy;
use poolcar_tracking_system_backend_test::create_app;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env
    dotenvy::dotenv().ok();

    // Setup database pool
    let db_url = std::env::var("DATABASE_URL").context("Failed to read DATABASE_URL")?;
    let db_pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .context("Failed to connect to Postgres")?;
    println!("Database OK");

    // Setup redis pool
    let redis_url = std::env::var("REDIS_URL").context("Failed to read REDIS_URL")?;
    let redis_cfg = deadpool_redis::Config::from_url(redis_url);
    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool");

    // Port binding
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .context("Failed to bind to port 3000")?;
    println!("Listening on {}", listener.local_addr()?);

    // Axum
    let app = create_app(db_pool, redis_pool);
    axum::serve(listener, app).await?;

    Ok(())
}
