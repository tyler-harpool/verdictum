//! HTTP request handlers for the ToDo API
//!
//! This module contains all the HTTP request handlers for the API endpoints,
//! including the ToDo CRUD operations and API documentation endpoints.

use anyhow::Result;
use spin_sdk::http::{conversions::IntoBody, Response, ResponseBuilder};

/// API documentation handlers
pub mod docs;
/// ToDo item CRUD operation handlers
pub(crate) mod todo;

/// Helper struct for creating JSON responses
pub(super) struct JsonResponse {}

impl JsonResponse {
    /// Create a successful JSON response with the given payload
    ///
    /// This method creates an HTTP 200 response with:
    /// - Status code: 200 (OK)
    /// - Content-Type: application/json
    /// - Body: The provided payload serialized to JSON
    pub(super) fn from(payload: impl IntoBody) -> Result<Response> {
        Ok(ResponseBuilder::new(200)
            .header("content-type", "application/json")
            .body(payload)
            .build())
    }
}
