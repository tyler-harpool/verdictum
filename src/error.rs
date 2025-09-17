//! Error handling module for the ToDo API
//!
//! Provides custom error types and conversions for consistent error handling
//! across the application.

use spin_sdk::http::{IntoResponse, Response};
use std::fmt;

/// API Error type representing all possible errors in the application
#[derive(Debug)]
pub enum ApiError {
    /// Item not found (404)
    NotFound(String),
    /// Bad request - validation error (400)
    BadRequest(String),
    /// Internal server error (500)
    Internal(String),
    /// Storage operation failed
    StorageError(anyhow::Error),
    /// JSON serialization/deserialization error
    SerializationError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ApiError::StorageError(err) => write!(f, "Storage error: {}", err),
            ApiError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (404, msg),
            ApiError::BadRequest(msg) => (400, msg),
            ApiError::Internal(msg) => (500, msg),
            ApiError::StorageError(_) => (500, "Storage operation failed".to_string()),
            ApiError::SerializationError(_) => (400, "Invalid data format".to_string()),
        };

        Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(serde_json::json!({
                "error": message,
                "status": status
            }).to_string())
            .build()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::StorageError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::SerializationError(err.to_string())
    }
}

impl From<uuid::Error> for ApiError {
    fn from(_: uuid::Error) -> Self {
        ApiError::BadRequest("Invalid UUID format".to_string())
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Validation helper for input data
pub mod validation {
    use super::ApiError;

    /// Validate ToDo content
    pub fn validate_todo_content(content: &str) -> Result<(), ApiError> {
        if content.trim().is_empty() {
            return Err(ApiError::BadRequest("ToDo content cannot be empty".to_string()));
        }

        if content.len() > 1000 {
            return Err(ApiError::BadRequest(
                "ToDo content cannot exceed 1000 characters".to_string(),
            ));
        }

        Ok(())
    }
}