use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("configuration not found")]
    ConfigNotFound,
    #[error("other error: {0}")]
    Other(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for BackendError {
    fn into_response(self) -> Response {
        let status = match self {
            BackendError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BackendError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BackendError::SerdeJson(_) => StatusCode::BAD_REQUEST,
            BackendError::ConfigNotFound => StatusCode::NOT_FOUND,
            BackendError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = axum::Json(ErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

pub type BackendResult<T> = Result<T, BackendError>;
