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
    pub police_number: String,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: String,
    pub recorded_at: String,
}

#[derive(Serialize)]
pub struct CarStatusBody {
    pub car_id: i32,
    pub gas_level: f64,
    pub kilometres: f64,
    pub status_type: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub async fn post_history(token: String, car_id: i32, gas_level: f64, kilometres: f64, status_type: String) -> HistoryResult {
    if status_type != "Departure" && status_type != "Return" {
        return HistoryResult {
            success: false,
            error_message: Some("Invalid status type: must be \"Departure\" or \"Return\"".to_string()),
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
        .post(concatcp!(BASE_URL, "/cars/history"))
        .header(AUTHORIZATION, auth_header)
        .json(&body)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let body_text = match response.text().await {
                Ok(b) => b,
                Err(e) => return HistoryResult {
                    success: false,
                    error_message: Some(format!("Failed to read response: {}", e)),
                    items: None,
                },
            };

            if status.is_success() {
                HistoryResult {
                    success: true,
                    error_message: None,
                    items: None,
                }
            } else {
                let msg = serde_json::from_str::<ErrorResponse>(&body_text)
                    .map(|e| e.error)
                    .unwrap_or_else(|_| format!("Request failed with status {}", status));
                HistoryResult {
                    success: false,
                    error_message: Some(msg),
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
        .get(concatcp!(BASE_URL, "/cars/history"))
        .header(AUTHORIZATION, auth_header)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let body_text = match response.text().await {
                Ok(b) => b,
                Err(e) => return HistoryResult {
                    success: false,
                    error_message: Some(format!("Failed to read response: {}", e)),
                    items: None,
                },
            };

            if status.is_success() {
                match serde_json::from_str::<Vec<CarStatus>>(&body_text) {
                    Ok(statuses) => HistoryResult {
                        success: true,
                        error_message: None,
                        items: Some(statuses),
                    },
                    Err(_) => HistoryResult {
                        success: false,
                        error_message: Some(format!("Failed to parse API response: {}", body_text)),
                        items: None,
                    },
                }
            } else {
                let msg = serde_json::from_str::<ErrorResponse>(&body_text)
                    .map(|e| e.error)
                    .unwrap_or_else(|_| format!("Request failed with status {}", status));
                HistoryResult {
                    success: false,
                    error_message: Some(msg),
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
