//! URL-based configuration management HTTP handlers
//!
//! These handlers extract tenant information from the URL path instead of headers,
//! supporting RESTful API design patterns like `/api/courts/sdny/config`

use crate::error::ApiError;
use crate::ports::config_repository::ConfigRepository;
use crate::services::config_service::ConfigService;
use crate::utils::{json_response as json, repository_factory::RepositoryFactory};
use serde_json::Value;
use spin_sdk::http::{Params, Request, Response};
use std::collections::HashMap;
use std::sync::Arc;

/// Get merged configuration for a district from URL
///
/// URL pattern: `/api/courts/{district}/config`
/// Example: `/api/courts/sdny/config`
pub fn get_config(req: Request, params: Params) -> Response {
    // Extract district from URL parameter
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    // Get optional judge ID from query params or header for backward compatibility
    let judge_id = req.header("x-judge-id")
        .and_then(|h| h.as_str())
        .filter(|s| !s.is_empty());

    // Create repository using URL-based extraction
    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    // Get configuration
    let result = futures::executor::block_on(service.get_config(district_id, judge_id));

    match result {
        Ok(config_response) => {
            // Return successful response
            json::success_response(&config_response)
        },
        Err(e) => json::error_response(&e),
    }
}

/// Get district-level configuration overrides from URL
///
/// URL pattern: `/api/courts/{district}/config/overrides/district`
pub fn get_district_overrides(req: Request, params: Params) -> Response {
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(
        service.get_district_overrides(district_id)
    );

    match result {
        Ok(overrides) => json::success_response(&overrides),
        Err(e) => json::error_response(&e),
    }
}

/// Get judge-level configuration overrides from URL
///
/// URL pattern: `/api/courts/{district}/config/overrides/judge/{judge_id}`
pub fn get_judge_overrides(req: Request, params: Params) -> Response {
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    let judge_id = match params.get("judge_id") {
        Some(judge) if !judge.is_empty() => judge,
        _ => return json::error_response(&ApiError::BadRequest(
            "Judge ID is required in URL".to_string()
        )),
    };

    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(
        service.get_judge_overrides(district_id, judge_id)
    );

    match result {
        Ok(overrides) => json::success_response(&overrides),
        Err(e) => json::error_response(&e),
    }
}

/// Update district configuration from URL
///
/// URL pattern: `PUT /api/courts/{district}/config/overrides/district`
pub fn update_district_config(req: Request, params: Params) -> Response {
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    // Parse request body
    let config: HashMap<String, Value> = match serde_json::from_slice(&req.body()) {
        Ok(c) => c,
        Err(e) => return json::error_response(&ApiError::BadRequest(
            format!("Invalid JSON in request body: {}", e)
        )),
    };

    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(
        service.update_district_config(district_id, config)
    );

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": format!("District {} configuration updated successfully", district_id)
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Update judge configuration from URL
///
/// URL pattern: `PUT /api/courts/{district}/config/overrides/judge/{judge_id}`
pub fn update_judge_config(req: Request, params: Params) -> Response {
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    let judge_id = match params.get("judge_id") {
        Some(judge) if !judge.is_empty() => judge,
        _ => return json::error_response(&ApiError::BadRequest(
            "Judge ID is required in URL".to_string()
        )),
    };

    let config: HashMap<String, Value> = match serde_json::from_slice(&req.body()) {
        Ok(c) => c,
        Err(e) => return json::error_response(&ApiError::BadRequest(
            format!("Invalid JSON in request body: {}", e)
        )),
    };

    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(
        service.update_judge_config(district_id, judge_id, config)
    );

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": format!("Judge {} configuration updated successfully", judge_id)
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Clear district overrides from URL
///
/// URL pattern: `DELETE /api/courts/{district}/config/overrides/district`
pub fn clear_district_overrides(req: Request, params: Params) -> Response {
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(
        service.clear_district_overrides(district_id)
    );

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": format!("District {} overrides cleared successfully", district_id)
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Clear judge overrides from URL
///
/// URL pattern: `DELETE /api/courts/{district}/config/overrides/judge/{judge_id}`
pub fn clear_judge_overrides(req: Request, params: Params) -> Response {
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    let judge_id = match params.get("judge_id") {
        Some(judge) if !judge.is_empty() => judge,
        _ => return json::error_response(&ApiError::BadRequest(
            "Judge ID is required in URL".to_string()
        )),
    };

    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(
        service.clear_judge_overrides(district_id, judge_id)
    );

    match result {
        Ok(()) => json::success_response(&serde_json::json!({
            "message": format!("Judge {} overrides cleared successfully", judge_id)
        })),
        Err(e) => json::error_response(&e),
    }
}

/// Preview configuration with temporary overrides from URL
///
/// URL pattern: `POST /api/courts/{district}/config/preview`
pub fn preview_config(req: Request, params: Params) -> Response {
    let district_id = match params.get("district") {
        Some(district) if !district.is_empty() => district,
        _ => return json::error_response(&ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        )),
    };

    // Parse preview request
    #[derive(serde::Deserialize)]
    struct PreviewRequest {
        judge_id: Option<String>,
        overrides: HashMap<String, Value>,
    }

    let preview_req: PreviewRequest = match serde_json::from_slice(&req.body()) {
        Ok(p) => p,
        Err(e) => return json::error_response(&ApiError::BadRequest(
            format!("Invalid preview request: {}", e)
        )),
    };

    let repo = match RepositoryFactory::config_repo_from_url(&req) {
        Ok(repo) => Arc::new(repo) as Arc<dyn ConfigRepository>,
        Err(e) => return json::error_response(&ApiError::BadRequest(e)),
    };

    let service = ConfigService::new(repo);

    let result = futures::executor::block_on(
        service.preview_config_changes(district_id, preview_req.judge_id.as_deref(), preview_req.overrides)
    );

    match result {
        Ok(preview) => json::success_response(&preview),
        Err(e) => json::error_response(&e),
    }
}