use anyhow::{Context, Result};

/// Configuration type
/// Holds required configuration values.
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub mqtt_url: String,
    pub mqtt_client: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub mqtt_secure: bool,
    pub mqtt_ca_crt: Option<String>,
    pub mqtt_port: u16,
    pub jwt_secret: String,
    pub google_api_key: String,
}

impl Config {
    /// Load configuration from environment variables.
    /// Returns an error if any required variable is missing.
    pub fn from_env() -> Result<Config> {
        let mqtt_secure = std::env::var("MQTT_SECURE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .context("MQTT_SECURE must be a boolean (true/false)")?;

        let mqtt_ca_crt = if mqtt_secure {
            Some(
                std::env::var("MQTT_CA_CRT")
                    .context("MQTT_CA_CRT must be set when MQTT_SECURE is true")?,
            )
        } else {
            std::env::var("MQTT_CA_CRT").ok()
        };

        Ok(Config {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            redis_url: std::env::var("REDIS_URL").context("REDIS_URL must be set")?,
            mqtt_url: std::env::var("MQTT_URL").context("MQTT_URL must be set")?,
            mqtt_client: std::env::var("MQTT_CLIENT").context("MQTT_CLIENT must be set")?,
            mqtt_username: std::env::var("MQTT_USERNAME").context("MQTT_USERNAME must be set")?,
            mqtt_password: std::env::var("MQTT_PASSWORD").context("MQTT_PASSWORD must be set")?,
            mqtt_secure,
            mqtt_ca_crt,
            mqtt_port: std::env::var("MQTT_PORT")
                .context("MQTT_PORT must be set")?
                .parse()?,
            jwt_secret: std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
            google_api_key: std::env::var("GOOGLE_API_KEY")
                .context("GOOGLE_API_KEY must be set")?,
        })
    }
}
