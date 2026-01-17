use anyhow::Context;
use poolcar_backend::create_app;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Tracker {
    pub tracker_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct TrackerWithDetails {
    pub tracker_id: i32,
    pub name: String,
    pub car_id: Option<i32>,
    pub car_name: Option<String>,
    pub car_type_id: Option<i32>,
    pub car_type_name: Option<String>,
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
            INSERT INTO trackers (name)
            VALUES ('Tracker 1'), ('Tracker 2'), ('Tracker 3')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed trackers");
}

#[sqlx::test]
async fn test_get_trackers(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/trackers", address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let tracker_count = body["tracker_count"]
        .as_u64()
        .expect("tracker_count should be a number");
    assert_eq!(tracker_count, 3, "Expected 3 trackers");
    let trackers: Vec<TrackerWithDetails> = serde_json::from_value(body["trackers"].clone())
        .expect("Failed to deserialize trackers array");
    assert_eq!(trackers.len(), 3, "Expected 3 trackers in array");

    // Tracker 1 check
    assert_eq!(trackers[0].tracker_id, 1);
    assert_eq!(trackers[0].name, "Tracker 1");
    assert_eq!(trackers[0].car_id, None);
    assert_eq!(trackers[0].car_name, None);
    assert_eq!(trackers[0].car_type_id, None);
    assert_eq!(trackers[0].car_type_name, None);

    // Tracker 2 check
    assert_eq!(trackers[1].tracker_id, 2);
    assert_eq!(trackers[1].name, "Tracker 2");
    assert_eq!(trackers[1].car_id, None);
    assert_eq!(trackers[1].car_name, None);
    assert_eq!(trackers[1].car_type_id, None);
    assert_eq!(trackers[1].car_type_name, None);

    // Tracker 3 check
    assert_eq!(trackers[2].tracker_id, 3);
    assert_eq!(trackers[2].name, "Tracker 3");
    assert_eq!(trackers[2].car_id, None);
    assert_eq!(trackers[2].car_name, None);
    assert_eq!(trackers[2].car_type_id, None);
    assert_eq!(trackers[2].car_type_name, None);

    handle.abort();
}

#[sqlx::test]
async fn test_create_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .post(format!("{}/trackers", address))
        .json(&serde_json::json!({
            "name": "Created Tracker"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Created Tracker");
    assert_eq!(
        body["tracker_id"]
            .as_u64()
            .expect("tracker_id should be a number"),
        4
    );

    // Database check
    let tracker = sqlx::query_as::<_, Tracker>("SELECT * FROM trackers WHERE tracker_id = 4")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch tracker");
    assert_eq!(tracker.tracker_id, 4);
    assert_eq!(tracker.name, "Created Tracker");

    handle.abort();
}

#[sqlx::test]
async fn test_update_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/trackers/1", address))
        .json(&serde_json::json!({
            "name": "Updated Tracker"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Updated Tracker");
    assert_eq!(
        body["tracker_id"]
            .as_u64()
            .expect("tracker_id should be a number"),
        1
    );

    // Database check
    let tracker = sqlx::query_as::<_, Tracker>("SELECT * FROM trackers WHERE tracker_id = 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch tracker");
    assert_eq!(tracker.tracker_id, 1);
    assert_eq!(tracker.name, "Updated Tracker");

    handle.abort();
}

#[sqlx::test]
async fn test_delete_tracker(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .delete(format!("{}/trackers/1", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Tracker 1");
    assert_eq!(
        body["tracker_id"]
            .as_u64()
            .expect("tracker_id should be a number"),
        1
    );

    // Database check
    let tracker = sqlx::query_as::<_, Tracker>(
        "SELECT * FROM trackers WHERE tracker_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch tracker");
    assert_eq!(tracker.tracker_id, 1);
    assert_eq!(tracker.name, "Tracker 1");

    handle.abort();
}
