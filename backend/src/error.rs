use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Password hashing failed")]
    HashError(#[from] argon2::password_hash::Error),

    #[error("Invalid password or usernameemail")]
    InvalidLogin,

    #[error("Page not found")]
    NotFound,

    #[error("Missing field")]
    MissingField,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Database(error_message) => {
                eprintln!("DATABASE ERROR: {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            AppError::HashError(error_message) => {
                eprintln!("HASHING ERROR: {:?}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            AppError::InvalidLogin => {
                eprintln!("Wrong credentials");
                (StatusCode::UNAUTHORIZED, "Wrong credentials")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::MissingField => (StatusCode::BAD_REQUEST, "Missing field"),
        };

        (status, message).into_response()
    }
}
