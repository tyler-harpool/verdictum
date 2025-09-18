//! Configuration management HTTP handlers
//!
//! Provides endpoints for retrieving and managing hierarchical configurations
//! with support for district and judge-level overrides.

use crate::error::ApiError;
use crate::ports::config_repository::ConfigRepository;
use crate::services::config_service::ConfigService;
use crate::utils::{json_response as json, repository_factory::RepositoryFactory};
use serde_json::Value;
use spin_sdk::http::{Params, Request, Response};
use std::collections::HashMap;
use std::sync::Arc;

/// Get merged configuration for a district and optional judge
#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (status = 200, description = "Configuration retrieved successfully"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Configuration not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("X-Judge-ID" = String, Header, description = "Optional judge identifier for judge-specific configuration", example = "judge-123")
    ),
)]
pub fn get_config(req: Request, _params: Params) -> Response {
    // Get district from header (required)
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    // Get optional judge ID from header
    let judge_id = req.header("x-judge-id")
        .and_then(|h| h.as_str())
        .filter(|s| !s.is_empty());

    // Create repository and service
    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    // Get configuration
    // Use futures executor for async operations in Spin
    let result = futures::executor::block_on(service.get_config(district_id, judge_id));

    match result {
        Ok(config_response) => json::success_response(&config_response),
        Err(e) => json::error_response(&e),
    }
}

/// Get district-level configuration overrides only
#[utoipa::path(
    get,
    path = "/api/config/overrides/district",
    responses(
        (status = 200, description = "District overrides retrieved successfully"),
        (status = 404, description = "No district overrides found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn get_district_overrides(req: Request, _params: Params) -> Response {
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(service.get_district_overrides(district_id));

    match result {
        Ok(Some(overrides)) => json::success_response(&overrides),
        Ok(None) => json::error_response(&ApiError::NotFound(
            "No district overrides found".to_string()
        )),
        Err(e) => json::error_response(&e),
    }
}

/// Get judge-level configuration overrides only
#[utoipa::path(
    get,
    path = "/api/config/overrides/judge",
    responses(
        (status = 200, description = "Judge overrides retrieved successfully"),
        (status = 404, description = "No judge overrides found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("X-Judge-ID" = String, Header, description = "Judge identifier", example = "judge-123")
    ),
)]
pub fn get_judge_overrides(req: Request, _params: Params) -> Response {
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    let judge_id = match req.header("x-judge-id") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Judge-ID header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Judge-ID header is required".to_string()
        )),
    };

    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(service.get_judge_overrides(district_id, judge_id));

    match result {
        Ok(Some(overrides)) => json::success_response(&overrides),
        Ok(None) => json::error_response(&ApiError::NotFound(
            "No judge overrides found".to_string()
        )),
        Err(e) => json::error_response(&e),
    }
}

/// Update district-level configuration overrides
#[utoipa::path(
    put,
    path = "/api/config/overrides/district",
    request_body = HashMap<String, Value>,
    responses(
        (status = 200, description = "District overrides updated successfully"),
        (status = 400, description = "Invalid configuration values"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn update_district_config(req: Request, _params: Params) -> Response {
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    // Parse request body
    let updates: HashMap<String, Value> = match json::parse_body(req.body()) {
        Ok(u) => u,
        Err(e) => return json::error_response(&e),
    };

    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    // Validate updates
    if let Err(e) = service.validate_updates(&updates) {
        return json::error_response(&e);
    }

    let result = futures::executor::block_on(service.update_district_config(district_id, updates));

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": "District configuration updated successfully"
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Update judge-level configuration overrides
#[utoipa::path(
    put,
    path = "/api/config/overrides/judge",
    request_body = HashMap<String, Value>,
    responses(
        (status = 200, description = "Judge overrides updated successfully"),
        (status = 400, description = "Invalid configuration values"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("X-Judge-ID" = String, Header, description = "Judge identifier", example = "judge-123")
    ),
)]
pub fn update_judge_config(req: Request, _params: Params) -> Response {
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    let judge_id = match req.header("x-judge-id") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Judge-ID header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Judge-ID header is required".to_string()
        )),
    };

    // Parse request body
    let updates: HashMap<String, Value> = match json::parse_body(req.body()) {
        Ok(u) => u,
        Err(e) => return json::error_response(&e),
    };

    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    // Validate updates
    if let Err(e) = service.validate_updates(&updates) {
        return json::error_response(&e);
    }

    let result = futures::executor::block_on(service.update_judge_config(district_id, judge_id, updates));

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": "Judge configuration updated successfully"
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Clear district overrides (revert to base configuration)
#[utoipa::path(
    delete,
    path = "/api/config/overrides/district",
    responses(
        (status = 200, description = "District overrides cleared successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn clear_district_overrides(req: Request, _params: Params) -> Response {
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(service.clear_district_overrides(district_id));

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": "District overrides cleared successfully"
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Clear judge overrides (revert to district configuration)
#[utoipa::path(
    delete,
    path = "/api/config/overrides/judge",
    responses(
        (status = 200, description = "Judge overrides cleared successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("X-Judge-ID" = String, Header, description = "Judge identifier", example = "judge-123")
    ),
)]
pub fn clear_judge_overrides(req: Request, _params: Params) -> Response {
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    let judge_id = match req.header("x-judge-id") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Judge-ID header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Judge-ID header is required".to_string()
        )),
    };

    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(service.clear_judge_overrides(district_id, judge_id));

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": "Judge overrides cleared successfully"
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Preview configuration with proposed changes
#[utoipa::path(
    post,
    path = "/api/config/preview",
    request_body = HashMap<String, Value>,
    responses(
        (status = 200, description = "Configuration preview generated successfully"),
        (status = 400, description = "Invalid configuration values"),
        (status = 500, description = "Internal server error")
    ),
    tag = "configuration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("X-Judge-ID" = String, Header, description = "Optional judge identifier", example = "judge-123")
    ),
)]
pub fn preview_config(req: Request, _params: Params) -> Response {
    let district_id = match req.header("x-court-district") {
        Some(header) => match header.as_str() {
            Some(value) if !value.is_empty() => value,
            _ => return json::error_response(&ApiError::BadRequest(
                "X-Court-District header is required".to_string()
            )),
        },
        None => return json::error_response(&ApiError::BadRequest(
            "X-Court-District header is required".to_string()
        )),
    };

    let judge_id = req.header("x-judge-id")
        .and_then(|h| h.as_str())
        .filter(|s| !s.is_empty());

    // Parse request body
    let changes: HashMap<String, Value> = match json::parse_body(req.body()) {
        Ok(c) => c,
        Err(e) => return json::error_response(&e),
    };

    let repo = Arc::new(RepositoryFactory::config_repo(&req)) as Arc<dyn ConfigRepository>;
    let service = ConfigService::new(repo);

    // Validate proposed changes
    if let Err(e) = service.validate_updates(&changes) {
        return json::error_response(&e);
    }

    let result = futures::executor::block_on(service.preview_config_changes(district_id, judge_id, changes));

    match result {
        Ok(config_response) => json::success_response(&config_response),
        Err(e) => json::error_response(&e),
    }
}