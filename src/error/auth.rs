use axum::http::StatusCode;
use thiserror::Error;

use crate::error::ErrorResponse;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,

    #[error("Invalid cookie")]
    InvalidCookie,

    #[error("Invalid username or password")]
    WrongCredentials,
}

#[allow(clippy::match_single_binding)]
impl ErrorResponse for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::UNAUTHORIZED,
        }
    }
}
