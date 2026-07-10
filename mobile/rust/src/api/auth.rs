use reqwest::Client;
use serde::{Deserialize, Serialize};
use const_format::concatcp;
use reqwest::header::AUTHORIZATION;
use crate::api::BASE_URL;

pub struct AuthResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub token: Option<String>,
}

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Successful login response: { "token": "...", "username": "...", "role": "..." }
#[derive(Deserialize)]
struct LoginSuccess {
    token: String,
    #[allow(dead_code)]
    username: String,
    role: String,
}

/// Error response: { "error": "..." }
#[derive(Deserialize)]
struct LoginError {
    error: String,
}

pub async fn verify(token: String) -> AuthResult {
    let client_result = Client::builder()
        .danger_accept_invalid_certs(true)
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            return AuthResult {
                success: false,
                error_message: Some(format!("Failed to build client: {}", e)),
                token: None,
            }
        }
    };

    let auth_header = format!("Bearer {}", token);

    let res = client
        .get(concatcp!(BASE_URL, "/verify"))
        .header(AUTHORIZATION, auth_header)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let body = match response.text().await {
                Ok(b) => b,
                Err(e) => {
                    return AuthResult {
                        success: false,
                        error_message: Some(format!("Failed to read response: {}", e)),
                        token: None,
                    }
                }
            };

            if status.is_success() {
                match serde_json::from_str::<LoginSuccess>(&body) {
                    Ok(data) => {
                        if data.role == "Security" || data.role == "Admin" {
                            AuthResult {
                                success: true,
                                error_message: None,
                                token: Some(data.token),
                            }
                        } else {
                            AuthResult {
                                success: false,
                                error_message: Some("User is not authorized".to_string()),
                                token: None,
                            }
                        }
                    }
                    Err(_) => AuthResult {
                        success: false,
                        error_message: Some("Failed to parse API response".to_string()),
                        token: None,
                    },
                }
            } else {
                // Try to extract the error message from the response body
                let msg = serde_json::from_str::<LoginError>(&body)
                    .map(|e| e.error)
                    .unwrap_or_else(|_| {
                        if status.as_u16() == 401 {
                            "Invalid username or password".to_string()
                        } else if status.as_u16() == 403 {
                            "Access denied".to_string()
                        } else {
                            format!("Request failed with status {}", status)
                        }
                    });
                AuthResult {
                    success: false,
                    error_message: Some(msg),
                    token: None,
                }
            }
        }
        Err(e) => AuthResult {
            success: false,
            error_message: Some(format!("Network error: {}", e)),
            token: None,
        },
    }
}

pub async fn login(username: String, password: String) -> AuthResult {
    let client_result = Client::builder()
        .danger_accept_invalid_certs(true)
        .build();

    let client = match client_result {
        Ok(c) => c,
        Err(e) => {
            return AuthResult {
                success: false,
                error_message: Some(format!("Failed to build client: {}", e)),
                token: None,
            }
        }
    };

    let req_body = LoginRequest {
        username,
        password,
    };

    let res = client
        .post(concatcp!(BASE_URL, "/login"))
        .json(&req_body)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let body = match response.text().await {
                Ok(b) => b,
                Err(e) => {
                    return AuthResult {
                        success: false,
                        error_message: Some(format!("Failed to read response: {}", e)),
                        token: None,
                    }
                }
            };

            if status.is_success() {
                match serde_json::from_str::<LoginSuccess>(&body) {
                    Ok(data) => {
                        if data.role == "Security" || data.role == "Admin" {
                            AuthResult {
                                success: true,
                                error_message: None,
                                token: Some(data.token),
                            }
                        } else {
                            AuthResult {
                                success: false,
                                error_message: Some("User is not authorized".to_string()),
                                token: None,
                            }
                        }
                    }
                    Err(_) => AuthResult {
                        success: false,
                        error_message: Some("Failed to parse API response".to_string()),
                        token: None,
                    },
                }
            } else {
                // Try to extract the error message from the response body
                let msg = serde_json::from_str::<LoginError>(&body)
                    .map(|e| e.error)
                    .unwrap_or_else(|_| {
                        if status.as_u16() == 401 {
                            "Invalid username or password".to_string()
                        } else if status.as_u16() == 403 {
                            "Access denied".to_string()
                        } else {
                            format!("Request failed with status {}", status)
                        }
                    });
                AuthResult {
                    success: false,
                    error_message: Some(msg),
                    token: None,
                }
            }
        }
        Err(e) => AuthResult {
            success: false,
            error_message: Some(format!("Network error: {}", e)),
            token: None,
        },
    }
}
