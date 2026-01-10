use anyhow::Context;
use poolcar_tracking_system_backend_test::create_app;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct CarType {
    pub car_type_id: i32,
    pub name: String,
}

async fn spawn_app(db_pool: PgPool) -> (String, JoinHandle<()>) {
    seed_database(&db_pool).await;

    // Setup redis pool
    dotenvy::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL")
        .context("Failed to read REDIS_URL")
        .unwrap();
    let redis_cfg = deadpool_redis::Config::from_url(redis_url);
    let redis_pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create Redis pool");

    let app = create_app(db_pool, redis_pool, None);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (address, handle)
}

async fn seed_database(pool: &PgPool) {
    sqlx::query(
        r#"
            INSERT INTO car_types (name)
            VALUES ('Delivery'), ('Passenger')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed car types");
}

#[sqlx::test]
async fn test_get_car_types(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/cars/types", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let car_type_count = body["car_type_count"]
        .as_u64()
        .expect("car_type_count should be a number");
    assert_eq!(car_type_count, 2, "Expected 2 car_types");
    let car_types = body["car_types"]
        .as_array()
        .expect("car_types should be an array");
    assert_eq!(car_types.len(), 2, "Expected 2 car_types in array");

    handle.abort();
}

#[sqlx::test]
async fn test_create_car_type(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/cars/types", address))
        .json(&serde_json::json!({ "name": "Cargo" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Cargo");
    assert_eq!(
        body["car_type_id"]
            .as_i64()
            .expect("car_type_id should be a number"),
        3,
    );

    // Database check
    let car_type = sqlx::query_as::<_, CarType>("SELECT * FROM car_types WHERE car_type_id = 3")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch car_type");
    assert_eq!(car_type.car_type_id, 3);
    assert_eq!(car_type.name, "Cargo");

    handle.abort();
}

#[sqlx::test]
async fn test_update_car_type(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/cars/types/1", address))
        .json(&serde_json::json!({
            "name": "Cargo"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Cargo");
    assert_eq!(
        body["car_type_id"]
            .as_u64()
            .expect("car_type_id should be a number"),
        1
    );

    // Database check
    let query_response =
        sqlx::query_as::<_, CarType>("SELECT * FROM car_types WHERE car_type_id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch car_type");
    assert_eq!(query_response.car_type_id, 1);
    assert_eq!(query_response.name, "Cargo");

    handle.abort();
}

#[sqlx::test]
async fn test_delete_car_type(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .delete(format!("{}/cars/types/1", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Delivery");
    assert_eq!(
        body["car_type_id"]
            .as_u64()
            .expect("car_type_id should be a number"),
        1
    );

    // Database check
    let query_response = sqlx::query_as::<_, CarType>(
        "SELECT * FROM car_types WHERE car_type_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch car_type");
    assert_eq!(query_response.car_type_id, 1);
    assert_eq!(query_response.name, "Delivery");

    handle.abort();
}
