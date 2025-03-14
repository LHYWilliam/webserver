mod auth;
mod database;
mod room;
mod ticket;

pub use auth::AuthError;
pub use database::DatabaseError;
pub use room::RoomError;
pub use ticket::TicketError;
pub type Result<T> = std::result::Result<T, Error>;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Ticket(#[from] TicketError),

    #[error(transparent)]
    Database(#[from] DatabaseError),

    #[error(transparent)]
    Room(#[from] RoomError),

    #[error("Unknown error")]
    Unknown,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("[{:^12}] - {self:?}", "Error");

        let (status, message) = match self {
            Error::Auth(e) => (e.status_code(), e.to_string()),
            Error::Ticket(e) => (e.status_code(), e.to_string()),
            Error::Database(e) => (e.status_code(), e.to_string()),
            Error::Room(e) => (e.status_code(), e.to_string()),
            Error::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, message).into_response()
    }
}

trait ErrorStatusCode {
    fn status_code(&self) -> StatusCode;
}
