//! REST API handlers for deadline tracking and compliance
//!
//! This module provides HTTP endpoints for managing deadlines,
//! extensions, and compliance reporting in the federal court system.

use crate::utils::repository_factory::RepositoryFactory;
use crate::domain::deadline::{
    Deadline, DeadlineType, DeadlineStatus, ExtensionRequest,
    ExtensionStatus, DeadlineCalculator, DeadlineMonitor, FederalRule
};
use crate::error::{ApiError, ApiResult};
use crate::ports::deadline_repository::{
    DeadlineRepository, ExtensionRepository, ReminderRepository,
    DeadlineQuery, DeadlineComplianceRepository
};
use crate::utils::query_parser;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use uuid::Uuid;
use utoipa::ToSchema;

/// Request to update deadline status
#[derive(Deserialize, ToSchema)]
pub struct StatusUpdate {
    pub status: DeadlineStatus,
}

/// Search response for deadlines
#[derive(Serialize, ToSchema)]
pub struct SearchResponse {
    pub deadlines: Vec<Deadline>,
    pub total: usize,
}

/// Response for reminder sending
#[derive(Serialize, ToSchema)]
pub struct ReminderResponse {
    pub sent_count: usize,
    pub recipients: Vec<String>,
}

/// Response for pending extension requests
#[derive(Serialize, ToSchema)]
pub struct PendingExtensionsResponse {
    pub extensions: Vec<(Uuid, ExtensionRequest)>,
    pub total: usize,
}

/// Request model for creating a deadline
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDeadlineRequest {
    pub case_id: Uuid,
    pub deadline_type: DeadlineType,
    pub due_date: DateTime<Utc>,
    pub triggering_event: String,
    pub triggering_date: DateTime<Utc>,
    pub applicable_rule: String,
    pub description: String,
    pub responsible_party: String,
    pub is_jurisdictional: bool,
    pub is_extendable: bool,
}

/// Request model for requesting an extension
#[derive(Debug, Deserialize, ToSchema)]
pub struct RequestExtensionRequest {
    pub requested_by: String,
    pub new_due_date: DateTime<Utc>,
    pub reason: String,
    pub opposed_by: Vec<String>,
}

/// Request model for ruling on extension
#[derive(Debug, Deserialize, ToSchema)]
pub struct RuleOnExtensionRequest {
    pub status: ExtensionStatus,
    pub order_text: Option<String>,
}

/// Request model for calculating FRCP deadlines
#[derive(Debug, Deserialize, ToSchema)]
pub struct CalculateDeadlinesRequest {
    pub triggering_event: String,
    pub triggering_date: DateTime<Utc>,
    pub case_id: Uuid,
}

/// Response model for compliance statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct ComplianceStatsResponse {
    pub total_deadlines: usize,
    pub completed_on_time: usize,
    pub completed_late: usize,
    pub pending: usize,
    pub overdue: usize,
    pub compliance_rate: f32,
}

impl ComplianceStatsResponse {
    pub fn from_statistics(stats: crate::ports::deadline_repository::ComplianceStatistics) -> Self {
        Self {
            total_deadlines: stats.total_deadlines,
            completed_on_time: stats.completed_on_time,
            completed_late: stats.completed_late,
            pending: stats.pending,
            overdue: stats.overdue,
            compliance_rate: stats.compliance_rate,
        }
    }
}

/// Create a new deadline
#[utoipa::path(
    post,
    path = "/api/deadlines",
    request_body = CreateDeadlineRequest,
    responses(
        (status = 201, description = "Deadline created successfully", body = Deadline),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_deadline(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateDeadlineRequest = serde_json::from_slice(body)?;

    let deadline = Deadline {
        id: Uuid::new_v4(),
        case_id: request.case_id,
        deadline_type: request.deadline_type,
        due_date: request.due_date,
        triggering_event: request.triggering_event,
        triggering_date: request.triggering_date,
        applicable_rule: request.applicable_rule,
        description: request.description,
        responsible_party: request.responsible_party,
        is_jurisdictional: request.is_jurisdictional,
        is_extendable: request.is_extendable,
        status: DeadlineStatus::Pending,
        completion_date: None,
        extension_requests: Vec::new(),
        reminders_sent: Vec::new(),
    };

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    repo.save_deadline(&deadline)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&deadline)?)
        .build())
}

/// Get deadlines for a case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/deadlines",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of deadlines for the case", body = [Deadline]),
        (status = 400, description = "Invalid case ID"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Management",
)]
pub fn get_case_deadlines(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let mut deadlines = repo.find_deadlines_by_case(case_id)?;

    // Update statuses
    DeadlineMonitor::update_deadline_statuses(&mut deadlines, Utc::now());

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&deadlines)?)
        .build())
}

/// Get a specific deadline
#[utoipa::path(
    get,
    path = "/api/deadlines/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Deadline ID")
    ),
    responses(
        (status = 200, description = "Deadline found", body = Deadline),
        (status = 404, description = "Deadline not found"),
        (status = 400, description = "Invalid deadline ID")
    ),
    tag = "Deadline Management",
)]
pub fn get_deadline(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid deadline ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let deadline = repo
        .find_deadline_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Deadline not found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&deadline)?)
        .build())
}

/// Mark deadline as completed
#[utoipa::path(
    patch,
    path = "/api/deadlines/{id}/complete",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Deadline ID")
    ),
    responses(
        (status = 200, description = "Deadline marked as completed", body = Deadline),
        (status = 404, description = "Deadline not found"),
        (status = 400, description = "Invalid deadline ID")
    ),
    tag = "Deadline Management",
)]
pub fn complete_deadline(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid deadline ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    repo.complete_deadline(id, Utc::now())?;

    let deadline = repo
        .find_deadline_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Deadline not found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&deadline)?)
        .build())
}

/// Request extension for a deadline
#[utoipa::path(
    post,
    path = "/api/deadlines/{deadline_id}/extensions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("deadline_id" = Uuid, Path, description = "Deadline ID")
    ),
    request_body = RequestExtensionRequest,
    responses(
        (status = 201, description = "Extension request submitted", body = ExtensionRequest),
        (status = 404, description = "Deadline not found"),
        (status = 400, description = "Invalid request data or deadline not extendable")
    ),
    tag = "Extension Management",
)]
pub fn request_extension(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let deadline_id = params
        .get("deadline_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid deadline ID".to_string()))?;

    let body = req.body();
    let request: RequestExtensionRequest = serde_json::from_slice(body)?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let deadline = repo
        .find_deadline_by_id(deadline_id)?
        .ok_or_else(|| ApiError::NotFound("Deadline not found".to_string()))?;

    if !deadline.is_extendable {
        return Err(ApiError::BadRequest("This deadline is not extendable".to_string()));
    }

    if deadline.is_jurisdictional {
        return Err(ApiError::BadRequest("Jurisdictional deadlines cannot be extended".to_string()));
    }

    let extension = ExtensionRequest {
        id: Uuid::new_v4(),
        requested_date: Utc::now(),
        requested_by: request.requested_by,
        new_due_date: request.new_due_date,
        reason: request.reason,
        opposed_by: request.opposed_by,
        status: ExtensionStatus::Pending,
        ruling_date: None,
        order_text: None,
    };

    repo.save_extension(deadline_id, &extension)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&extension)?)
        .build())
}

/// Rule on extension request
#[utoipa::path(
    patch,
    path = "/api/extensions/{extension_id}/ruling",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("extension_id" = Uuid, Path, description = "Extension request ID")
    ),
    request_body = RuleOnExtensionRequest,
    responses(
        (status = 200, description = "Extension ruling recorded"),
        (status = 404, description = "Extension request not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Extension Management",
)]
pub fn rule_on_extension(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let extension_id = params
        .get("extension_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid extension ID".to_string()))?;

    let body = req.body();
    let request: RuleOnExtensionRequest = serde_json::from_slice(body)?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let status = request.status.clone();
    repo.update_extension_status(extension_id, status)?;

    // If there's order text, we could store it or process it
    if let Some(order_text) = request.order_text {
        // In a real system, this would be stored with the ruling
        println!("Extension ruling order: {}", order_text);
    }

    // If granted, update the deadline's due date
    if request.status == ExtensionStatus::Granted {
        // Find the deadline with this extension
        let deadlines = repo.find_deadlines_by_status(DeadlineStatus::Pending)?;
        for mut deadline in deadlines {
            if let Some(ext) = deadline.extension_requests.iter().find(|e| e.id == extension_id) {
                if request.status == ExtensionStatus::Granted {
                    deadline.due_date = ext.new_due_date;
                    deadline.status = DeadlineStatus::Extended;
                    repo.save_deadline(&deadline)?;
                    break;
                }
            }
        }
    }

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"success\": true}".to_vec())
        .build())
}

/// Get upcoming deadlines
#[utoipa::path(
    get,
    path = "/api/deadlines/upcoming",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("days" = Option<i64>, Query, description = "Number of days to look ahead (default: 30)")
    ),
    responses(
        (status = 200, description = "List of upcoming deadlines", body = [Deadline]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Management",
)]
pub fn get_upcoming_deadlines(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let days = params
        .get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(30);

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let mut deadlines = repo.find_upcoming_deadlines(days)?;

    // Update statuses
    DeadlineMonitor::update_deadline_statuses(&mut deadlines, Utc::now());

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&deadlines)?)
        .build())
}

/// Get urgent deadlines
#[utoipa::path(
    get,
    path = "/api/deadlines/urgent",
    responses(
        (status = 200, description = "List of urgent deadlines requiring immediate attention", body = [Deadline]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_urgent_deadlines(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::deadline_repo(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let all_deadlines = repo.find_deadlines_by_status(DeadlineStatus::Pending)?;
    let urgent = DeadlineMonitor::get_urgent_deadlines(&all_deadlines);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&urgent)?)
        .build())
}

/// Calculate FRCP deadlines
#[utoipa::path(
    post,
    path = "/api/deadlines/calculate-frcp",
    request_body = CalculateDeadlinesRequest,
    responses(
        (status = 201, description = "FRCP deadlines calculated and created", body = [Deadline]),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Calculation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn calculate_frcp_deadlines(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CalculateDeadlinesRequest = serde_json::from_slice(body)?;

    let mut deadlines = DeadlineCalculator::calculate_frcp_deadlines(
        &request.triggering_event,
        request.triggering_date,
    );

    // Set the case ID for all calculated deadlines
    for deadline in &mut deadlines {
        deadline.case_id = request.case_id;
    }

    // Save all calculated deadlines
    let repo = match RepositoryFactory::deadline_repo(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    for deadline in &deadlines {
        repo.save_deadline(deadline)?;
    }

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&deadlines)?)
        .build())
}

/// Search deadlines
#[utoipa::path(
    get,
    path = "/api/deadlines/search",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Option<Uuid>, Query, description = "Filter by case ID"),
        ("type" = Option<String>, Query, description = "Filter by deadline type"),
        ("status" = Option<String>, Query, description = "Filter by deadline status"),
        ("party" = Option<String>, Query, description = "Filter by responsible party"),
        ("jurisdictional" = Option<bool>, Query, description = "Filter by jurisdictional deadlines"),
        ("from" = Option<String>, Query, description = "Due date from (RFC3339 format)"),
        ("to" = Option<String>, Query, description = "Due date to (RFC3339 format)"),
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit")
    ),
    responses(
        (status = 200, description = "Search results with deadlines and total count"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Management",
)]
pub fn search_deadlines(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let params = query_parser::parse_query_string(query_string);

    let query = DeadlineQuery {
        case_id: query_parser::get_uuid(&params, "case_id"),
        deadline_type: query_parser::get_json(&params, "type"),
        status: query_parser::get_json(&params, "status"),
        responsible_party: query_parser::get_string(&params, "party"),
        is_jurisdictional: query_parser::get_bool(&params, "jurisdictional"),
        due_date_from: query_parser::get_datetime(&params, "from"),
        due_date_to: query_parser::get_datetime(&params, "to"),
        offset: query_parser::get_usize(&params, "offset").unwrap_or(0),
        limit: query_parser::get_usize(&params, "limit").unwrap_or(50),
    };

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let (deadlines, total) = repo.search_deadlines(query)?;


    let response = SearchResponse { deadlines, total };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get compliance statistics
#[utoipa::path(
    get,
    path = "/api/deadlines/compliance-stats",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Option<Uuid>, Query, description = "Filter by case ID (optional)")
    ),
    responses(
        (status = 200, description = "Deadline compliance statistics", body = ComplianceStatsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Analytics",
)]
pub fn get_compliance_stats(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok());

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let stats = repo.get_compliance_statistics(case_id)?;
    let response = ComplianceStatsResponse::from_statistics(stats);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Generate compliance report
#[utoipa::path(
    get,
    path = "/api/deadlines/compliance-report",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("start" = Option<String>, Query, description = "Start date (RFC3339 format, defaults to 30 days ago)"),
        ("end" = Option<String>, Query, description = "End date (RFC3339 format, defaults to now)")
    ),
    responses(
        (status = 200, description = "Comprehensive compliance report"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Analytics",
)]
pub fn generate_compliance_report(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let start_date = params
        .get("start")
        .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));

    let end_date = params
        .get("end")
        .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let report = repo.generate_compliance_report(start_date, end_date)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&report)?)
        .build())
}

/// Get performance metrics
#[utoipa::path(
    get,
    path = "/api/deadlines/performance-metrics",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("party" = Option<String>, Query, description = "Filter by responsible party")
    ),
    responses(
        (status = 200, description = "Performance metrics for deadline compliance"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Analytics",
)]
pub fn get_performance_metrics(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let party = params.get("party").map(String::from);

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let metrics = repo.get_performance_metrics(party)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&metrics)?)
        .build())
}

/// Find missed jurisdictional deadlines
#[utoipa::path(
    get,
    path = "/api/deadlines/missed-jurisdictional",
    responses(
        (status = 200, description = "List of missed jurisdictional deadlines", body = [Deadline]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Compliance",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_missed_jurisdictional(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::deadline_repo(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let missed = repo.find_missed_jurisdictional()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&missed)?)
        .build())
}

/// Get pending reminders
#[utoipa::path(
    get,
    path = "/api/deadlines/reminders/pending",
    responses(
        (status = 200, description = "List of pending deadline reminders"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reminder Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_pending_reminders(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::deadline_repo(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let reminders = repo.get_pending_reminders()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&reminders)?)
        .build())
}

/// Send reminders (typically called by a scheduled job)
#[utoipa::path(
    post,
    path = "/api/deadlines/reminders/send",
    responses(
        (status = 200, description = "Reminders sent successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reminder Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn send_reminders(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::deadline_repo(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let reminders = repo.get_pending_reminders()?;

    // In a real system, this would send emails/notifications
    // For now, just save them as sent
    repo.save_reminders(&reminders)?;


    let response = ReminderResponse {
        sent_count: reminders.len(),
        recipients: reminders.iter().map(|r| r.recipient.clone()).collect(),
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get deadlines by type for a case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/deadlines/type/{type}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Uuid, Path, description = "Case ID"),
        ("type" = String, Path, description = "Deadline type")
    ),
    responses(
        (status = 200, description = "List of deadlines of the specified type", body = [Deadline]),
        (status = 400, description = "Invalid case ID or deadline type")
    ),
    tag = "Deadline Management",
)]
pub fn get_deadlines_by_type(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let deadline_type_str = params
        .get("type")
        .ok_or_else(|| ApiError::BadRequest("Deadline type required".to_string()))?;

    let deadline_type: DeadlineType = serde_json::from_str(&format!("\"{}\"", deadline_type_str))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let deadlines = repo.find_deadlines_by_type(case_id, deadline_type)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&deadlines)?)
        .build())
}

/// Update deadline status
#[utoipa::path(
    patch,
    path = "/api/deadlines/{id}/status",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Deadline ID")
    ),
    request_body(content = StatusUpdate, description = "New deadline status"),
    responses(
        (status = 200, description = "Deadline status updated successfully"),
        (status = 404, description = "Deadline not found"),
        (status = 400, description = "Invalid deadline ID or status")
    ),
    tag = "Deadline Management",
)]
pub fn update_deadline_status(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid deadline ID".to_string()))?;


    let body = req.body();
    let update: StatusUpdate = serde_json::from_slice(body)?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    repo.update_deadline_status(id, update.status)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"success\": true}".to_vec())
        .build())
}

/// Delete a deadline
#[utoipa::path(
    delete,
    path = "/api/deadlines/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Deadline ID")
    ),
    responses(
        (status = 200, description = "Deadline deleted successfully"),
        (status = 404, description = "Deadline not found"),
        (status = 400, description = "Invalid deadline ID")
    ),
    tag = "Deadline Management",
)]
pub fn delete_deadline(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid deadline ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let deleted = repo.delete_deadline(id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"deleted": deleted}))?)
        .build())
}

/// Get reminders by deadline
#[utoipa::path(
    get,
    path = "/api/deadlines/{deadline_id}/reminders",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("deadline_id" = Uuid, Path, description = "Deadline ID")
    ),
    responses(
        (status = 200, description = "List of reminders for the deadline"),
        (status = 400, description = "Invalid deadline ID")
    ),
    tag = "Reminder Management",
)]
pub fn get_reminders_by_deadline(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let deadline_id = params
        .get("deadline_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid deadline ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let reminders = repo.find_reminders_by_deadline(deadline_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&reminders)?)
        .build())
}

/// Get reminders by recipient
#[utoipa::path(
    get,
    path = "/api/deadlines/reminders/recipient/{recipient}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("recipient" = String, Path, description = "Recipient name or email")
    ),
    responses(
        (status = 200, description = "List of reminders for the recipient"),
        (status = 400, description = "Recipient required")
    ),
    tag = "Reminder Management",
)]
pub fn get_reminders_by_recipient(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let recipient = params
        .get("recipient")
        .ok_or_else(|| ApiError::BadRequest("Recipient required".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let reminders = repo.find_reminders_by_recipient(recipient)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&reminders)?)
        .build())
}

/// Acknowledge a reminder
#[utoipa::path(
    patch,
    path = "/api/deadlines/reminders/{reminder_id}/acknowledge",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("reminder_id" = Uuid, Path, description = "Reminder ID")
    ),
    responses(
        (status = 200, description = "Reminder acknowledged successfully"),
        (status = 404, description = "Reminder not found"),
        (status = 400, description = "Invalid reminder ID")
    ),
    tag = "Reminder Management",
)]
pub fn acknowledge_reminder(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let reminder_id = params
        .get("reminder_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid reminder ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    repo.acknowledge_reminder(reminder_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"acknowledged\": true}".to_vec())
        .build())
}

/// Get extension by ID
#[utoipa::path(
    get,
    path = "/api/extensions/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Extension request ID")
    ),
    responses(
        (status = 200, description = "Extension request found", body = ExtensionRequest),
        (status = 404, description = "Extension request not found"),
        (status = 400, description = "Invalid extension ID")
    ),
    tag = "Extension Management",
)]
pub fn get_extension_by_id(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid extension ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let extension = repo
        .find_extension_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Extension not found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&extension)?)
        .build())
}

/// Get extensions by deadline
#[utoipa::path(
    get,
    path = "/api/deadlines/{deadline_id}/extensions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("deadline_id" = Uuid, Path, description = "Deadline ID")
    ),
    responses(
        (status = 200, description = "List of extension requests for the deadline", body = [ExtensionRequest]),
        (status = 400, description = "Invalid deadline ID")
    ),
    tag = "Extension Management",
)]
pub fn get_extensions_by_deadline(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let deadline_id = params
        .get("deadline_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid deadline ID".to_string()))?;

    let repo = match RepositoryFactory::deadline_repo(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let extensions = repo.find_extensions_by_deadline(deadline_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&extensions)?)
        .build())
}

/// Get pending extensions
#[utoipa::path(
    get,
    path = "/api/extensions/pending",
    responses(
        (status = 200, description = "List of pending extension requests"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Extension Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_pending_extensions(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::deadline_repo(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let extensions = repo.find_pending_extensions()?;


    let response = PendingExtensionsResponse {
        total: extensions.len(),
        extensions,
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get federal rules database
#[utoipa::path(
    get,
    path = "/api/deadlines/federal-rules",
    responses(
        (status = 200, description = "List of federal rules for deadline calculation", body = [FederalRule]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Deadline Calculation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_federal_rules(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {

    let rules = vec![
        FederalRule {
            rule_number: "FRCP 12(a)(1)(A)".to_string(),
            title: "Answer to complaint".to_string(),
            days_to_respond: 21,
            is_calendar_days: true,
            includes_weekends: false,
            service_adds_days: 3,
        },
        FederalRule {
            rule_number: "FRCP 56(b)".to_string(),
            title: "Motion for summary judgment".to_string(),
            days_to_respond: 30,
            is_calendar_days: false,
            includes_weekends: false,
            service_adds_days: 0,
        },
        FederalRule {
            rule_number: "FRCrP 45(c)".to_string(),
            title: "Speedy Trial - Trial must commence".to_string(),
            days_to_respond: 70,
            is_calendar_days: true,
            includes_weekends: true,
            service_adds_days: 0,
        },
    ];

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&rules)?)
        .build())
}
