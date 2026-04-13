use anyhow::Context;
use clap::Parser;
use poolcar_backend::auth_utils::hash_password;
use poolcar_backend::config::Config;
use sqlx::mysql::MySqlPoolOptions;

#[derive(Parser, Debug)]
#[command(author, version, about = "Create an initial admin user", long_about = None)]
struct Args {
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

    /// Role ID for the admin user (typically 1 for Admin)
    #[arg(short, long, default_value_t = 1)]
    role_id: i32,
}

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

    // Parse command line arguments
    let args = Args::parse();

    let config = Config::from_env()?;
    tracing::info!("Environment configuration loaded");

    // Database
    let db_pool = MySqlPoolOptions::new()
        .connect(&config.database_url)
        .await
        .context("Failed to connect to the database pool")?;
    tracing::info!("Database connection established");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .context("Failed to run migrations")?;
    tracing::info!("Migrations completed");

    tracing::info!("Creating admin user: {}", args.username);

    // Hash the password
    let hashed_password = hash_password(&args.password).context("Failed to hash password")?;

    // Insert the user into the database
    sqlx::query(
        "INSERT INTO users (username, email, password, full_name, user_role_id) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&args.username)
    .bind(&args.email)
    .bind(&hashed_password)
    .bind(&args.full_name)
    .bind(args.role_id)
    .execute(&db_pool)
    .await
    .context("Failed to create admin user!")?;

    tracing::info!("Admin user '{}' created successfully!", args.username);

    Ok(())
}
