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

#[derive(Deserialize)]
struct LoginData {
    role: String,
    token: String,
    #[allow(dead_code)]
    username: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    status: String,
    data: Option<LoginData>,
    message: Option<String>,
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
                token: None
            }
        }
    };

    // Make sure we pass the token as a Bearer token
    let auth_header = format!("Bearer {}", token);

    let res = client
        .get(concatcp!(BASE_URL, "/auth/verify"))
        .header(AUTHORIZATION, auth_header)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<LoginResponse>().await {
                if json.status == "success" {
                    if let Some(data) = json.data {
                        if data.role == "Security" {
                            return AuthResult {
                                success: true,
                                error_message: None,
                                token: Some(data.token),
                            };
                        } else {
                            return AuthResult {
                                success: false,
                                error_message: Some("User is not authorized".to_string()),
                                token: None,
                            };
                        }
                    }
                    return AuthResult {
                        success: false,
                        error_message: Some("Invalid response format: Missing user data".to_string()),
                        token: None,
                    };
                } else {
                    return AuthResult {
                        success: false,
                        error_message: Some(
                            json.message.unwrap_or_else(|| "Token is invalid or expired".to_string()),
                        ),
                        token: None,
                    };
                }
            }
            AuthResult {
                success: false,
                error_message: Some("Failed to parse API response".to_string()),
                token: None,
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
        .post(concatcp!(BASE_URL, "/auth/login"))
        .json(&req_body)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<LoginResponse>().await {
                if json.status == "success" {
                    if let Some(data) = json.data {
                        if data.role == "Security" {
                            return AuthResult {
                                success: true,
                                error_message: None,
                                token: Some(data.token),
                            };
                        } else {
                            return AuthResult {
                                success: false,
                                error_message: Some("User is not authorized".to_string()),
                                token: None,
                            };
                        }
                    }
                    return AuthResult {
                        success: false,
                        error_message: Some("Invalid response format: Missing user data".to_string()),
                        token: None,
                    };
                } else {
                    return AuthResult {
                        success: false,
                        error_message: Some(
                            json.message.unwrap_or_else(|| "Invalid credentials".to_string()),
                        ),
                        token: None,
                    };
                }
            }
            AuthResult {
                success: false,
                error_message: Some("Failed to parse API response".to_string()),
                token: None,
            }
        }
        Err(e) => AuthResult {
            success: false,
            error_message: Some(format!("Network error: {}", e)),
            token: None,
        },
    }
}
