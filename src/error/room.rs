use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorStatusCode;

#[derive(Debug, Error)]
pub enum RoomError {
    #[error("User not found")]
    UserNotFound,

    #[error("Room not found")]
    RoomNotFound,
}

#[allow(clippy::match_single_binding)]
impl ErrorStatusCode for RoomError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::NOT_FOUND,
        }
    }
}
