//! JSON response helpers for HTTP handlers

use crate::error::ApiError;
use spin_sdk::http::Response;

/// Create a success JSON response
pub fn success_response<T: serde::Serialize>(data: &T) -> Response {
    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(data).unwrap())
        .build()
}

/// Create an error JSON response
pub fn error_response(error: &ApiError) -> Response {
    let status = match error {
        ApiError::NotFound(_) => 404,
        ApiError::BadRequest(_) => 400,
        ApiError::ValidationError(_) => 400,
        ApiError::InvalidInput(_) => 400,
        ApiError::SerializationError(_) => 400,
        ApiError::Forbidden(_) => 403,
        ApiError::Conflict(_) => 409,
        ApiError::StorageError(_) => 500,
        ApiError::Internal(_) => 500,
        ApiError::InternalServerError(_) => 500,
    };

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(error).unwrap())
        .build()
}

/// Parse request body as JSON
pub fn parse_body<T: for<'de> serde::Deserialize<'de>>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body)
        .map_err(|e| ApiError::BadRequest(format!("Invalid JSON: {}", e)))
}