mod common;

use common::TestApp;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, MySqlPool};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Tracker {
    pub tracker_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize, PartialEq)]
pub struct TrackerWithDetails {
    pub tracker_id: i32,
    pub name: String,
    pub car_id: Option<i32>,
    pub car_name: Option<String>,
    pub car_type_id: Option<i32>,
    pub car_type_name: Option<String>,
}

impl TrackerWithDetails {
    fn new(id: i32, name: &str) -> Self {
        Self {
            tracker_id: id,
            name: name.to_string(),
            car_id: None,
            car_name: None,
            car_type_id: None,
            car_type_name: None,
        }
    }
}

async fn seed_trackers(pool: &MySqlPool) {
    sqlx::query(
        r#"INSERT INTO trackers (name) VALUES ('Tracker 1'), ('Tracker 2'), ('Tracker 3')"#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed trackers");
}

#[sqlx::test]
async fn test_get_trackers(pool: MySqlPool) {
    seed_trackers(&pool).await;
    let app = TestApp::spawn(pool).await;

    let body: serde_json::Value = app
        .get("/trackers")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["tracker_count"], 3);

    let trackers: Vec<TrackerWithDetails> =
        serde_json::from_value(body["trackers"].clone()).unwrap();

    assert_eq!(trackers[0], TrackerWithDetails::new(1, "Tracker 1"));
    assert_eq!(trackers[1], TrackerWithDetails::new(2, "Tracker 2"));
    assert_eq!(trackers[2], TrackerWithDetails::new(3, "Tracker 3"));
}

#[sqlx::test]
async fn test_get_tracker(pool: MySqlPool) {
    seed_trackers(&pool).await;
    let app = TestApp::spawn(pool).await;

    let body: TrackerWithDetails = app
        .get("/trackers/1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body, TrackerWithDetails::new(1, "Tracker 1"));
}

#[sqlx::test]
async fn test_create_tracker(pool: MySqlPool) {
    seed_trackers(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .post("/trackers")
        .json(&serde_json::json!({"name": "Created Tracker"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let tracker: Tracker =
        sqlx::query_as("SELECT tracker_id, name FROM trackers WHERE tracker_id = 4")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(tracker.name, "Created Tracker");
}

#[sqlx::test]
async fn test_update_tracker(pool: MySqlPool) {
    seed_trackers(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/trackers/1")
        .json(&serde_json::json!({"name": "Updated Tracker"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let tracker: Tracker =
        sqlx::query_as("SELECT tracker_id, name FROM trackers WHERE tracker_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(tracker.name, "Updated Tracker");
}

#[sqlx::test]
async fn test_delete_tracker(pool: MySqlPool) {
    seed_trackers(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app.delete("/trackers/1").send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let tracker: Tracker = sqlx::query_as(
        "SELECT tracker_id, name FROM trackers WHERE tracker_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tracker.name, "Tracker 1");
}
