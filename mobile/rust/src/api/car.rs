use reqwest::Client;
use serde::{Deserialize, Serialize};
use reqwest::header::AUTHORIZATION;
use crate::api::BASE_URL;

pub struct CarResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub car: Option<Car>,
}

#[derive(Serialize, Deserialize)]
pub struct Car {
    pub car_id: i32,
    pub name: String,
    pub police_number: String,
    pub active: bool,
    pub car_type_id: i32,
    pub car_type_name: String,
    pub tracker_id: Option<i32>,
    pub tracker_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Deserialize)]
struct CarResponse {
    pub car: Option<Car>,
    pub message: Option<String>,
    pub status: Option<String>,
}

pub async fn get_car_by_id(token: String, car_id: i32) -> CarResult {
    let client_result = Client::builder()
        .danger_accept_invalid_certs(true)
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            return CarResult {
                success: false,
                error_message: Some(format!("Failed to build client: {}", e)),
                car: None,
            }
        }
    };

    let auth_header = format!("Bearer {}", token);
    let url = format!("{}/cars/{}", BASE_URL, car_id);

    let res = client
        .get(&url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await;

    match res {
        Ok(response) => {
            let text = match response.text().await {
                Ok(t) => t,
                Err(_) => return CarResult {
                    success: false,
                    error_message: Some("Failed to read response body".to_string()),
                    car: None,
                }
            };
            
            // Try parsing as Car directly (if the backend returns the car object as the root JSON)
            if let Ok(car) = serde_json::from_str::<Car>(&text) {
                return CarResult {
                    success: true,
                    error_message: None,
                    car: Some(car),
                };
            }
            
            // Try parsing as a wrapped response (e.g. { "status": "...", "car": { ... } })
            if let Ok(json) = serde_json::from_str::<CarResponse>(&text) {
                if json.status.as_deref() == Some("error") {
                    return CarResult {
                        success: false,
                        error_message: Some(json.message.unwrap_or_else(|| "Unknown error".to_string())),
                        car: None,
                    };
                }

                if let Some(car) = json.car {
                    return CarResult {
                        success: true,
                        error_message: None,
                        car: Some(car),
                    };
                }
            }

            CarResult {
                success: false,
                error_message: Some(format!("Failed to parse API response: {}", text)),
                car: None,
            }
        }
        Err(e) => CarResult {
            success: false,
            error_message: Some(format!("Network error: {}", e)),
            car: None,
        },
    }
}
