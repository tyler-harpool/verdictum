//! REST API handlers for judge management
//!
//! This module provides HTTP endpoints for managing judges, assignments,
//! recusals, and conflicts in the federal court system.

use crate::domain::judge::{
    Judge, JudgeTitle, JudgeStatus, CaseAssignment, RecusalMotion,
    ConflictOfInterest, JudgeConflictType, RecusalReason, RecusalStatus,
    AssignmentType, JudgeAssignmentService, CaseType
};
use crate::error::{ApiError, ApiResult};
use crate::ports::judge_repository::{
    JudgeRepository, CaseAssignmentRepository, RecusalRepository,
    ConflictRepository, JudgeQuery, JudgeQueryRepository
};
use crate::utils::{query_parser, repository_factory::RepositoryFactory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use uuid::Uuid;
use utoipa::ToSchema;

/// Response for conflict checking
#[derive(Serialize, ToSchema)]
pub struct ConflictCheckResponse {
    pub conflicts_found: bool,
    pub judges_with_conflicts: Vec<Uuid>,
    pub conflict_details: Vec<(Uuid, ConflictOfInterest)>,
}

/// Search response for judges
#[derive(Serialize, ToSchema)]
pub struct SearchResponse {
    pub judges: Vec<Judge>,
    pub total: usize,
}

/// Request to process recusal
#[derive(Deserialize, ToSchema)]
pub struct ProcessRequest {
    pub replacement_judge_id: Uuid,
    pub parties: Vec<String>,
}

/// Request model for creating a new judge
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateJudgeRequest {
    pub name: String,
    pub title: JudgeTitle,
    pub district: String,
    pub courtroom: String,
}

/// Request model for updating judge status
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateJudgeStatusRequest {
    pub status: JudgeStatus,
}

/// Request model for case assignment
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignCaseRequest {
    pub case_id: Uuid,
    pub case_type: CaseType,
    pub parties: Vec<String>,
    pub preferred_date: Option<DateTime<Utc>>,
    pub assignment_type: AssignmentType,
    pub reason: String,
}

/// Request model for filing a recusal motion
#[derive(Debug, Deserialize, ToSchema)]
pub struct FileRecusalRequest {
    pub case_id: Uuid,
    pub filed_by: String,
    pub reason: RecusalReason,
    pub detailed_grounds: String,
}

/// Request model for ruling on recusal
#[derive(Debug, Deserialize, ToSchema)]
pub struct RuleOnRecusalRequest {
    pub status: RecusalStatus,
    pub replacement_judge_id: Option<Uuid>,
}

/// Request model for adding conflict of interest
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddConflictRequest {
    pub party_name: Option<String>,
    pub law_firm: Option<String>,
    pub corporation: Option<String>,
    pub conflict_type: JudgeConflictType,
    pub notes: String,
}

/// Response model for judge workload statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkloadResponse {
    pub total_judges: usize,
    pub active_judges: usize,
    pub average_caseload: f32,
    pub overloaded_judges: usize,
    pub available_capacity: usize,
}

impl WorkloadResponse {
    pub fn from_statistics(stats: crate::ports::judge_repository::WorkloadStatistics) -> Self {
        Self {
            total_judges: stats.total_judges,
            active_judges: stats.active_judges,
            average_caseload: stats.average_caseload,
            overloaded_judges: stats.overloaded_judges,
            available_capacity: stats.available_capacity,
        }
    }
}

/// Create a new judge
#[utoipa::path(
    post,
    path = "/api/judges",
    request_body = CreateJudgeRequest,
    responses(
        (status = 201, description = "Judge created successfully", body = Judge),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judge Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_judge(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateJudgeRequest = serde_json::from_slice(body)?;

    let judge = Judge::new(
        request.name,
        request.title,
        request.district,
        request.courtroom,
    );

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    repo.save_judge(&judge)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judge)?)
        .build())
}

/// Get all judges
#[utoipa::path(
    get,
    path = "/api/judges",
    responses(
        (status = 200, description = "List of all judges", body = [Judge]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judge Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_all_judges(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::judge_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let judges = repo.find_all_judges()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judges)?)
        .build())
}

/// Get judge by ID
#[utoipa::path(
    get,
    path = "/api/judges/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "Judge found", body = Judge),
        (status = 404, description = "Judge not found"),
        (status = 400, description = "Invalid judge ID")
    ),
    tag = "Judge Management",
)]
pub fn get_judge_by_id(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let judge = repo
        .find_judge_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Judge not found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judge)?)
        .build())
}

/// Update judge status
#[utoipa::path(
    patch,
    path = "/api/judges/{id}/status",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Judge ID")
    ),
    request_body = UpdateJudgeStatusRequest,
    responses(
        (status = 200, description = "Judge status updated", body = Judge),
        (status = 404, description = "Judge not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Judge Management",
)]
pub fn update_judge_status(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let body = req.body();
    let request: UpdateJudgeStatusRequest = serde_json::from_slice(body)?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let mut judge = repo
        .find_judge_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Judge not found".to_string()))?;

    judge.update_status(request.status);
    repo.save_judge(&judge)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judge)?)
        .build())
}

/// Get available judges for assignment
#[utoipa::path(
    get,
    path = "/api/judges/available",
    responses(
        (status = 200, description = "List of available judges", body = [Judge]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judge Assignment",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_available_judges(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::judge_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let judges = repo.find_available_judges()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judges)?)
        .build())
}

/// Assign a case to a judge
#[utoipa::path(
    post,
    path = "/api/judges/assignments",
    request_body = AssignCaseRequest,
    responses(
        (status = 201, description = "Case assigned successfully", body = CaseAssignment),
        (status = 400, description = "Invalid assignment request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judge Assignment",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn assign_case(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: AssignCaseRequest = serde_json::from_slice(body)?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let available_judges = repo.find_available_judges()?;

    // Use the assignment service to find the best judge
    let judge_id = JudgeAssignmentService::assign_judge(
        &available_judges,
        request.case_type,
        &request.parties,
        request.preferred_date,
    ).map_err(|e| ApiError::BadRequest(e))?;

    // Create assignment record
    let assignment = JudgeAssignmentService::create_assignment(
        request.case_id,
        judge_id,
        request.assignment_type,
        request.reason,
    );

    // Update judge's caseload
    if let Ok(Some(mut judge)) = repo.find_judge_by_id(judge_id) {
        let _ = judge.assign_case();
        repo.save_judge(&judge)?;
    }

    // Save assignment
    repo.save_assignment(&assignment)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&assignment)?)
        .build())
}

/// Get case assignment
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/assignment",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "Case assignment found", body = CaseAssignment),
        (status = 404, description = "Assignment not found"),
        (status = 400, description = "Invalid case ID")
    ),
    tag = "Judge Assignment",
)]
pub fn get_case_assignment(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let assignment = repo
        .find_assignment_by_case(case_id)?
        .ok_or_else(|| ApiError::NotFound("Assignment not found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&assignment)?)
        .build())
}

/// File a recusal motion
#[utoipa::path(
    post,
    path = "/api/judges/{judge_id}/recusals",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = Uuid, Path, description = "Judge ID")
    ),
    request_body = FileRecusalRequest,
    responses(
        (status = 201, description = "Recusal motion filed", body = RecusalMotion),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Judge not found")
    ),
    tag = "Recusal Management",
)]
pub fn file_recusal(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let body = req.body();
    let request: FileRecusalRequest = serde_json::from_slice(body)?;

    let motion = RecusalMotion {
        id: Uuid::new_v4(),
        case_id: request.case_id,
        judge_id,
        filed_by: request.filed_by,
        filed_date: Utc::now(),
        reason: request.reason,
        detailed_grounds: request.detailed_grounds,
        status: RecusalStatus::Pending,
        ruling_date: None,
        replacement_judge_id: None,
    };

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    repo.save_recusal(&motion)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&motion)?)
        .build())
}

/// Rule on a recusal motion
#[utoipa::path(
    patch,
    path = "/api/recusals/{recusal_id}/ruling",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("recusal_id" = Uuid, Path, description = "Recusal motion ID")
    ),
    request_body = RuleOnRecusalRequest,
    responses(
        (status = 200, description = "Recusal ruling updated", body = RecusalMotion),
        (status = 404, description = "Recusal motion not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Recusal Management",
)]
pub fn rule_on_recusal(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let recusal_id = params
        .get("recusal_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid recusal ID".to_string()))?;

    let body = req.body();
    let request: RuleOnRecusalRequest = serde_json::from_slice(body)?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let mut motion = repo
        .find_recusal_by_id(recusal_id)?
        .ok_or_else(|| ApiError::NotFound("Recusal motion not found".to_string()))?;

    motion.status = request.status;
    motion.ruling_date = Some(Utc::now());

    if request.status == RecusalStatus::Granted {
        // If granted, handle judge replacement
        if let Some(replacement_id) = request.replacement_judge_id {
            motion.replacement_judge_id = Some(replacement_id);

            // Update assignment
            if let Ok(Some(_assignment)) = repo.find_assignment_by_case(motion.case_id) {
                let new_assignment = CaseAssignment {
                    id: Uuid::new_v4(),
                    case_id: motion.case_id,
                    judge_id: replacement_id,
                    assignment_type: AssignmentType::Reassignment,
                    assigned_date: Utc::now(),
                    reason: format!("Reassigned due to recusal: {}", motion.detailed_grounds),
                    previous_judge_id: Some(motion.judge_id),
                    reassignment_reason: Some("Recusal granted".to_string()),
                };
                repo.save_assignment(&new_assignment)?;

                // Update caseloads
                if let Ok(Some(mut old_judge)) = repo.find_judge_by_id(motion.judge_id) {
                    old_judge.unassign_case();
                    repo.save_judge(&old_judge)?;
                }
                if let Ok(Some(mut new_judge)) = repo.find_judge_by_id(replacement_id) {
                    let _ = new_judge.assign_case();
                    repo.save_judge(&new_judge)?;
                }
            }
        }
    }

    repo.save_recusal(&motion)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&motion)?)
        .build())
}

/// Get pending recusals
#[utoipa::path(
    get,
    path = "/api/recusals/pending",
    responses(
        (status = 200, description = "List of pending recusal motions", body = [RecusalMotion]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Recusal Management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_pending_recusals(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::judge_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let recusals = repo.find_pending_recusals()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&recusals)?)
        .build())
}

/// Add conflict of interest
#[utoipa::path(
    post,
    path = "/api/judges/{judge_id}/conflicts",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = Uuid, Path, description = "Judge ID")
    ),
    request_body = AddConflictRequest,
    responses(
        (status = 201, description = "Conflict of interest added", body = ConflictOfInterest),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Judge not found")
    ),
    tag = "Conflict Management",
)]
pub fn add_conflict(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let body = req.body();
    let request: AddConflictRequest = serde_json::from_slice(body)?;

    let conflict = ConflictOfInterest {
        id: Uuid::new_v4(),
        party_name: request.party_name,
        law_firm: request.law_firm,
        corporation: request.corporation,
        conflict_type: request.conflict_type,
        start_date: Utc::now(),
        end_date: None,
        notes: request.notes,
    };

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    repo.save_conflict(judge_id, &conflict)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&conflict)?)
        .build())
}

/// Check for conflicts with a party
#[utoipa::path(
    get,
    path = "/api/conflicts/check/{party}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("party" = String, Path, description = "Party name to check for conflicts")
    ),
    responses(
        (status = 200, description = "Conflict check results"),
        (status = 400, description = "Party name required")
    ),
    tag = "Conflict Management",
)]
pub fn check_conflicts(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let party_name = params
        .get("party")
        .ok_or_else(|| ApiError::BadRequest("Party name required".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let conflicts = repo.find_conflicts_by_party(party_name)?;


    let response = ConflictCheckResponse {
        conflicts_found: !conflicts.is_empty(),
        judges_with_conflicts: conflicts.iter().map(|(judge_id, _)| *judge_id).collect(),
        conflict_details: conflicts,
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get workload statistics
#[utoipa::path(
    get,
    path = "/api/judges/workload",
    responses(
        (status = 200, description = "Judge workload statistics", body = WorkloadResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judge Analytics",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_workload_stats(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = match RepositoryFactory::judge_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let stats = repo.get_workload_statistics()?;
    let response = WorkloadResponse::from_statistics(stats);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Search judges with filters
#[utoipa::path(
    get,
    path = "/api/judges/search",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("status" = Option<String>, Query, description = "Filter by judge status"),
        ("title" = Option<String>, Query, description = "Filter by judge title"),
        ("district" = Option<String>, Query, description = "Filter by district"),
        ("accepts_criminal" = Option<bool>, Query, description = "Filter by criminal case acceptance"),
        ("accepts_civil" = Option<bool>, Query, description = "Filter by civil case acceptance"),
        ("max_caseload" = Option<u32>, Query, description = "Maximum caseload percentage"),
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit")
    ),
    responses(
        (status = 200, description = "Search results with judges and total count"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judge Management",
)]
pub fn search_judges(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let params = query_parser::parse_query_string(query_string);

    let query = JudgeQuery {
        status: query_parser::get_json(&params, "status"),
        title: query_parser::get_json(&params, "title"),
        district: query_parser::get_string(&params, "district"),
        accepts_criminal: query_parser::get_bool(&params, "accepts_criminal"),
        accepts_civil: query_parser::get_bool(&params, "accepts_civil"),
        max_caseload_percentage: query_parser::get_usize(&params, "max_caseload").map(|u| u as f32),
        offset: query_parser::get_usize(&params, "offset").unwrap_or(0),
        limit: query_parser::get_usize(&params, "limit").unwrap_or(50),
    };

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let (judges, total) = repo.search_judges(query)?;


    let response = SearchResponse { judges, total };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get judges by status
#[utoipa::path(
    get,
    path = "/api/judges/status/{status}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("status" = String, Path, description = "Judge status (active, inactive, retired, vacation)")
    ),
    responses(
        (status = 200, description = "List of judges with specified status", body = [Judge]),
        (status = 400, description = "Invalid status")
    ),
    tag = "Judge Management",
)]
pub fn get_judges_by_status(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let status_str = params
        .get("status")
        .ok_or_else(|| ApiError::BadRequest("Status required".to_string()))?;

    let status: JudgeStatus = serde_json::from_str(&format!("\"{}\"", status_str))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let judges = repo.find_judges_by_status(status)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judges)?)
        .build())
}

/// Get judges by district
#[utoipa::path(
    get,
    path = "/api/judges/district/{district}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("district" = String, Path, description = "District name")
    ),
    responses(
        (status = 200, description = "List of judges in the specified district", body = [Judge]),
        (status = 400, description = "District required")
    ),
    tag = "Judge Management",
)]
pub fn get_judges_by_district(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let district = params
        .get("district")
        .ok_or_else(|| ApiError::BadRequest("District required".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let judges = repo.find_judges_by_district(district)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judges)?)
        .build())
}

/// Delete a judge
#[utoipa::path(
    delete,
    path = "/api/judges/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "Judge deleted successfully"),
        (status = 404, description = "Judge not found"),
        (status = 400, description = "Invalid judge ID")
    ),
    tag = "Judge Management",
)]
pub fn delete_judge(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let deleted = repo.delete_judge(id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"deleted": deleted}))?)
        .build())
}

/// Get assignment history for a case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/assignment-history",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "Assignment history for the case", body = [CaseAssignment]),
        (status = 400, description = "Invalid case ID")
    ),
    tag = "Judge Assignment",
)]
pub fn get_assignment_history(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let history = repo.find_assignment_history(case_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&history)?)
        .build())
}

/// Delete an assignment
#[utoipa::path(
    delete,
    path = "/api/assignments/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Assignment deleted successfully"),
        (status = 404, description = "Assignment not found"),
        (status = 400, description = "Invalid assignment ID")
    ),
    tag = "Judge Assignment",
)]
pub fn delete_assignment(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid assignment ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let deleted = repo.delete_assignment(id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"deleted": deleted}))?)
        .build())
}

/// Get recusals by case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/recusals",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of recusal motions for the case", body = [RecusalMotion]),
        (status = 400, description = "Invalid case ID")
    ),
    tag = "Recusal Management",
)]
pub fn get_recusals_by_case(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let recusals = repo.find_recusals_by_case(case_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&recusals)?)
        .build())
}

/// Get recusals by judge
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/recusals",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = Uuid, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "List of recusal motions for the judge", body = [RecusalMotion]),
        (status = 400, description = "Invalid judge ID")
    ),
    tag = "Recusal Management",
)]
pub fn get_recusals_by_judge(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let recusals = repo.find_recusals_by_judge(judge_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&recusals)?)
        .build())
}

/// Get conflicts by judge
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/conflicts",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = Uuid, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "List of conflicts of interest for the judge", body = [ConflictOfInterest]),
        (status = 400, description = "Invalid judge ID")
    ),
    tag = "Conflict Management",
)]
pub fn get_conflicts_by_judge(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let conflicts = repo.find_conflicts_by_judge(judge_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&conflicts)?)
        .build())
}

/// Check if judge has conflict with party
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/conflicts/check/{party}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = Uuid, Path, description = "Judge ID"),
        ("party" = String, Path, description = "Party name to check for conflicts")
    ),
    responses(
        (status = 200, description = "Conflict check result"),
        (status = 400, description = "Invalid judge ID or party name required")
    ),
    tag = "Conflict Management",
)]
pub fn has_conflict(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let party_name = params
        .get("party")
        .ok_or_else(|| ApiError::BadRequest("Party name required".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let has_conflict = repo.has_conflict(judge_id, party_name)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"has_conflict": has_conflict}))?)
        .build())
}

/// Delete a conflict
#[utoipa::path(
    delete,
    path = "/api/judges/{judge_id}/conflicts/{conflict_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = Uuid, Path, description = "Judge ID"),
        ("conflict_id" = Uuid, Path, description = "Conflict ID")
    ),
    responses(
        (status = 200, description = "Conflict deleted successfully"),
        (status = 404, description = "Judge or conflict not found"),
        (status = 400, description = "Invalid judge ID or conflict ID")
    ),
    tag = "Conflict Management",
)]
pub fn delete_conflict(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let conflict_id = params
        .get("conflict_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid conflict ID".to_string()))?;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let deleted = repo.delete_conflict(judge_id, conflict_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"deleted": deleted}))?)
        .build())
}

/// Get judges on vacation
#[utoipa::path(
    get,
    path = "/api/judges/vacation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("start" = Option<String>, Query, description = "Start date (RFC3339 format)"),
        ("end" = Option<String>, Query, description = "End date (RFC3339 format)")
    ),
    responses(
        (status = 200, description = "List of judges on vacation during the specified period", body = [Judge]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judge Management",
)]
pub fn get_judges_on_vacation(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let query_params = query_parser::parse_query_string(query_string);

    let start_date = query_parser::get_datetime(&query_params, "start")
        .unwrap_or_else(|| Utc::now());
    let end_date = query_parser::get_datetime(&query_params, "end")
        .unwrap_or_else(|| Utc::now() + chrono::Duration::days(30));

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let judges = repo.find_judges_on_vacation(start_date, end_date)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&judges)?)
        .build())
}

/// Process a recusal
#[utoipa::path(
    post,
    path = "/api/recusals/{recusal_id}/process",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("recusal_id" = Uuid, Path, description = "Recusal motion ID")
    ),
    request_body(content = ProcessRequest, description = "Processing details including replacement judge and parties"),
    responses(
        (status = 200, description = "Recusal processed and new assignment created", body = CaseAssignment),
        (status = 404, description = "Recusal motion not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Recusal Management",
)]
pub fn process_recusal(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let recusal_id = params
        .get("recusal_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid recusal ID".to_string()))?;


    let body = req.body();
    let request: ProcessRequest = serde_json::from_slice(body)?;

    // Log the requested replacement judge for audit purposes
    let _replacement_id = request.replacement_judge_id;

    let repo = match RepositoryFactory::judge_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let motion = repo
        .find_recusal_by_id(recusal_id)?
        .ok_or_else(|| ApiError::NotFound("Recusal motion not found".to_string()))?;

    // Get available judges
    let available_judges = repo.find_available_judges()?;

    // Process the recusal
    let replacement_judge_id = JudgeAssignmentService::process_recusal(
        &motion,
        &available_judges,
        &request.parties,
    ).map_err(|e| ApiError::BadRequest(e))?;

    // Create new assignment
    let new_assignment = CaseAssignment {
        id: Uuid::new_v4(),
        case_id: motion.case_id,
        judge_id: replacement_judge_id,
        assignment_type: AssignmentType::Reassignment,
        assigned_date: Utc::now(),
        reason: format!("Reassigned due to recusal: {}", motion.detailed_grounds),
        previous_judge_id: Some(motion.judge_id),
        reassignment_reason: Some("Recusal processed".to_string()),
    };

    repo.save_assignment(&new_assignment)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&new_assignment)?)
        .build())
}
