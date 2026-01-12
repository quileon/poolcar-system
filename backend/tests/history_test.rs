use anyhow::Context;
use chrono::NaiveDateTime;
use poolcar_backend::create_app;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use std::str::FromStr;
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct History {
    pub history_id: i32,
    pub car_id: Option<i32>,
    pub contact_id: i32,
    pub activity_id: i32,
    pub tracker_id: Option<i32>,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_latitude: Option<Decimal>,
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, FromRow, Deserialize)]
struct HistoryWithDetails {
    pub history_id: i32,
    pub car_id: Option<i32>,
    pub car_name: Option<String>,
    pub contact_id: i32,
    pub contact_name: String,
    pub activity_id: i32,
    pub activity_name: String,
    pub tracker_id: Option<i32>,
    pub tracker_name: Option<String>,
    pub finished_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_latitude: Option<Decimal>,
    pub finished_longitude: Option<Decimal>,
    pub description: Option<String>,
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
            VALUES ('Tracker 1'), ('Tracker 2'), ('Tracker 3')
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

    sqlx::query(
        r#"
            INSERT INTO contact_types (name)
            VALUES ('Supplier'), ('Consumer')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed contact_types");

    sqlx::query(
        r#"
            INSERT INTO contacts (name, latitude, longitude, contact_type_id)
            VALUES ('Contact 1', '1.0', '1.0', 1),
                   ('Contact 2', '2.0', '2.0', 2),
                   ('Contact 3', '3.0', '3.0', 1)
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed contacts");

    sqlx::query(
        r#"
            INSERT INTO activities (name)
            VALUES ('Meeting'), ('Delivery'), ('Trial T1')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed contact_types");

    sqlx::query(
        r#"
            INSERT INTO histories (car_id, contact_id, activity_id, tracker_id, finished_at, started_at, finished_latitude, finished_longitude, description)
            VALUES (1, 1, 1, 1, '2023-01-01 01:00:00', '2023-01-01 00:00:00', '1.0', '1.0', 'Meeting with supplier'),
                   (NULL, 2, 2, NULL, NULL, '2023-01-01 00:00:00', NULL, NULL, 'Delivery to consumer'),
                   (NULL, 3, 3, NULL, NULL, '2023-01-01 00:00:00', NULL, NULL, 'Trial test')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed histories");
}

#[sqlx::test]
async fn test_get_histories(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/histories", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let history_count = body["history_count"].as_u64().expect("history_count");
    assert_eq!(history_count, 3, "Expected 3 history_count");

    // Data check
    let histories: Vec<HistoryWithDetails> = serde_json::from_value(body["histories"].clone())
        .expect("Failed to deserialize history array");

    assert_eq!(histories.len(), 3, "Expected 3 histories in array");

    // history 1
    assert_eq!(
        histories[0].history_id, 1,
        "first array history_id should be 1"
    );
    assert_eq!(
        histories[0].car_id,
        Some(1),
        "first array car_id should be 1"
    );
    assert_eq!(
        histories[0].car_name,
        Some("Car 1".into()),
        "first array car_name should be Car 1"
    );
    assert_eq!(
        histories[0].contact_id, 1,
        "first array contact_id should be 1"
    );
    assert_eq!(
        histories[0].contact_name, "Contact 1",
        "first array contact_name should be Contact 1"
    );
    assert_eq!(
        histories[0].activity_id, 1,
        "first array activity_id should be 1"
    );
    assert_eq!(
        histories[0].activity_name, "Meeting",
        "first array activity_name should be Meeting"
    );
    assert_eq!(
        histories[0].tracker_id,
        Some(1),
        "first array tracker_id should be 1"
    );
    assert_eq!(
        histories[0].tracker_name,
        Some("Tracker 1".into()),
        "first array tracker_name should be Tracker 1"
    );
    assert_eq!(
        histories[0].started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "first array started_at should be 2023-01-01T00:00:00Z"
    );
    assert_eq!(
        histories[0].finished_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T01:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "first array finished_at should be 2023-01-01T01:00:00Z"
    );
    assert_eq!(
        histories[0].finished_latitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "first array finished_latitude should be 1.0"
    );
    assert_eq!(
        histories[0].finished_longitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "first array finished_longitude should be 1.0"
    );
    assert_eq!(
        histories[0].description,
        Some("Meeting with supplier".to_string()),
        "first array description should be Meeting with supplier"
    );

    // history 2
    assert_eq!(
        histories[1].history_id, 2,
        "second array history_id should be 2"
    );
    assert_eq!(
        histories[1].car_id, None,
        "second array car_id should be None"
    );
    assert_eq!(
        histories[1].car_name, None,
        "second array car_name should be None"
    );
    assert_eq!(
        histories[1].contact_id, 2,
        "second array contact_id should be 2"
    );
    assert_eq!(
        histories[1].contact_name, "Contact 2",
        "second array contact_name should be Contact 2"
    );
    assert_eq!(
        histories[1].activity_id, 2,
        "second array activity_id should be 2"
    );
    assert_eq!(
        histories[1].activity_name, "Delivery",
        "second array activity_name should be Call"
    );
    assert_eq!(
        histories[1].tracker_id, None,
        "second array tracker_id should be None"
    );
    assert_eq!(
        histories[1].tracker_name, None,
        "second array tracker_name should be None"
    );
    assert_eq!(
        histories[1].started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "second array started_at should be 2023-01-01T00:00:00Z"
    );
    assert_eq!(
        histories[1].finished_at, None,
        "second array finished_at should be None"
    );
    assert_eq!(
        histories[1].finished_latitude, None,
        "second array finished_latitude should be None"
    );
    assert_eq!(
        histories[1].finished_longitude, None,
        "second array finished_longitude should be None"
    );
    assert_eq!(
        histories[1].description,
        Some("Delivery to consumer".to_string()),
        "first array description should be Delivery to consumer"
    );

    // history 3
    assert_eq!(
        histories[2].history_id, 3,
        "third array history_id should be 3"
    );
    assert_eq!(
        histories[2].car_id, None,
        "third array car_id should be None"
    );
    assert_eq!(
        histories[2].car_name, None,
        "third array car_name should be None"
    );
    assert_eq!(
        histories[2].contact_id, 3,
        "third array contact_id should be 3"
    );
    assert_eq!(
        histories[2].contact_name, "Contact 3",
        "third array contact_name should be Contact 3"
    );
    assert_eq!(
        histories[2].activity_id, 3,
        "third array activity_id should be 3"
    );
    assert_eq!(
        histories[2].activity_name, "Trial T1",
        "third array activity_name should be Trial T1"
    );
    assert_eq!(
        histories[2].tracker_id, None,
        "third array tracker_id should be None"
    );
    assert_eq!(
        histories[2].tracker_name, None,
        "third array tracker_name should be None"
    );
    assert_eq!(
        histories[2].started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "third array started_at should be 2023-01-01T00:00:00Z"
    );
    assert_eq!(
        histories[2].finished_at, None,
        "third array finished_at should be None"
    );
    assert_eq!(
        histories[2].finished_latitude, None,
        "third array finished_latitude should be None"
    );
    assert_eq!(
        histories[2].finished_longitude, None,
        "third array finished_longitude should be None"
    );
    assert_eq!(
        histories[2].description,
        Some("Trial test".to_string()),
        "first array description should be Trial test"
    );

    handle.abort();
}

#[sqlx::test]
async fn test_create_noncomplete_history(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/histories", address))
        .json(&serde_json::json!({
            "car_id": null,
            "contact_id": 1,
            "activity_id": 1,
            "tracker_id": null,
            "finished_at": null,
            "started_at": "2023-01-01T00:00:00",
            "finished_latitude": null,
            "finished_longitude": null,
            "description": null,
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response status should be 200"
    );

    // Body check
    let body: History =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.history_id, 4, "history_id should be 4");
    assert_eq!(body.car_id, None, "car_id should be None");
    assert_eq!(body.contact_id, 1, "contact_id should be 1");
    assert_eq!(body.activity_id, 1, "activity_id should be 1");
    assert_eq!(body.tracker_id, None, "tracker_id should be None");
    assert_eq!(
        body.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(body.finished_at, None, "finished_at should be None");
    assert_eq!(
        body.finished_latitude, None,
        "finished_latitude should be None"
    );
    assert_eq!(
        body.finished_longitude, None,
        "finished_longitude should be None"
    );
    assert_eq!(body.description, None, "description should be None");

    // Database check
    let query_response =
        sqlx::query_as::<_, History>("SELECT * FROM histories WHERE history_id = 4")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch history");
    assert_eq!(query_response.history_id, 4, "history_id should be 4");
    assert_eq!(query_response.car_id, None, "car_id should be None");
    assert_eq!(query_response.contact_id, 1, "contact_id should be 1");
    assert_eq!(query_response.activity_id, 1, "activity_id should be 1");
    assert_eq!(query_response.tracker_id, None, "tracker_id should be None");
    assert_eq!(
        query_response.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        query_response.finished_at, None,
        "finished_at should be None"
    );
    assert_eq!(
        query_response.finished_latitude, None,
        "finished_latitude should be None"
    );
    assert_eq!(
        query_response.finished_longitude, None,
        "finished_longitude should be None"
    );
    assert_eq!(
        query_response.description, None,
        "description should be None"
    );

    handle.abort();
}

#[sqlx::test]
async fn test_create_complete_history(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/histories", address))
        .json(&serde_json::json!({
            "car_id": 1,
            "contact_id": 1,
            "activity_id": 1,
            "tracker_id": 1,
            "started_at": "2023-01-01T00:00:00",
            "finished_at": "2023-01-01T00:00:00",
            "finished_latitude": 1.0,
            "finished_longitude": 1.0,
            "description": "Ini deskripsi",
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response status should be 200"
    );

    // Body check
    let body: History =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.history_id, 4, "history_id should be 4");
    assert_eq!(body.car_id, Some(1), "car_id should be 1");
    assert_eq!(body.contact_id, 1, "contact_id should be 1");
    assert_eq!(body.activity_id, 1, "activity_id should be 1");
    assert_eq!(body.tracker_id, Some(1), "tracker_id should be 1");
    assert_eq!(
        body.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        body.finished_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "finished_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        body.finished_latitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_latitude should be 1.0"
    );
    assert_eq!(
        body.finished_longitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_longitude should be 1.0"
    );
    assert_eq!(
        body.description,
        Some("Ini deskripsi".into()),
        "description should be Ini deskripsi"
    );

    // Database check
    let query_response =
        sqlx::query_as::<_, History>("SELECT * FROM histories WHERE history_id = 4")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch history");
    assert_eq!(query_response.history_id, 4, "history_id should be 4");
    assert_eq!(query_response.car_id, Some(1), "car_id should be None");
    assert_eq!(query_response.contact_id, 1, "contact_id should be 1");
    assert_eq!(query_response.activity_id, 1, "activity_id should be 1");
    assert_eq!(query_response.tracker_id, Some(1), "tracker_id should be 1");
    assert_eq!(
        query_response.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        query_response.finished_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "finished_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        query_response.finished_latitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_latitude should be 1.0"
    );
    assert_eq!(
        query_response.finished_longitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_longitude should be 1.0"
    );
    assert_eq!(
        query_response.description,
        Some("Ini deskripsi".into()),
        "description should be Ini deskripsi"
    );

    handle.abort();
}

#[sqlx::test]
async fn test_update_history_to_complete(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/histories/3", address))
        .json(&serde_json::json!({
            "car_id": 1,
            "contact_id": 1,
            "activity_id": 1,
            "tracker_id": 1,
            "started_at": "2023-01-01T00:00:00",
            "finished_at": "2023-01-01T00:00:00",
            "finished_latitude": 1.0,
            "finished_longitude": 1.0,
            "description": "Ini deskripsi",
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response status should be 200"
    );

    // Body check
    let body: History =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.history_id, 3, "history_id should be 3");
    assert_eq!(body.car_id, Some(1), "car_id should be 1");
    assert_eq!(body.contact_id, 1, "contact_id should be 1");
    assert_eq!(body.activity_id, 1, "activity_id should be 1");
    assert_eq!(body.tracker_id, Some(1), "tracker_id should be 1");
    assert_eq!(
        body.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        body.finished_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "finished_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        body.finished_latitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_latitude should be 1.0"
    );
    assert_eq!(
        body.finished_longitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_longitude should be 1.0"
    );
    assert_eq!(
        body.description,
        Some("Ini deskripsi".into()),
        "description should be Ini deskripsi"
    );

    // Database check
    let query_response =
        sqlx::query_as::<_, History>("SELECT * FROM histories WHERE history_id = 3")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch history");
    assert_eq!(query_response.history_id, 3, "history_id should be 3");
    assert_eq!(query_response.car_id, Some(1), "car_id should be None");
    assert_eq!(query_response.contact_id, 1, "contact_id should be 1");
    assert_eq!(query_response.activity_id, 1, "activity_id should be 1");
    assert_eq!(query_response.tracker_id, Some(1), "tracker_id should be 1");
    assert_eq!(
        query_response.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        query_response.finished_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "finished_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        query_response.finished_latitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_latitude should be 1.0"
    );
    assert_eq!(
        query_response.finished_longitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_longitude should be 1.0"
    );
    assert_eq!(
        query_response.description,
        Some("Ini deskripsi".into()),
        "description should be Ini deskripsi"
    );

    handle.abort();
}

#[sqlx::test]
async fn test_update_history_to_uncomplete(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/histories/1", address))
        .json(&serde_json::json!({
            "car_id": null,
            "contact_id": 1,
            "activity_id": 1,
            "tracker_id": null,
            "finished_at": null,
            "started_at": "2023-01-01T00:00:00",
            "finished_latitude": null,
            "finished_longitude": null,
            "description": null,
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response status should be 200"
    );

    // Body check
    let body: History =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.history_id, 1, "history_id should be 4");
    assert_eq!(body.car_id, None, "car_id should be None");
    assert_eq!(body.contact_id, 1, "contact_id should be 1");
    assert_eq!(body.activity_id, 1, "activity_id should be 1");
    assert_eq!(body.tracker_id, None, "tracker_id should be None");
    assert_eq!(
        body.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(body.finished_at, None, "finished_at should be None");
    assert_eq!(
        body.finished_latitude, None,
        "finished_latitude should be None"
    );
    assert_eq!(
        body.finished_longitude, None,
        "finished_longitude should be None"
    );
    assert_eq!(body.description, None, "description should be None");

    // Database check
    let query_response =
        sqlx::query_as::<_, History>("SELECT * FROM histories WHERE history_id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch history");
    assert_eq!(query_response.history_id, 1, "history_id should be 1");
    assert_eq!(query_response.car_id, None, "car_id should be None");
    assert_eq!(query_response.contact_id, 1, "contact_id should be 1");
    assert_eq!(query_response.activity_id, 1, "activity_id should be 1");
    assert_eq!(query_response.tracker_id, None, "tracker_id should be None");
    assert_eq!(
        query_response.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        query_response.finished_at, None,
        "finished_at should be None"
    );
    assert_eq!(
        query_response.finished_latitude, None,
        "finished_latitude should be None"
    );
    assert_eq!(
        query_response.finished_longitude, None,
        "finished_longitude should be None"
    );
    assert_eq!(
        query_response.description, None,
        "description should be None"
    );

    handle.abort();
}

#[sqlx::test]
async fn delete_history(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .delete(format!("{}/histories/1", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response status should be 200"
    );

    // Body check
    let body: History =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.history_id, 1, "history_id should be 1");
    assert_eq!(body.car_id, Some(1), "car_id should be 1");
    assert_eq!(body.contact_id, 1, "contact_id should be 1");
    assert_eq!(body.activity_id, 1, "activity_id should be 1");
    assert_eq!(body.tracker_id, Some(1), "tracker_id should be 1");
    assert_eq!(
        body.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        body.finished_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T01:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "finished_at should be 2023-01-01T01:00:00"
    );
    assert_eq!(
        body.finished_latitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_latitude should be 1.0"
    );
    assert_eq!(
        body.finished_longitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_longitude should be 1.0"
    );
    assert_eq!(
        body.description,
        Some("Meeting with supplier".into()),
        "description should be Ini deskripsi"
    );

    // Database check
    let query_response = sqlx::query_as::<_, History>(
        "SELECT * FROM histories WHERE history_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch history");
    assert_eq!(query_response.history_id, 1, "history_id should be 1");
    assert_eq!(query_response.car_id, Some(1), "car_id should be None");
    assert_eq!(query_response.contact_id, 1, "contact_id should be 1");
    assert_eq!(query_response.activity_id, 1, "activity_id should be 1");
    assert_eq!(query_response.tracker_id, Some(1), "tracker_id should be 1");
    assert_eq!(
        query_response.started_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "started_at should be 2023-01-01T00:00:00"
    );
    assert_eq!(
        query_response.finished_at,
        Some(
            chrono::NaiveDateTime::parse_from_str("2023-01-01T01:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        ),
        "finished_at should be 2023-01-01T01:00:00"
    );
    assert_eq!(
        query_response.finished_latitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_latitude should be 1.0"
    );
    assert_eq!(
        query_response.finished_longitude,
        Some(Decimal::from_str("1.0").unwrap()),
        "finished_longitude should be 1.0"
    );
    assert_eq!(
        query_response.description,
        Some("Meeting with supplier".into()),
        "description should be Ini deskripsi"
    );

    handle.abort();
}
