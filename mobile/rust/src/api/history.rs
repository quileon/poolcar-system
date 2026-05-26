use reqwest::Client;
use serde::{Deserialize, Serialize};
use reqwest::header::AUTHORIZATION;
use const_format::concatcp;
use crate::api::BASE_URL;

pub struct HistoryResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub items: Option<Vec<CarStatus>>,
}

#[derive(Serialize, Deserialize)]
pub struct CarStatus {
    pub car_status_id: i32,
    pub car_id: i32,
    pub car_name: String,
    pub car_police_number: String,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: String,
    pub recorded_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Serialize)]
pub struct CarStatusBody {
    pub car_id: i32,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: String,
}

#[derive(Deserialize)]
struct HistoryResponse {
    car_statuses: Option<Vec<CarStatus>>,
    #[allow(dead_code)]
    car_status_count: Option<i32>,
    message: Option<String>,
    status: Option<String>,
}


pub async fn post_history(token: String, car_id: i32, gas_level: f64, kilometres: f64, status_type: String) -> HistoryResult {
    if status_type != "DEPARTURE" && status_type != "RETURN" {
        return HistoryResult {
            success: false,
            error_message: Some("Invalid status type".to_string()),
            items: None,
        };
    }

    let client_result = Client::builder()
        .danger_accept_invalid_certs(true)
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            return HistoryResult {
                success: false,
                error_message: Some(format!("Failed to build client: {}", e)),
                items: None,
            }
        }
    };

    let auth_header = format!("Bearer {}", token);

    let body = CarStatusBody {
        car_id,
        gas_level,
        kilometres,
        status_type,
    };

    let res = client
        .post(concatcp!(BASE_URL, "/cars/status"))
        .header(AUTHORIZATION, auth_header)
        .json(&body)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                HistoryResult {
                    success: true,
                    error_message: None,
                    items: None,
                }
            } else {
                HistoryResult {
                    success: false,
                    error_message: Some(format!("Failed with status: {}", response.status())),
                    items: None,
                }
            }
        }
        Err(e) => HistoryResult {
            success: false,
            error_message: Some(format!("Network error: {}", e)),
            items: None,
        },
    }
}

pub async fn get_histories(token: String) -> HistoryResult {
    let client_result = Client::builder()
        .danger_accept_invalid_certs(true)
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            return HistoryResult {
                success: false,
                error_message: Some(format!("Failed to build client: {}", e)),
                items: None,
            }
        }
    };

    let auth_header = format!("Bearer {}", token);

    let res = client
        .get(concatcp!(BASE_URL, "/cars/status"))
        .header(AUTHORIZATION, auth_header)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<HistoryResponse>().await {
                // Check if the backend returned an explicit error object
                if json.status.as_deref() == Some("error") {
                    return HistoryResult {
                        success: false,
                        error_message: Some(json.message.unwrap_or_else(|| "Unknown error".to_string())),
                        items: None,
                    };
                }

                // If we have car statuses, it's a success
                if let Some(statuses) = json.car_statuses {
                    return HistoryResult {
                        success: true,
                        error_message: None,
                        items: Some(statuses),
                    };
                }

                // Fallback for unexpected format
                return HistoryResult {
                    success: false,
                    error_message: Some("Invalid response format: Missing car_statuses".to_string()),
                    items: None,
                };
            }
            HistoryResult {
                success: false,
                error_message: Some("Failed to parse API response".to_string()),
                items: None,
            }
        }
        Err(e) => HistoryResult {
            success: false,
            error_message: Some(format!("Network error: {}", e)),
            items: None,
        },
    }
}
