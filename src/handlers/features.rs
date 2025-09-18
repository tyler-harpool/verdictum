//! REST API handlers for feature flag management
//!
//! This module provides HTTP endpoints for managing feature flags
//! and implementation tracking in the judicial system.

use crate::domain::features::{
    FeatureManager, JudicialFeatures, ImplementationTracker,
    FeatureStatus, ImplementationStatus
};
use crate::error::{ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use spin_sdk::key_value::Store;
use utoipa::ToSchema;

/// Request to update implementation status
#[derive(Deserialize, ToSchema)]
pub struct UpdateRequest {
    pub feature_name: String,
    pub status: Option<ImplementationStatus>,
    pub progress: Option<u8>,
}

/// Request to override feature flags
#[derive(Deserialize, ToSchema)]
pub struct OverrideRequest {
    pub feature_path: String,
    pub enabled: bool,
}

const FEATURES_KEY: &str = "features:config";
const TRACKER_KEY: &str = "features:tracker";

/// Response for feature status
#[derive(Debug, Serialize, ToSchema)]
pub struct FeaturesResponse {
    pub enabled_features: Vec<String>,
    pub case_management: bool,
    pub judge_assignment: bool,
    pub docket_management: bool,
    pub calendar_scheduling: bool,
    pub deadline_tracking: bool,
    pub document_filing: bool,
    pub notifications: bool,
    pub reporting: bool,
    pub user_management: bool,
    pub audit_logging: bool,
}

/// Request to update feature flags
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFeaturesRequest {
    pub feature_path: String,
    pub enabled: bool,
}

/// Get current feature flags
#[utoipa::path(
    get,
    path = "/api/features",
    responses(
        (status = 200, description = "Current feature flag configuration", body = FeaturesResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Feature Management"
)]
pub fn get_features(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let features = if let Some(data) = store.get_json::<JudicialFeatures>(FEATURES_KEY)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        JudicialFeatures::default()
    };

    let manager = FeatureManager::with_config(features.clone());

    let response = FeaturesResponse {
        enabled_features: manager.get_enabled_features(),
        case_management: features.core.case_management,
        judge_assignment: features.advanced.judge_assignment,
        docket_management: features.core.basic_docket,
        calendar_scheduling: features.advanced.automated_scheduling,
        deadline_tracking: features.advanced.deadline_tracking,
        document_filing: false, // Not in current structure
        notifications: false, // Not in current structure
        reporting: features.advanced.statistical_reporting,
        user_management: false, // Not in current structure
        audit_logging: false, // Not in current structure
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Update feature flags
#[utoipa::path(
    patch,
    path = "/api/features",
    request_body = UpdateFeaturesRequest,
    responses(
        (status = 200, description = "Feature flag updated successfully"),
        (status = 400, description = "Invalid feature path or request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Feature Management"
)]
pub fn update_feature(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: UpdateFeaturesRequest = serde_json::from_slice(body)?;

    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let mut features = if let Some(data) = store.get_json::<JudicialFeatures>(FEATURES_KEY)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        JudicialFeatures::default()
    };

    // Update the appropriate feature based on the path
    match request.feature_path.as_str() {
        "case_management" => features.core.case_management = request.enabled,
        "judge_assignment" => features.advanced.judge_assignment = request.enabled,
        "docket_management" => features.core.basic_docket = request.enabled,
        "calendar_scheduling" => features.advanced.automated_scheduling = request.enabled,
        "deadline_tracking" => features.advanced.deadline_tracking = request.enabled,
        "reporting" => features.advanced.statistical_reporting = request.enabled,
        "party_management" => features.core.party_management = request.enabled,
        "sentencing_calculator" => features.advanced.sentencing_calculator = request.enabled,
        "mdl_proceedings" => features.experimental.mdl_proceedings = request.enabled,
        "ai_assisted_research" => features.experimental.ai_assisted_research = request.enabled,
        "automated_transcription" => features.experimental.automated_transcription = request.enabled,
        "predictive_analytics" => features.experimental.predictive_analytics = request.enabled,
        _ => return Err(ApiError::BadRequest(format!("Unknown feature: {}", request.feature_path))),
    }

    store.set_json(FEATURES_KEY, &features)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"success\": true}".to_vec())
        .build())
}

/// Get implementation tracker status
#[utoipa::path(
    get,
    path = "/api/features/implementation",
    responses(
        (status = 200, description = "Implementation status and progress tracking"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Implementation Tracking"
)]
pub fn get_implementation_status(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let mut tracker = if let Some(data) = store.get_json::<ImplementationTracker>(TRACKER_KEY)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        ImplementationTracker::new()
    };

    // Add default features if empty
    if tracker.features.is_empty() {
        tracker.add_feature(FeatureStatus {
            name: "Case Management".to_string(),
            module: "criminal_case".to_string(),
            status: ImplementationStatus::Completed,
            progress_percentage: 100,
            estimated_hours: 40,
            actual_hours: Some(45),
            blockers: Vec::new(),
            dependencies: Vec::new(),
        });

        tracker.add_feature(FeatureStatus {
            name: "Judge Assignment".to_string(),
            module: "judge".to_string(),
            status: ImplementationStatus::Completed,
            progress_percentage: 100,
            estimated_hours: 30,
            actual_hours: Some(32),
            blockers: Vec::new(),
            dependencies: Vec::new(),
        });

        tracker.add_feature(FeatureStatus {
            name: "Docket Management".to_string(),
            module: "docket".to_string(),
            status: ImplementationStatus::Completed,
            progress_percentage: 100,
            estimated_hours: 35,
            actual_hours: Some(38),
            blockers: Vec::new(),
            dependencies: Vec::new(),
        });

        tracker.add_feature(FeatureStatus {
            name: "Deadline Tracking".to_string(),
            module: "deadline".to_string(),
            status: ImplementationStatus::Completed,
            progress_percentage: 100,
            estimated_hours: 25,
            actual_hours: Some(28),
            blockers: Vec::new(),
            dependencies: Vec::new(),
        });

        store.set_json(TRACKER_KEY, &tracker)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;
    }

    let summary = tracker.get_summary();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&summary)?)
        .build())
}

/// Update implementation status for a feature
#[utoipa::path(
    patch,
    path = "/api/features/implementation",
    request_body(content = UpdateRequest, description = "Implementation status update"),
    responses(
        (status = 200, description = "Implementation status updated successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Implementation Tracking"
)]
pub fn update_implementation(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {

    let body = req.body();
    let request: UpdateRequest = serde_json::from_slice(body)?;

    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let mut tracker = if let Some(data) = store.get_json::<ImplementationTracker>(TRACKER_KEY)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        ImplementationTracker::new()
    };

    if let Some(status) = request.status {
        tracker.update_status(&request.feature_name, status);
    }

    if let Some(progress) = request.progress {
        tracker.update_progress(&request.feature_name, progress);
    }

    store.set_json(TRACKER_KEY, &tracker)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"success\": true}".to_vec())
        .build())
}

/// Get blocked features
#[utoipa::path(
    get,
    path = "/api/features/blocked",
    responses(
        (status = 200, description = "List of features that are currently blocked"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Implementation Tracking"
)]
pub fn get_blocked_features(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let tracker = if let Some(data) = store.get_json::<ImplementationTracker>(TRACKER_KEY)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        ImplementationTracker::new()
    };

    let blocked = tracker.get_blocked_features();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&blocked)?)
        .build())
}

/// Get features ready to start
#[utoipa::path(
    get,
    path = "/api/features/ready",
    responses(
        (status = 200, description = "List of features ready to start implementation"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Implementation Tracking"
)]
pub fn get_ready_features(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let tracker = if let Some(data) = store.get_json::<ImplementationTracker>(TRACKER_KEY)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        ImplementationTracker::new()
    };

    let ready = tracker.get_ready_to_start();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&ready)?)
        .build())
}

/// Create new FeatureManager instance
#[utoipa::path(
    post,
    path = "/api/features/manager",
    responses(
        (status = 201, description = "Feature manager created successfully", body = FeatureManager),
        (status = 500, description = "Internal server error")
    ),
    tag = "Feature Management"
)]
pub fn create_feature_manager(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let manager = FeatureManager::new();

    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    store.set_json("features:manager", &manager)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&manager)?)
        .build())
}

/// Check if a specific feature is enabled
#[utoipa::path(
    get,
    path = "/api/features/{feature_path}/enabled",
    params(
        ("feature_path" = String, Path, description = "Feature path to check")
    ),
    responses(
        (status = 200, description = "Feature enablement status"),
        (status = 400, description = "Feature path required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Feature Management"
)]
pub fn is_feature_enabled(_req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let feature_path = params
        .get("feature_path")
        .ok_or_else(|| ApiError::BadRequest("Feature path required".to_string()))?;

    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let manager = if let Some(data) = store.get_json::<FeatureManager>("features:manager")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        FeatureManager::new()
    };

    let is_enabled = manager.is_enabled(feature_path);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({
            "feature_path": feature_path,
            "enabled": is_enabled
        }))?)
        .build())
}

/// Set feature override
#[utoipa::path(
    post,
    path = "/api/features/override",
    request_body(content = OverrideRequest, description = "Feature override configuration"),
    responses(
        (status = 200, description = "Feature override set successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Feature Management"
)]
pub fn set_feature_override(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {

    let body = req.body();
    let request: OverrideRequest = serde_json::from_slice(body)?;

    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let mut manager = if let Some(data) = store.get_json::<FeatureManager>("features:manager")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        FeatureManager::new()
    };

    manager.set_override(&request.feature_path, request.enabled);

    store.set_json("features:manager", &manager)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"success\": true}".to_vec())
        .build())
}

/// Clear all feature overrides
#[utoipa::path(
    delete,
    path = "/api/features/override",
    responses(
        (status = 200, description = "All feature overrides cleared successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Feature Management"
)]
pub fn clear_feature_overrides(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let store = Store::open("default")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    let mut manager = if let Some(data) = store.get_json::<FeatureManager>("features:manager")
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))? {
        data
    } else {
        FeatureManager::new()
    };

    manager.clear_overrides();

    store.set_json("features:manager", &manager)
        .map_err(|e| ApiError::Internal(format!("Store error: {}", e)))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"overrides_cleared\": true}".to_vec())
        .build())
}