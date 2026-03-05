use axum::{http::StatusCode, response::IntoResponse};
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
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();

        let status = match &self {
            AppError::DatabaseError(error_message) => {
                eprintln!("Database error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::RedisPoolError(error_message) => {
                eprintln!("Redis error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::RedisError(error_message) => {
                eprintln!("Redis error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::CsvError(error_message) => {
                eprintln!("CSV error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::ParseJsonError(error_message) => {
                eprintln!("JSON parsing error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::HashError(error_message) => {
                eprintln!("Hashing error {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::StdIoError(error_message) => {
                eprintln!("Standard input error {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::MissingField => StatusCode::BAD_REQUEST,
            AppError::WrongCredentials => StatusCode::UNAUTHORIZED,
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,
            AppError::EncodingError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Internal(error_message) => {
                eprintln!("Internal error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, message).into_response()
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
