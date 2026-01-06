use poolcar_tracking_system_backend_test::create_app;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Tracker {
    pub tracker_id: i32,
    pub name: String,
}

async fn spawn_app(pool: PgPool) -> (String, JoinHandle<()>) {
    let app = create_app(pool);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (address, handle)
}

async fn seed_trackers(pool: &PgPool) {
    sqlx::query(
        r#"
            INSERT INTO trackers (name)
            VALUES ('Batman Tracker'), ('Superman Tracker'), ('Robin Tracker')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed trackers");
}

#[sqlx::test]
async fn test_get_trackers(pool: PgPool) {
    seed_trackers(&pool).await;
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
    let trackers = body["trackers"]
        .as_array()
        .expect("trackers should be an array");
    assert_eq!(trackers.len(), 3, "Expected 3 trackers in array");

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
        1
    );

    // Database check
    let tracker = sqlx::query_as::<_, Tracker>("SELECT * FROM trackers WHERE tracker_id = 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch tracker");
    assert_eq!(tracker.tracker_id, 1);
    assert_eq!(tracker.name, "Created Tracker");

    handle.abort();
}

#[sqlx::test]
async fn test_update_tracker(pool: PgPool) {
    seed_trackers(&pool).await;
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
    seed_trackers(&pool).await;
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
    assert_eq!(body["name"], "Batman Tracker");
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
    assert_eq!(tracker.name, "Batman Tracker");

    handle.abort();
}
