use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TicketNotFound { id: u64 },
    InvalidAuth,
    WorngUsernameOrPassword,

    SQLiteErrorInsertFailed,
    TicketError(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("--> {:<8} - {self:?}", "Error");

        match self {
            Error::TicketNotFound { id } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Ticket with id {id} not found")),
            )
                .into_response(),

            Error::InvalidAuth => (StatusCode::UNAUTHORIZED, Html("Unauthorized")).into_response(),

            Error::WorngUsernameOrPassword => {
                (StatusCode::UNAUTHORIZED, Html("Worng username or password")).into_response()
            }

            Error::SQLiteErrorInsertFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("SQLite insert failed"),
            )
                .into_response(),

            Error::TicketError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Html(msg)).into_response()
            }
        }
    }
}
