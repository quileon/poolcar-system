use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

/// Error types for Axum.
///
/// Instead of handling the error directly and responding the client with the error message.
/// This code automatically handles the error message, outputing into stderr while also responding with appropriate messages to the client.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    HashError(#[from] argon2::password_hash::Error),

    #[error("Page not found")]
    PageNotFound,

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
            AppError::Database(error_message) => {
                eprintln!("Database error: {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::HashError(error_message) => {
                eprintln!("Hashing error {:?}", error_message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::PageNotFound => StatusCode::NOT_FOUND,
            AppError::MissingField => StatusCode::BAD_REQUEST,
            AppError::WrongCredentials => StatusCode::UNAUTHORIZED,
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,
            AppError::EncodingError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, message).into_response()
    }
}
