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
                StatusCode::BAD_REQUEST,
                Html("Miss username or password cookie".to_string()),
            ),
            Error::AuthErrorWorngUsernameOrPassword => (
                StatusCode::UNAUTHORIZED,
                Html("Worng username or password".to_string()),
            ),

            Error::TicketErrorCreateFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Ticket create failed".to_string()),
            ),
            Error::TicketErrorIdNotFound { id } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Ticket with id {id} not found")),
            ),
            Error::SQLiteErrorInsertFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("SQLite insert failed".to_string()),
            ),
            Error::SQLiteErrorSelectFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("SQLite select failed".to_string()),
            ),
            Error::SQLiteErrorDeleteFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("SQLite delete failed".to_string()),
            ),
        }
        .into_response()
    }
}
