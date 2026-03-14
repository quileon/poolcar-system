use crate::config;
use tokio::sync::broadcast;

pub struct AppState {
    pub db: sqlx::MySqlPool,
    pub redis: deadpool_redis::Pool,
    pub tx: broadcast::Sender<String>,
    pub config: config::Config,
}
