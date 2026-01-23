use anyhow::Context;
use deadpool_redis::Runtime;
use dotenvy;
use poolcar_backend::create_app;
use rumqttc::{MqttOptions, Transport};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::signal;

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

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .context("Failed to run migrations")?;
    println!("Migrations OK");

    // Setup redis pool
    let redis_url = std::env::var("REDIS_URL").context("Failed to read REDIS_URL")?;
    let redis_cfg = deadpool_redis::Config::from_url(redis_url);
    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool");
    println!("Redis OK");

    // Setup MQTT options with TLS
    let mqtt_url = std::env::var("MQTT_URL").context("Failed to read MQTT_URL")?;
    let mqtt_client = std::env::var("MQTT_CLIENT").context("Failed to read MQTT_CLIENT")?;
    let mqtt_username = std::env::var("MQTT_USERNAME").context("Failed to read MQTT_USERNAME")?;
    let mqtt_password = std::env::var("MQTT_PASSWORD").context("Failed to read MQTT_PASSWORD")?;

    let mut mqtt_options = MqttOptions::new(mqtt_client, mqtt_url, 8883);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(mqtt_username, mqtt_password);

    let ca_cert = include_bytes!("../assets/emqxsl-ca.crt").to_vec();
    let transport = Transport::Tls(rumqttc::TlsConfiguration::Simple {
        ca: ca_cert,
        alpn: None,
        client_auth: None,
    });
    mqtt_options.set_transport(transport);
    println!("MQTT OK");

    // Port binding
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7270")
        .await
        .context(format!("Failed to bind to port 7270"))?;
    println!("Listening on {}", listener.local_addr()?);

    // Axum
    let app = create_app(db_pool, redis_pool, Some(mqtt_options));
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
