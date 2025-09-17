//! Health check endpoint for monitoring and deployment checks

use crate::error::ApiResult;
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use spin_sdk::key_value::Store;
use utoipa::ToSchema;

/// Health status response
#[derive(serde::Serialize, ToSchema)]
pub struct HealthStatus {
    status: &'static str,
    version: &'static str,
    storage: &'static str,
    timestamp: String,
}

/// Health check endpoint
///
/// Returns the health status of the API including storage connectivity
#[utoipa::path(
    get,
    path = "/api/health",
    tags = ["monitoring"],
    description = "Check the health status of the API",
    responses(
        (status = 200, description = "API is healthy", body = HealthStatus),
        (status = 503, description = "API is unhealthy")
    )
)]
pub(crate) fn health_check(_req: Request, _p: Params) -> ApiResult<impl IntoResponse> {
    // Check if storage is accessible
    let storage_status = match Store::open_default() {
        Ok(store) => {
            // Try to list keys to ensure storage is working
            match store.get_keys() {
                Ok(_) => "connected",
                Err(_) => "error",
            }
        }
        Err(_) => "disconnected",
    };

    let health = HealthStatus {
        status: if storage_status == "connected" { "healthy" } else { "degraded" },
        version: env!("CARGO_PKG_VERSION"),
        storage: storage_status,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let status_code = if storage_status == "connected" { 200 } else { 503 };

    Ok(ResponseBuilder::new(status_code)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&health).unwrap())
        .build())
}