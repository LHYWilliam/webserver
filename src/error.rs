use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AuthErrorMissCookie,
    AuthErrorWorngUsernameOrPassword,

    TicketErrorCreateFailed,
    TicketErrorIdNotFound { id: i64 },

    SQLiteErrorInsertFailed,
    SQLiteErrorSelectFailed,
    SQLiteErrorDeleteFailed,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("--> {:<8} - {self:?}", "Error");

        match self {
            Error::AuthErrorMissCookie => (
                StatusCode::UNAUTHORIZED,
                Html("Miss username or password cookie"),
            )
                .into_response(),

            Error::AuthErrorWorngUsernameOrPassword => {
                (StatusCode::UNAUTHORIZED, Html("Worng username or password")).into_response()
            }

            Error::TicketErrorCreateFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Ticket create failed"),
            )
                .into_response(),

            Error::TicketErrorIdNotFound { id } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Ticket with id {id} not found")),
            )
                .into_response(),

            Error::SQLiteErrorInsertFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("SQLite insert failed"),
            )
                .into_response(),

            Error::SQLiteErrorSelectFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("SQLite select failed"),
            )
                .into_response(),

            Error::SQLiteErrorDeleteFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("SQLite delete failed"),
            )
                .into_response(),
        }
    }
}
