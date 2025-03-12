use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorStatusCode;

#[derive(Debug, Error)]
pub enum RoomError {
    #[error("Room not found")]
    RoomNotFound,

    #[error("User not found")]
    UserNotFound,
}

#[allow(clippy::match_single_binding)]
impl ErrorStatusCode for RoomError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::RoomNotFound => StatusCode::NOT_FOUND,
            Self::UserNotFound => StatusCode::NOT_FOUND,
        }
    }
}
