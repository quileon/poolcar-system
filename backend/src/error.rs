use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

/// Error types for Axum.
///
/// Instead of handling the error directly and responding the client with the error message.
/// This code automatically handles the error message, outputing into stderr while also responding with appropriate messages to the client.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error")]
    RedisPoolError(#[from] deadpool_redis::PoolError),

    #[error("Internal server error")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Internal server error")]
    ParseJsonError(#[from] serde_json::Error),

    #[error("Internal server error")]
    HashError(#[from] argon2::password_hash::Error),

    #[error("Missing field")]
    MissingField,

    #[error("Wrong credentials")]
    WrongCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Internal server error")]
    EncodingError,
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
            AppError::ParseJsonError(error_message) => {
                eprintln!("JSON parsing error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::HashError(error_message) => {
                eprintln!("Hashing error {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::MissingField => StatusCode::BAD_REQUEST,
            AppError::WrongCredentials => StatusCode::UNAUTHORIZED,
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,
            AppError::EncodingError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, message).into_response()
    }
}
