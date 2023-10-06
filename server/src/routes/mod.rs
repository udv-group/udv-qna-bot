mod categories;
mod questions;
mod users;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
pub use categories::category_router;
pub use questions::questions_router;
pub use users::users_router;

pub type ApiResponse<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    SqlxError(sqlx::Error),
    IoError(std::io::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        ApiError::SqlxError(value)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(value: std::io::Error) -> Self {
        ApiError::IoError(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::SqlxError(err) => match err {
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::IoError(err) => match err.kind() {
                std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
        .into_response()
    }
}
