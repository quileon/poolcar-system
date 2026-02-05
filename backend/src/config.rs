use anyhow::{Context, Result};

/// Configuration type
/// Holds required configuration values.
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub mqtt_url: String,
    pub mqtt_client: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub jwt_secret: String,
}

impl Config {
    /// Load configuration from environment variables.
    /// Returns an error if any required variable is missing.
    pub fn from_env() -> Result<Config> {
        Ok(Config {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            redis_url: std::env::var("REDIS_URL").context("REDIS_URL must be set")?,
            mqtt_url: std::env::var("MQTT_URL").context("MQTT_URL must be set")?,
            mqtt_client: std::env::var("MQTT_CLIENT").context("MQTT_CLIENT must be set")?,
            mqtt_username: std::env::var("MQTT_USERNAME").context("MQTT_USERNAME must be set")?,
            mqtt_password: std::env::var("MQTT_PASSWORD").context("MQTT_PASSWORD must be set")?,
            jwt_secret: std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
        })
    }
}
