use axum::http::StatusCode;
use thiserror::Error;

use crate::error::ErrorResponse;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database insert failed")]
    InsertFailed,

    #[error("Database select failed")]
    SelectFailed,

    #[error("Database delete failed")]
    DeleteFailed,
}

#[allow(clippy::match_single_binding)]
impl ErrorResponse for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
