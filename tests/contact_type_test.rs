use poolcar_tracking_system_backend_test::create_app;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct ContactType {
    pub contact_type_id: i32,
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

async fn seed_contact_types(pool: &PgPool) {
    sqlx::query(
        r#"
            INSERT INTO contact_types (name)
            VALUES ('Supplier'), ('Consumer')
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed contact types");
}

#[sqlx::test]
async fn test_get_contact_types(pool: PgPool) {
    seed_contact_types(&pool).await;
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/contacts/types", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let contact_type_count = body["contact_type_count"]
        .as_u64()
        .expect("contact_type_count should be a number");
    assert_eq!(contact_type_count, 2, "Expected 2 contact_type_count");
    let contact_types = body["contact_types"]
        .as_array()
        .expect("contact_types should be an array");
    assert_eq!(contact_types.len(), 2, "Expected 2 contact_types in array");

    handle.abort();
}

#[sqlx::test]
async fn test_create_contact_type(pool: PgPool) {
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/contacts/types", address))
        .json(&serde_json::json!({ "name": "Investor" }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Investor");
    assert_eq!(
        body["contact_type_id"]
            .as_i64()
            .expect("contact_type_id should be a number"),
        1,
    );

    // Database check
    let query_response =
        sqlx::query_as::<_, ContactType>("SELECT * FROM contact_types WHERE contact_type_id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch car_type");
    assert_eq!(query_response.contact_type_id, 1);
    assert_eq!(query_response.name, "Investor");

    handle.abort();
}

#[sqlx::test]
async fn test_update_car_type(pool: PgPool) {
    seed_contact_types(&pool).await;
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .put(format!("{}/contacts/types/1", address))
        .json(&serde_json::json!({
            "name": "Investor"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Investor");
    assert_eq!(
        body["contact_type_id"]
            .as_u64()
            .expect("contact_type_id should be a number"),
        1
    );

    // Database check
    let query_response =
        sqlx::query_as::<_, ContactType>("SELECT * FROM contact_types WHERE contact_type_id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch contact_type");
    assert_eq!(query_response.contact_type_id, 1);
    assert_eq!(query_response.name, "Investor");

    handle.abort();
}

#[sqlx::test]
async fn test_delete_car_type(pool: PgPool) {
    seed_contact_types(&pool).await;
    let (address, handle) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    // Curl
    let response = client
        .delete(format!("{}/contacts/types/1", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Response check
    assert_eq!(response.status().as_u16(), 200);

    // Body check
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "Supplier");
    assert_eq!(
        body["contact_type_id"]
            .as_u64()
            .expect("contact_type_id should be a number"),
        1
    );

    // Database check
    let query_response = sqlx::query_as::<_, ContactType>(
        "SELECT * FROM contact_types WHERE contact_type_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch contact_type");
    assert_eq!(query_response.contact_type_id, 1);
    assert_eq!(query_response.name, "Supplier");

    handle.abort();
}
