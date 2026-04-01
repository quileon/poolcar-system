use anyhow::Context;
use deadpool_redis::Runtime;
use dotenvy;
use poolcar_backend::{config::Config, create_app};
use rand::{distr, Rng};
use rumqttc::{MqttOptions, Transport};
use sqlx::mysql::MySqlPoolOptions;
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .compact()
        .init();

    let config = Config::from_env()?;
    tracing::info!("Environment configuration loaded");

    // Database
    let db_pool = MySqlPoolOptions::new()
        .connect(&config.database_url)
        .await
        .context("Failed to cretae Database pool")?;
    tracing::info!("Database connection established");

    // Migrate
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .context("Failed to run migrations")?;
    tracing::info!("Migrations completed");

    // Redis
    let redis_cfg = deadpool_redis::Config::from_url(&config.redis_url);
    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool");
    tracing::info!("Redis connection established");

    // MQTT
    let mut rng = rand::rng();
    let random_suffix: String = (0..8)
        .map(|_| rng.sample(distr::Alphanumeric) as char)
        .collect();
    let mqtt_client = format!("{}-{}", config.mqtt_client, random_suffix);

    let mut mqtt_options = MqttOptions::new(mqtt_client, &config.mqtt_url, 8883);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(&config.mqtt_username, &config.mqtt_password);

    let ca_cert = std::fs::read(&config.mqtt_ca_crt).context("Failed to read MQTT certificate")?;
    let transport = Transport::Tls(rumqttc::TlsConfiguration::Simple {
        ca: ca_cert,
        alpn: None,
        client_auth: None,
    });
    mqtt_options.set_transport(transport);
    tracing::info!("MQTT client configured");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7270")
        .await
        .context("Failed to bind to port 7270")?;
    let listener_address = listener.local_addr()?;

    // Axum
    let app = create_app(db_pool, redis_pool, Some(mqtt_options), config);
    tracing::info!("Axum started on {}", listener_address);
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
        _ = ctrl_c => {
            tracing::info!("Shutdown signal received (Ctrl+C)");
        },
        _ = terminate => {
            tracing::info!("Shutdown signal received (SIGTERM)");
        },
    }
}
