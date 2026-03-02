mod common;

use common::TestApp;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq)]
pub struct CarType {
    pub car_type_id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq)]
pub struct CarTypeWithCount {
    pub car_type_id: i32,
    pub name: String,
    pub car_count: i64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct GetCarTypesResponse {
    pub car_types: Vec<CarTypeWithCount>,
    pub car_type_count: usize,
}

impl CarTypeWithCount {
    fn new(id: i32, name: &str, car_count: i64) -> Self {
        Self {
            car_type_id: id,
            name: name.to_string(),
            car_count,
        }
    }
}

impl CarType {
    fn new(id: i32, name: &str) -> Self {
        Self {
            car_type_id: id,
            name: name.to_string(),
        }
    }
}

async fn seed_car_types(pool: &PgPool) {
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
    seed_car_types(&pool).await;
    let app = TestApp::spawn(pool).await;

    let body: GetCarTypesResponse = app
        .get("/cars/types")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body.car_type_count, 2);
    assert_eq!(body.car_types[0], CarTypeWithCount::new(1, "Delivery", 0));
    assert_eq!(body.car_types[1], CarTypeWithCount::new(2, "Passenger", 0));
}

#[sqlx::test]
async fn test_get_car_type(pool: PgPool) {
    seed_car_types(&pool).await;
    let app = TestApp::spawn(pool).await;

    let body: CarTypeWithCount = app
        .get("/cars/types/1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body, CarTypeWithCount::new(1, "Delivery", 0));
}

#[sqlx::test]
async fn test_create_car_type(pool: PgPool) {
    seed_car_types(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .post("/cars/types")
        .json(&serde_json::json!({ "name": "Cargo" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car_type: CarType =
        sqlx::query_as("SELECT car_type_id, name FROM car_types WHERE car_type_id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(car_type, CarType::new(3, "Cargo"));
}

#[sqlx::test]
async fn test_update_car_type(pool: PgPool) {
    seed_car_types(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/cars/types/1")
        .json(&serde_json::json!({"name": "Cargo"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car_type: CarType =
        sqlx::query_as("SELECT car_type_id, name FROM car_types WHERE car_type_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(car_type, CarType::new(1, "Cargo"));
}

#[sqlx::test]
async fn test_delete_car_type(pool: PgPool) {
    seed_car_types(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app.delete("/cars/types/1").send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car_type: CarType = sqlx::query_as(
        "SELECT car_type_id, name FROM car_types WHERE car_type_id = 1 AND deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(car_type, CarType::new(1, "Delivery"));
}
