use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AuthErrorInvalidToken,
    AuthErrorInvalidCookie,
    AuthErrorWrongUsernameOrPassword,

    TicketErrorCreateFailed,
    TicketErrorIdNotFound { id: i64 },

    SQLiteErrorInsertFailed,
    SQLiteErrorSelectFailed,
    SQLiteErrorDeleteFailed,

    ExtractorError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("--> {:<8} - {self:?}", "Error");

        match self {
            Error::AuthErrorInvalidToken => {
                (StatusCode::UNAUTHORIZED, Html("Invalid token".to_string()))
            }

            Error::AuthErrorInvalidCookie => {
                (StatusCode::UNAUTHORIZED, Html("Invalid Cookie".to_string()))
            }

            Error::AuthErrorWrongUsernameOrPassword => (
                StatusCode::UNAUTHORIZED,
                Html("Invalid username or password".to_string()),
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

            Error::ExtractorError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Extractor error".to_string()),
            ),
        }
        .into_response()
    }
}
