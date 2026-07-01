mod auth;
mod entities;

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
use rocket::routes;
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
    let redis_url =
        env::var("REDIS_URL").map_err(|_| anyhow!("REDIS_URL environment variable is not set"))?;
    let rocket_port = env::var("ROCKET_PORT")
        .map_err(|_| anyhow!("ROCKET_PORT environment variable is not set"))?;
    let rocket_port: u16 = rocket_port
        .parse()
        .map_err(|e| anyhow!("Failed to parse ROCKET_PORT as u16: {}", e))?;

    let (db, redis) = tokio::try_join!(connect_db(&db_url), connect_redis(&redis_url))?;
    run_rocket(rocket_port, db, redis).await?;

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

// async fn connect_mqtt(
//     mqtt_host: &str,
//     mqtt_port: u16,
//     mqtt_client: &str,
//     mqtt_use_tls: bool,
//     mqtt_username: &str,
//     mqtt_password: &str,
// ) -> anyhow::Result<(rumqttc::AsyncClient, rumqttc::EventLoop)> {
//     let mut rng = rand::rng();
//     let random_suffix: String = (0..8)
//         .map(|_| rng.sample(distr::Alphanumeric) as char)
//         .collect();
//     let mqtt_client = format!("{}-{}", mqtt_client, random_suffix);

//     let mut mqtt_options = MqttOptions::new(mqtt_client, mqtt_host, mqtt_port);
//     mqtt_options.set_keep_alive(Duration::from_secs(5));
//     mqtt_options.set_credentials(mqtt_username, mqtt_password);

//     if mqtt_use_tls {
//         mqtt_options.set_transport(Transport::Tls(TlsConfiguration::Native));
//     }

//     let (mqtt_client, mqtt_event_loop) = AsyncClient::new(mqtt_options, 10);
//     info!("MQTT connection initialized successfully!");
//     Ok((mqtt_client, mqtt_event_loop))
// }

async fn run_rocket(
    rocket_port: u16,
    db: sea_orm::DatabaseConnection,
    redis: redis::Client,
) -> anyhow::Result<()> {
    let jwt_secret = env::var("JWT_SECRET")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| b"i-love-curren-chan".to_vec());

    let figment = rocket::Config::figment()
        .merge(("port", rocket_port))
        .merge(("address", "0.0.0.0"))
        .merge(("cli_colors", false))
        .merge(("log_level", "normal"));

    rocket::custom(figment)
        .manage(db)
        .manage(redis)
        .manage(auth::JwtSecret(jwt_secret))
        .mount(
            "/",
            routes![
                pages::login::login,
                pages::login::post_login,
                pages::login::logout,
                pages::api::verify,
                pages::api::api_login,
                pages::dashboard::dashboard,
                pages::trackers::list_trackers,
                pages::trackers::create_tracker,
                pages::trackers::update_tracker,
                pages::trackers::delete_tracker,
                pages::cars::list_cars,
                pages::cars::create_car,
                pages::cars::update_car,
                pages::cars::delete_car,
                pages::contacts::list_contacts,
                pages::contacts::create_contact,
                pages::contacts::update_contact,
                pages::contacts::delete_contact
            ],
        )
        .mount("/js", rocket::fs::FileServer::from("templates/js"))
        .mount("/css", rocket::fs::FileServer::from("templates/css"))
        .mount("/assets", rocket::fs::FileServer::from("templates/assets"))
        .register("/", rocket::catchers![pages::login::unauthorized])
        .launch()
        .await?;

    Ok(())
}
