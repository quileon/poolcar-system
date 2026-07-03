mod auth;
mod entities;
mod loops;
mod pages;
mod run;
mod types;

use crate::entities::sea_orm_active_enums::UserRole;
use crate::entities::users::{self, Entity as Users};
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
// use rand::{RngExt, distr};
// use rumqttc::{AsyncClient, MqttOptions, TlsConfiguration, Transport};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use std::env;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "poolcar")]
#[command(about = "Poolcar Backend CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Axum web server (default)
    Run,
    /// Create an initial admin user
    Createsuperuser {
        /// Username for the new admin account
        #[arg(short, long, default_value = "admin")]
        username: String,
        /// Password for the new admin account
        #[arg(short, long, default_value = "admin")]
        password: String,
        /// Email address for the new admin account
        #[arg(short, long, default_value = "admin@example.com")]
        email: String,
        /// Full name of the admin user
        #[arg(short, long, default_value = "Admin User")]
        full_name: String,
    },
}

#[rocket::main]
async fn main() {
    dotenv().ok();

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "htmx=debug,info".into())
        .add_directive("sqlx=warn".parse().unwrap())
        .add_directive("sea_orm=warn".parse().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            tracing::error!("DATABASE_URL environment variable is not set");
            std::process::exit(1);
        }
    };

    let result = match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => run_server(db_url).await,
        Commands::Createsuperuser {
            username,
            password,
            email,
            full_name,
        } => create_superuser(db_url, username, password, email, full_name).await,
    };

    if let Err(err) = result {
        tracing::error!("{}", err);
        std::process::exit(1);
    }
}

async fn create_superuser(
    db_url: String,
    username: String,
    password: String,
    email: String,
    full_name: String,
) -> anyhow::Result<()> {
    let db = connect_db(&db_url).await?;

    let existing_user: Option<users::Model> = Users::find()
        .filter(users::Column::Username.eq(&username))
        .one(&db)
        .await?;

    if existing_user.is_some() {
        return Err(anyhow!("User with username '{}' already exists", username));
    }

    let new_user = entities::users::ActiveModel {
        username: Set(username),
        password: Set(password),
        email: Set(email),
        full_name: Set(full_name),
        user_role: Set(UserRole::Admin),
        ..Default::default()
    };

    new_user.insert(&db).await?;

    info!("Superuser created successfully!");

    Ok(())
}

async fn run_server(db_url: String) -> anyhow::Result<()> {
    // Rocket environment variables
    let redis_url =
        env::var("REDIS_URL").map_err(|_| anyhow!("REDIS_URL environment variable is not set"))?;
    let rocket_port = env::var("ROCKET_PORT")
        .map_err(|_| anyhow!("ROCKET_PORT environment variable is not set"))?;
    let rocket_port: u16 = rocket_port
        .parse()
        .map_err(|e| anyhow!("Failed to parse ROCKET_PORT as u16: {}", e))?;

    // MQTT environment variables
    let mqtt_host = env::var("MQTT_URL").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_port: u16 = env::var("MQTT_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .unwrap_or(1883);
    let mqtt_client = env::var("MQTT_CLIENT").unwrap_or_else(|_| "poolcar_backend".to_string());
    let mqtt_use_tls = env::var("MQTT_SECURE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    let mqtt_username = env::var("MQTT_USERNAME").unwrap_or_else(|_| "poolcar".to_string());
    let mqtt_password = env::var("MQTT_PASSWORD").unwrap_or_else(|_| "poolcar".to_string());
    let mqtt_topic = env::var("MQTT_TOPIC").unwrap_or_else(|_| "poolcar/+".to_string());

    let (db, redis) = tokio::try_join!(connect_db(&db_url), connect_redis(&redis_url))?;

    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(1024);
    let tx_clone = tx.clone();

    tokio::try_join!(
        run::run_rocket(rocket_port, db.clone(), redis.clone(), tx_clone),
        run::run_mqtt(
            db.clone(),
            redis.clone(),
            &mqtt_host,
            mqtt_port,
            &mqtt_client,
            mqtt_use_tls,
            &mqtt_username,
            &mqtt_password,
            &mqtt_topic,
            tx,
        ),
        run::run_audit(db, redis)
    )?;

    Ok(())
}

async fn connect_db(db_url: &str) -> Result<DatabaseConnection, anyhow::Error> {
    let db_url = if db_url.starts_with("mariadb://") {
        db_url.replacen("mariadb://", "mysql://", 1)
    } else {
        db_url.to_string()
    };
    let db = Database::connect(&db_url).await?;
    db.ping().await?;
    Migrator::up(&db, None).await?;
    info!("Database connection established successfully!");
    Ok(db)
}

async fn connect_redis(redis_url: &str) -> Result<redis::Client, anyhow::Error> {
    let redis = redis::Client::open(redis_url)?;
    let mut redis_conn = redis.get_multiplexed_async_connection().await?;
    redis::cmd("PING")
        .query_async::<()>(&mut redis_conn)
        .await?;
    info!("Redis connection established successfully!");
    Ok(redis)
}
