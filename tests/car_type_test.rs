use poolcar_tracking_system_backend_test::create_app;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct CarType {
    pub car_type_id: i32,
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

async fn seed_car_types(pool: &PgPool) {
    sqlx::query!(
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
    seed_car_types(&pool).await;
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
    assert_eq!(car_type_count, 2, "Expected 2 trackers");
    let car_types = body["car_types"]
        .as_array()
        .expect("trackers should be an array");
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
        1,
    );

    // Database check
    let car_type = sqlx::query_as::<_, CarType>("SELECT * FROM car_types WHERE car_type_id = 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch tracker");
    assert_eq!(car_type.car_type_id, 1);
    assert_eq!(car_type.name, "Cargo");

    handle.abort();
}
