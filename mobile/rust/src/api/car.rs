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
    pub active: i32,
    pub car_type: String,
    pub tracker_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
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
            let status = response.status();
            let body_text = match response.text().await {
                Ok(t) => t,
                Err(e) => return CarResult {
                    success: false,
                    error_message: Some(format!("Failed to read response: {}", e)),
                    car: None,
                },
            };

            if status.is_success() {
                match serde_json::from_str::<Car>(&body_text) {
                    Ok(car) => CarResult {
                        success: true,
                        error_message: None,
                        car: Some(car),
                    },
                    Err(_) => CarResult {
                        success: false,
                        error_message: Some(format!("Failed to parse API response: {}", body_text)),
                        car: None,
                    },
                }
            } else {
                let msg = serde_json::from_str::<ErrorResponse>(&body_text)
                    .map(|e| e.error)
                    .unwrap_or_else(|_| format!("Request failed with status {}", status));
                CarResult {
                    success: false,
                    error_message: Some(msg),
                    car: None,
                }
            }
        }
        Err(e) => CarResult {
            success: false,
            error_message: Some(format!("Network error: {}", e)),
            car: None,
        },
    }
}
