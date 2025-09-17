//! Error handling module for the ToDo API
//!
//! Provides custom error types and conversions for consistent error handling
//! across the application.

use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Response};
use std::fmt;
use utoipa::ToSchema;

/// Error response model for API errors
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// HTTP status code
    pub status: u16,
    /// Optional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

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
        let (status, message, details) = match self {
            ApiError::NotFound(msg) => (404, "Not Found".to_string(), Some(msg)),
            ApiError::BadRequest(msg) => (400, "Bad Request".to_string(), Some(msg)),
            ApiError::Internal(msg) => (500, "Internal Server Error".to_string(), Some(msg)),
            ApiError::StorageError(err) => (
                500,
                "Storage operation failed".to_string(),
                Some(err.to_string()),
            ),
            ApiError::SerializationError(msg) => (
                400,
                "Invalid data format".to_string(),
                Some(msg),
            ),
        };

        let error_response = ErrorResponse {
            error: message,
            status,
            details,
        };

        Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&error_response).unwrap_or_default())
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