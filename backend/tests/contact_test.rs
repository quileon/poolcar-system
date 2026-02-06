use anyhow::Context;
use poolcar_backend::{config::Config, create_app};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use std::str::FromStr;
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Contact {
    pub contact_id: i32,
    pub name: String,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub contact_type_id: i32,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct ContactWithContactType {
    pub contact_id: i32,
    pub name: String,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub contact_type_id: i32,
    pub contact_type_name: String,
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
}

#[sqlx::test]
async fn test_get_contacts(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/contacts", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let contact_count = body["contact_count"]
        .as_u64()
        .expect("contact_count should be a number");
    assert_eq!(contact_count, 3, "Expected 3 contact_count");

    // Data check
    let contacts: Vec<ContactWithContactType> = serde_json::from_value(body["contacts"].clone())
        .expect("Failed to deserialize contacts array");

    assert_eq!(contacts.len(), 3, "Expected 3 contacts in array");

    // Contact 1
    assert_eq!(contacts[0].contact_id, 1);
    assert_eq!(contacts[0].name, "Contact 1");
    assert_eq!(contacts[0].latitude, Decimal::from_str("1.0").unwrap());
    assert_eq!(contacts[0].longitude, Decimal::from_str("1.0").unwrap());
    assert_eq!(contacts[0].contact_type_id, 1);
    assert_eq!(contacts[0].contact_type_name, "Supplier");

    // Contact 2
    assert_eq!(contacts[1].contact_id, 2);
    assert_eq!(contacts[1].name, "Contact 2");
    assert_eq!(contacts[1].latitude, Decimal::from_str("2.0").unwrap());
    assert_eq!(contacts[1].longitude, Decimal::from_str("2.0").unwrap());
    assert_eq!(contacts[1].contact_type_id, 2);
    assert_eq!(contacts[1].contact_type_name, "Consumer");

    // Contact 3
    assert_eq!(contacts[2].contact_id, 3);
    assert_eq!(contacts[2].name, "Contact 3");
    assert_eq!(contacts[2].latitude, Decimal::from_str("3.0").unwrap());
    assert_eq!(contacts[2].longitude, Decimal::from_str("3.0").unwrap());
    assert_eq!(contacts[2].contact_type_id, 1);
    assert_eq!(contacts[2].contact_type_name, "Supplier");

    handle.abort();
}

#[sqlx::test]
async fn test_create_contact_string_coordinates(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/contacts", address))
        .json(&serde_json::json!({ "name": "Contact 4", "latitude": "4.1234567890", "longitude": "4.1234567890", "contact_type_id": 1 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Contact =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.contact_id, 4);
    assert_eq!(body.name, "Contact 4");
    assert_eq!(body.latitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.longitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.contact_type_id, 1);

    // Database check
    let query_response =
        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE contact_id = 4")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch contact");
    assert_eq!(query_response.contact_id, 4);
    assert_eq!(query_response.name, "Contact 4");
    assert_eq!(
        query_response.latitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(
        query_response.longitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(query_response.contact_type_id, 1);

    handle.abort();
}

#[sqlx::test]
async fn test_create_contact_float_coordinates(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/contacts", address))
        .json(&serde_json::json!({ "name": "Contact 4", "latitude": 4.1234567890, "longitude": 4.1234567890, "contact_type_id": 1 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Contact =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.contact_id, 4);
    assert_eq!(body.name, "Contact 4");
    assert_eq!(body.latitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.longitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.contact_type_id, 1);

    // Database check
    let query_response =
        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE contact_id = 4")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch contact");
    assert_eq!(query_response.contact_id, 4);
    assert_eq!(query_response.name, "Contact 4");
    assert_eq!(
        query_response.latitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(
        query_response.longitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(query_response.contact_type_id, 1);

    handle.abort();
}

#[sqlx::test]
async fn test_update_contact_string_coordinates(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/contacts/1", address))
        .json(&serde_json::json!({ "name": "Contact 1.1", "latitude": "4.1234567890", "longitude": "4.1234567890", "contact_type_id": 1 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Contact =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.contact_id, 1);
    assert_eq!(body.name, "Contact 1.1");
    assert_eq!(body.latitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.longitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.contact_type_id, 1);

    // Database check
    let query_response =
        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE contact_id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch contact");
    assert_eq!(query_response.contact_id, 1);
    assert_eq!(query_response.name, "Contact 1.1");
    assert_eq!(
        query_response.latitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(
        query_response.longitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(query_response.contact_type_id, 1);

    handle.abort();
}

#[sqlx::test]
async fn test_update_contact_float_coordinates(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/contacts/1", address))
        .json(&serde_json::json!({ "name": "Contact 1.1", "latitude": 4.1234567890, "longitude": 4.1234567890, "contact_type_id": 1 }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Contact =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.contact_id, 1);
    assert_eq!(body.name, "Contact 1.1");
    assert_eq!(body.latitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.longitude, Decimal::from_str("4.12345679").unwrap());
    assert_eq!(body.contact_type_id, 1);

    // Database check
    let query_response =
        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE contact_id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch contact");
    assert_eq!(query_response.contact_id, 1);
    assert_eq!(query_response.name, "Contact 1.1");
    assert_eq!(
        query_response.latitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(
        query_response.longitude,
        Decimal::from_str("4.12345679").unwrap()
    );
    assert_eq!(query_response.contact_type_id, 1);

    handle.abort();
}

#[sqlx::test]
async fn test_delete_contact(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .delete(format!("{}/contacts/1", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: Contact =
        serde_json::from_value(response.json().await.expect("Failed to parse JSON"))
            .expect("Failed to deserialize JSON");
    assert_eq!(body.contact_id, 1);
    assert_eq!(body.name, "Contact 1");
    assert_eq!(body.latitude, Decimal::from_str("1.0").unwrap());
    assert_eq!(body.longitude, Decimal::from_str("1.0").unwrap());
    assert_eq!(body.contact_type_id, 1);

    // Database check
    let query_response = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE contact_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch contact");
    assert_eq!(query_response.contact_id, 1);
    assert_eq!(query_response.name, "Contact 1");
    assert_eq!(query_response.latitude, Decimal::from_str("1.0").unwrap());
    assert_eq!(query_response.longitude, Decimal::from_str("1.0").unwrap());
    assert_eq!(query_response.contact_type_id, 1);

    handle.abort();
}
