use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use db::{PoolError, TokioPostgresError};

#[derive(Debug)]
pub enum CustomError {
    FaultySetup(String),
    Database(String),
}

/// "{}"書式記述の使用を許可
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CustomError::FaultySetup(ref cause) => write!(f, "Setup Error: {}", cause),
            CustomError::Database(ref cause) => write!(f, "Database Error: {}", cause),
        }
    }
}

/// 次のエラーはブラウザに表示されますか?
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            CustomError::Database(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            CustomError::FaultySetup(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        };

        format!("status={}, message={}", status, error_message).into_response()
    }
}

impl From<axum::http::uri::InvalidUri> for CustomError {
    fn from(err: axum::http::uri::InvalidUri) -> Self {
        CustomError::FaultySetup(err.to_string())
    }
}

impl From<TokioPostgresError> for CustomError {
    fn from(err: TokioPostgresError) -> Self {
        CustomError::Database(err.to_string())
    }
}

impl From<PoolError> for CustomError {
    fn from(err: PoolError) -> Self {
        CustomError::Database(err.to_string())
    }
}
