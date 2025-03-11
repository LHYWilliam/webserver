use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorStatusCode;

#[derive(Debug, Error)]
pub enum TicketError {
    #[error("Failed to create ticket")]
    CreateFailed,

    #[error("Ticket with id {0} not found")]
    NotFound(i64),
}

impl ErrorStatusCode for TicketError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
