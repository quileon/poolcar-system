use anyhow::Context;
use poolcar_tracking_system_backend_test::create_app;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Car {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub tracker_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CarWithTracker {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub car_type_name: String,
    pub tracker_id: Option<i32>,
    pub tracker_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub message: String,
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

    sqlx::query(
        r#"
            INSERT INTO trackers (name)
            VALUES ('Batman Tracker'), ('Superman Tracker'), ('Robin Tracker')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed trackers");

    sqlx::query(
        r#"
            INSERT INTO cars (name, police_number, active, car_type_id, tracker_id)
            VALUES ('Car 1', 'ABC123', true, 1, 1),
                   ('Car 2', 'DEF456', false, 2, 2),
                   ('Car 3', 'GHI789', true, 1, NULL)
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed cars");
}

#[sqlx::test]
async fn test_get_cars(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/cars", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let car_count = body["car_count"]
        .as_u64()
        .expect("car_count should be a number");
    assert_eq!(car_count, 3, "Expected 3 car_count");

    // Data check
    let cars: Vec<CarWithTracker> =
        serde_json::from_value(body["cars"].clone()).expect("Failed to deserialize cars array");

    assert_eq!(cars.len(), 3, "Expected 3 cars in array");

    // Car 1
    assert_eq!(cars[0].car_id, 1);
    assert_eq!(cars[0].name, "Car 1");
    assert_eq!(cars[0].police_number, "ABC123");
    assert_eq!(cars[0].active, true);
    assert_eq!(cars[0].car_type_id, 1);
    assert_eq!(cars[0].car_type_name, "Delivery");
    assert_eq!(cars[0].tracker_id, Some(1));
    assert_eq!(cars[0].tracker_name, Some("Batman Tracker".to_string()));

    // Car 2
    assert_eq!(cars[1].car_id, 2);
    assert_eq!(cars[1].name, "Car 2");
    assert_eq!(cars[1].police_number, "DEF456");
    assert_eq!(cars[1].active, false);
    assert_eq!(cars[1].car_type_id, 2);
    assert_eq!(cars[1].car_type_name, "Passenger");
    assert_eq!(cars[1].tracker_id, Some(2));
    assert_eq!(cars[1].tracker_name, Some("Superman Tracker".to_string()));

    // Car 3 - has no tracker
    assert_eq!(cars[2].car_id, 3);
    assert_eq!(cars[2].name, "Car 3");
    assert_eq!(cars[2].police_number, "GHI789");
    assert_eq!(cars[2].active, true);
    assert_eq!(cars[2].car_type_id, 1);
    assert_eq!(cars[2].car_type_name, "Delivery");
    assert_eq!(cars[2].tracker_id, None);
    assert_eq!(cars[2].tracker_name, None);

    handle.abort();
}

#[sqlx::test]
async fn test_create_car_without_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/cars", address))
        .json(&serde_json::json!({ "name": "Car 4", "police_number": "JKL101112", "active": true, "car_type_id": 1 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Car = serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
        .expect("Failed to deserialize JSON");
    assert_eq!(body.car_id, 4);
    assert_eq!(body.name, "Car 4");
    assert_eq!(body.police_number, "JKL101112");
    assert_eq!(body.active, true);
    assert_eq!(body.car_type_id, 1);
    assert_eq!(body.tracker_id, None);

    // Database check
    let query_response = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE car_id = 4")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch car");
    assert_eq!(query_response.car_id, 4);
    assert_eq!(query_response.police_number, "JKL101112");
    assert_eq!(query_response.active, true);
    assert_eq!(query_response.car_type_id, 1);
    assert_eq!(query_response.tracker_id, None);

    handle.abort();
}

#[sqlx::test]
async fn test_create_car_with_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/cars", address))
        .json(&serde_json::json!({ "name": "Car 4", "police_number": "JKL101112", "active": true, "car_type_id": 1, "tracker_id": 3 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Car = serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
        .expect("Failed to deserialize JSON");
    assert_eq!(body.car_id, 4);
    assert_eq!(body.name, "Car 4");
    assert_eq!(body.police_number, "JKL101112");
    assert_eq!(body.active, true);
    assert_eq!(body.car_type_id, 1);
    assert_eq!(body.tracker_id, Some(3));

    // Database check
    let query_response = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE car_id = 4")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch car");
    assert_eq!(query_response.car_id, 4);
    assert_eq!(query_response.name, "Car 4");
    assert_eq!(query_response.police_number, "JKL101112");
    assert_eq!(query_response.active, true);
    assert_eq!(query_response.car_type_id, 1);
    assert_eq!(query_response.tracker_id, Some(3));

    handle.abort();
}

#[sqlx::test]
async fn test_create_car_with_existing_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/cars", address))
        .json(&serde_json::json!({ "name": "Car 4", "police_number": "JKL101112", "active": true, "car_type_id": 1, "tracker_id": 1 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 400);

    // Body check
    let body: ErrorResponse =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.message, "Tracker is already assigned");

    // Database check
    let query_response = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM cars WHERE car_id = 4")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch car");
    assert_eq!(query_response, 0, "Car 4 should not exist");

    handle.abort();
}

#[sqlx::test]
async fn test_update_car_without_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/cars/3", address))
        .json(&serde_json::json!({ "name": "Car 3.3", "police_number": "JKL101112", "active": false, "car_type_id": 2, "tracker_id": null }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Car = serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
        .expect("Failed to deserialize JSON");
    assert_eq!(body.car_id, 3);
    assert_eq!(body.name, "Car 3.3");
    assert_eq!(body.police_number, "JKL101112");
    assert_eq!(body.active, false);
    assert_eq!(body.car_type_id, 2);
    assert_eq!(body.tracker_id, None);

    // Database check
    let query_response = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE car_id = 3")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch car");
    assert_eq!(query_response.car_id, 3);
    assert_eq!(query_response.name, "Car 3.3");
    assert_eq!(query_response.police_number, "JKL101112");
    assert_eq!(query_response.active, false);
    assert_eq!(query_response.car_type_id, 2);
    assert_eq!(query_response.tracker_id, None);

    handle.abort();
}

#[sqlx::test]
async fn test_update_car_into_with_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/cars/3", address))
        .json(&serde_json::json!({ "name": "Car 3.3", "police_number": "JKL101112", "active": false, "car_type_id": 2, "tracker_id": 3 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Car = serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
        .expect("Failed to deserialize JSON");
    assert_eq!(body.car_id, 3);
    assert_eq!(body.name, "Car 3.3");
    assert_eq!(body.police_number, "JKL101112");
    assert_eq!(body.active, false);
    assert_eq!(body.car_type_id, 2);
    assert_eq!(body.tracker_id, Some(3));

    // Database check
    let query_response = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE car_id = 3")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch car");
    assert_eq!(query_response.car_id, 3);
    assert_eq!(query_response.name, "Car 3.3");
    assert_eq!(query_response.police_number, "JKL101112");
    assert_eq!(query_response.active, false);
    assert_eq!(query_response.car_type_id, 2);
    assert_eq!(query_response.tracker_id, Some(3));

    handle.abort();
}

#[sqlx::test]
async fn test_update_car_with_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/cars/1", address))
        .json(&serde_json::json!({ "name": "Car 1.1", "police_number": "JKL101112", "active": false, "car_type_id": 2, "tracker_id": 3 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Car = serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
        .expect("Failed to deserialize JSON");
    assert_eq!(body.car_id, 1);
    assert_eq!(body.name, "Car 1.1");
    assert_eq!(body.police_number, "JKL101112");
    assert_eq!(body.active, false);
    assert_eq!(body.car_type_id, 2);
    assert_eq!(body.tracker_id, Some(3));

    // Database check
    let query_response = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE car_id = 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch car");
    assert_eq!(query_response.car_id, 1);
    assert_eq!(query_response.name, "Car 1.1");
    assert_eq!(query_response.police_number, "JKL101112");
    assert_eq!(query_response.active, false);
    assert_eq!(query_response.car_type_id, 2);
    assert_eq!(query_response.tracker_id, Some(3));

    handle.abort();
}

#[sqlx::test]
async fn test_update_car_with_existing_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/cars/1", address))
        .json(&serde_json::json!({ "name": "Car 1.1", "police_number": "JKL101112", "active": false, "car_type_id": 2, "tracker_id": 2 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Body check
    let body: ErrorResponse =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.message, "Tracker is already assigned");

    // Database check
    let query_response = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM cars WHERE car_id = 1 AND tracker_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch car");
    assert_eq!(
        query_response, 0,
        "Car 1 should still exist with tracker_id still 1"
    );

    handle.abort();
}

#[sqlx::test]
async fn test_delete_car_without_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .delete(format!("{}/cars/3", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Car = serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
        .expect("Failed to deserialize JSON");
    assert_eq!(body.car_id, 3);
    assert_eq!(body.name, "Car 3");
    assert_eq!(body.police_number, "GHI789");
    assert_eq!(body.active, true);
    assert_eq!(body.car_type_id, 1);
    assert_eq!(body.tracker_id, None);

    // Database check
    let query_response =
        sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE car_id = 3 AND deleted_at IS NOT NULL")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch car");
    assert_eq!(query_response.car_id, 3);
    assert_eq!(query_response.name, "Car 3");
    assert_eq!(query_response.police_number, "GHI789");
    assert_eq!(query_response.active, true);
    assert_eq!(query_response.car_type_id, 1);
    assert_eq!(query_response.tracker_id, None);

    handle.abort();
}

#[sqlx::test]
async fn test_delete_car_with_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .delete(format!("{}/cars/1", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Car = serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
        .expect("Failed to deserialize JSON");
    assert_eq!(body.car_id, 1);
    assert_eq!(body.name, "Car 1");
    assert_eq!(body.police_number, "ABC123");
    assert_eq!(body.active, true);
    assert_eq!(body.car_type_id, 1);
    assert_eq!(body.tracker_id, None);

    // Database check
    let query_response =
        sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE car_id = 1 AND deleted_at IS NOT NULL")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch car");
    assert_eq!(query_response.car_id, 1);
    assert_eq!(query_response.name, "Car 1");
    assert_eq!(query_response.police_number, "ABC123");
    assert_eq!(query_response.active, true);
    assert_eq!(query_response.car_type_id, 1);
    assert_eq!(query_response.tracker_id, None);

    handle.abort();
}
