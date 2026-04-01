use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Error types for Axum.
///
/// Instead of handling the error directly and responding the client with the error message.
/// This code automatically handles the error message, outputing into stderr while also responding with appropriate messages to the client.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error")]
    DatabaseError(sqlx::Error),

    #[error("Internal server error")]
    RedisPoolError(#[from] deadpool_redis::PoolError),

    #[error("Internal server error")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Internal server error")]
    CsvError(#[from] csv::Error),

    #[error("Internal server error")]
    ParseJsonError(#[from] serde_json::Error),

    #[error("Internal server error")]
    HashError(#[from] argon2::password_hash::Error),

    #[error("Internal server error")]
    StdIoError(#[from] std::io::Error),

    #[error("Internal server error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Missing field")]
    MissingField,

    #[error("Wrong credentials")]
    WrongCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Internal server error")]
    EncodingError,

    #[error("Not found")]
    NotFound,

    #[error("Internal server error")]
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            other => AppError::DatabaseError(other),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::DatabaseError(error_message) => {
                eprintln!("Database error: {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::RedisPoolError(error_message) => {
                eprintln!("Redis pool error: {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache service error")
            }
            AppError::RedisError(error_message) => {
                eprintln!("Redis error: {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache service error")
            }
            AppError::CsvError(error_message) => {
                eprintln!("CSV error: {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "CSV export error")
            }
            AppError::ParseJsonError(error_message) => {
                eprintln!("JSON parsing error: {:?}", error_message);
                (StatusCode::BAD_REQUEST, "Invalid JSON format")
            }
            AppError::HashError(error_message) => {
                eprintln!("Hashing error {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Password hashing error")
            }
            AppError::StdIoError(error_message) => {
                eprintln!("Standard input error {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "File I/O error")
            }
            AppError::ReqwestError(error_message) => {
                eprintln!("Reqwest error {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "HTTP request error")
            }

            AppError::MissingField => (StatusCode::BAD_REQUEST, "Missing required field"),
            AppError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AppError::EncodingError => (StatusCode::INTERNAL_SERVER_ERROR, "Token encoding error"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Internal(error_message) => {
                eprintln!("Internal error: {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = json!({
            "status": "error",
            "message": error_message,
        });

        (status, Json(body)).into_response()
    }
}

/// Error types for MQTT handler tasks.
#[derive(Error, Debug)]
pub enum MqttError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis pool error")]
    RedisPoolError(#[from] deadpool_redis::PoolError),

    #[error("Redis error")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Failed to parse MQTT payload to JSON")]
    ParseJsonError(#[from] serde_json::Error),

    #[error("Location is not valid")]
    InvalidLocation,
}

/// Error types for other spawn tasks
#[derive(Error, Debug)]
pub enum TasksError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis pool error")]
    RedisPoolError(#[from] deadpool_redis::PoolError),

    #[error("Redis error")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Failed to parse JSON")]
    ParseJsonError(#[from] serde_json::Error),
}
