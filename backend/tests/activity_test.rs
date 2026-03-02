use anyhow::Context;
use poolcar_backend::{config::Config, create_app};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Activity {
    pub activity_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct ActivityWithCount {
    pub activity_id: i32,
    pub name: String,
    pub activity_count: i32,
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

    let app = create_app(db_pool, redis_pool, None, Config::from_env().unwrap());

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
            INSERT INTO activities (name)
            VALUES ('Meeting'), ('Delivery'), ('Trial T1')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed contact_types");
}

#[sqlx::test]
async fn test_get_activities(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/activities", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let activity_count = body["activity_count"].as_u64().expect("activity_count");
    assert_eq!(activity_count, 3, "Expected 3 activity_count");

    // Data check
    let activities: Vec<ActivityWithCount> = serde_json::from_value(body["activities"].clone())
        .expect("Failed to deserialize activity array");

    assert_eq!(activities.len(), 3, "Expected 3 activities in array");

    // Activity 1
    assert_eq!(
        activities[0].activity_id, 1,
        "first array activity_id should be 1"
    );
    assert_eq!(
        activities[0].name, "Meeting",
        "first array name should be Meeting"
    );
    assert_eq!(
        activities[0].activity_count, 0,
        "first array activity_count should be 0"
    );

    // Activity 2
    assert_eq!(
        activities[1].activity_id, 2,
        "second array activity_id should be 2"
    );
    assert_eq!(
        activities[1].name, "Delivery",
        "second array name should be Delivery"
    );
    assert_eq!(
        activities[1].activity_count, 0,
        "second array activity_count should be 0"
    );

    // Activity 3
    assert_eq!(
        activities[2].activity_id, 3,
        "third array activity_id should be 3"
    );
    assert_eq!(
        activities[2].name, "Trial T1",
        "third array name should be Trial T1"
    );
    assert_eq!(
        activities[2].activity_count, 0,
        "third array activity_count should be 0"
    );

    handle.abort();
}

#[sqlx::test]
async fn test_create_activity(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/activities", address))
        .json(&serde_json::json!({ "name": "Activity 4" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Activity =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.activity_id, 4);
    assert_eq!(body.name, "Activity 4");

    // Database check
    let query_response =
        sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE activity_id = 4")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch activity");
    assert_eq!(query_response.activity_id, 4);
    assert_eq!(query_response.name, "Activity 4");

    handle.abort();
}

#[sqlx::test]
async fn test_update_activity(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .put(format!("{}/activities/1", address))
        .json(&serde_json::json!({ "name": "Activity 1" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Activity =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.activity_id, 1);
    assert_eq!(body.name, "Activity 1");

    // Database check
    let query_response =
        sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE activity_id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch activity");
    assert_eq!(query_response.activity_id, 1);
    assert_eq!(query_response.name, "Activity 1");

    handle.abort();
}

#[sqlx::test]
async fn test_delete_activity(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .delete(format!("{}/activities/1", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Activity =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.activity_id, 1);
    assert_eq!(body.name, "Meeting");

    // Database check
    let query_response = sqlx::query_as::<_, Activity>(
        "SELECT * FROM activities WHERE activity_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch activity");
    assert_eq!(query_response.activity_id, 1);
    assert_eq!(query_response.name, "Meeting");

    handle.abort();
}
