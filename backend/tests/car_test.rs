mod common;

use common::TestApp;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq)]
pub struct Car {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub tracker_id: Option<i32>,
}

impl Car {
    fn new(
        id: i32,
        name: &str,
        police_number: &str,
        active: bool,
        car_type_id: i32,
        tracker_id: Option<i32>,
    ) -> Self {
        Self {
            car_id: id,
            name: name.to_string(),
            police_number: police_number.to_string(),
            active,
            car_type_id,
            tracker_id,
        }
    }
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq)]
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

impl CarWithTracker {
    fn new(
        id: i32,
        name: &str,
        police_number: &str,
        active: bool,
        car_type_id: i32,
        car_type_name: &str,
        tracker_id: Option<i32>,
        tracker_name: Option<&str>,
    ) -> Self {
        Self {
            car_id: id,
            name: name.to_string(),
            police_number: police_number.to_string(),
            active,
            car_type_id,
            car_type_name: car_type_name.to_string(),
            tracker_id,
            tracker_name: tracker_name.map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, FromRow, Deserialize, Serialize, PartialEq)]
pub struct GetCarsResponse {
    pub cars: Vec<CarWithTracker>,
    pub car_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

async fn seed_cars(pool: &PgPool) {
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
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool).await;

    let body: GetCarsResponse = app.get("/cars").send().await.unwrap().json().await.unwrap();
    assert_eq!(body.car_count, 3);

    assert_eq!(
        body.cars[0],
        CarWithTracker::new(
            1,
            "Car 1",
            "ABC123",
            true,
            1,
            "Delivery",
            Some(1),
            Some("Batman Tracker")
        )
    );
    assert_eq!(
        body.cars[1],
        CarWithTracker::new(
            2,
            "Car 2",
            "DEF456",
            false,
            2,
            "Passenger",
            Some(2),
            Some("Superman Tracker")
        )
    );
    assert_eq!(
        body.cars[2],
        CarWithTracker::new(3, "Car 3", "GHI789", true, 1, "Delivery", None, None)
    );
}

#[sqlx::test]
async fn test_get_car(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool).await;

    let body: CarWithTracker = app
        .get("/cars/1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(
        body,
        CarWithTracker::new(
            1,
            "Car 1",
            "ABC123",
            true,
            1,
            "Delivery",
            Some(1),
            Some("Batman Tracker")
        )
    );
}

#[sqlx::test]
async fn test_create_car_without_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .post("/cars")
        .json(&serde_json::json!({ "name": "Car 4", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": null}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 4")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(car, Car::new(4, "Car 4", "JKL101112", true, 1, None));
}

#[sqlx::test]
async fn test_create_car_with_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .post("/cars")
        .json(&serde_json::json!({ "name": "Car 4", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": Some(3)}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 4")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(car, Car::new(4, "Car 4", "JKL101112", true, 1, Some(3)));
}

#[sqlx::test]
async fn test_update_car_with_tracker_to_without_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/cars/1")
        .json(&serde_json::json!({ "name": "Car 1 Update", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": null}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(car, Car::new(1, "Car 1 Update", "JKL101112", true, 1, None));
}

#[sqlx::test]
async fn test_update_car_with_tracker_to_another_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/cars/1")
        .json(&serde_json::json!({ "name": "Car 1 Update", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": Some(3)}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        car,
        Car::new(1, "Car 1 Update", "JKL101112", true, 1, Some(3))
    );
}

#[sqlx::test]
async fn test_update_car_with_tracker_to_already_assigned_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/cars/1")
        .json(&serde_json::json!({ "name": "Car 1 Update", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": Some(2)}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);

    let car: Option<Car> = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 1")
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert_eq!(car, None);
}

#[sqlx::test]
async fn test_update_car_without_tracker_to_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/cars/3")
        .json(&serde_json::json!({ "name": "Car 3 Update", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": Some(3)}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 3")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        car,
        Car::new(3, "Car 3 Update", "JKL101112", true, 1, Some(3))
    );
}

#[sqlx::test]
async fn test_update_car_without_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/cars/3")
        .json(&serde_json::json!({ "name": "Car 3 Update", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": null}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 3")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(car, Car::new(3, "Car 3 Update", "JKL101112", true, 1, None));
}

#[sqlx::test]
async fn test_update_car_without_tracker_to_already_assigned_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app
        .put("/cars/3")
        .json(&serde_json::json!({ "name": "Car 3 Update", "police_number": "JKL101112", "active": true, "car_type_id": 1 , "tracker_id": Some(2)}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);

    let car: Option<Car> = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 3")
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert_eq!(car, None);
}

#[sqlx::test]
async fn test_delete_car_with_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app.delete("/cars/1").send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(car, Car::new(1, "Car 1", "ABC123", true, 1, None));
}

#[sqlx::test]
async fn test_delete_car_without_tracker(pool: PgPool) {
    seed_cars(&pool).await;
    let app = TestApp::spawn(pool.clone()).await;

    let response = app.delete("/cars/3").send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let car: Car = sqlx::query_as("SELECT car_id, name, police_number, active, car_type_id, tracker_id FROM cars WHERE car_id = 3")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(car, Car::new(3, "Car 3", "GHI789", true, 1, None));
}
