use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TicketNotFound { id: u64 },
    InvalidAuth,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::TicketNotFound { id } => {
                println!("--> {:<8} - {self:?}", "Error");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!("Ticket with id {id} not found")),
                )
                    .into_response()
            }

            Error::InvalidAuth => {
                println!("--> {:<8} - {self:?}", "Error");

                (StatusCode::UNAUTHORIZED, Html("Unauthorized")).into_response()
            }
        }
    }
}
