//! Criminal case management API handlers
//!
//! This module demonstrates hexagonal architecture by using repository pattern
//! for all data access, keeping the handlers decoupled from storage implementation.

use crate::domain::criminal_case::{CaseStatus, CasePriority, CrimeType, CriminalCase, PleaType, EventType, MotionType};
use crate::error::{ApiError, ApiResult};
use crate::ports::case_repository::{CaseRepository, CaseQuery, CaseQueryRepository};
use crate::utils::repository_factory::RepositoryFactory;
use crate::utils::json_response as json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use uuid::Uuid;
use utoipa::ToSchema;

/// Request model for creating a new case
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "title": "Cyber Attack on Banking System",
    "description": "Unauthorized access to customer database detected",
    "crimeType": "cybercrime",
    "assignedJudge": "Judge Smith",
    "location": "New York, NY"
}))]
pub struct CreateCaseRequest {
    pub title: String,
    pub description: String,
    #[serde(rename = "crimeType")]
    pub crime_type: CrimeType,
    #[serde(rename = "assignedJudge")]
    pub assigned_judge: String,
    pub location: String,
}

/// Response model for a criminal case
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaseResponse {
    pub id: Uuid,
    pub case_number: String,
    pub title: String,
    pub description: String,
    pub crime_type: CrimeType,
    pub status: CaseStatus,
    pub priority: CasePriority,
    pub assigned_judge: String,
    pub location: String,
    pub opened_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub defendants: Vec<String>,
    pub evidence: Vec<String>,
    pub notes_count: usize,
}

impl From<CriminalCase> for CaseResponse {
    fn from(case: CriminalCase) -> Self {
        Self {
            id: case.id,
            case_number: case.case_number,
            title: case.title,
            description: case.description,
            crime_type: case.crime_type,
            status: case.status,
            priority: case.priority,
            assigned_judge: case.assigned_judge,
            location: case.location,
            opened_at: case.opened_at.to_rfc3339(),
            updated_at: case.updated_at.to_rfc3339(),
            closed_at: case.closed_at.map(|dt| dt.to_rfc3339()),
            defendants: case.defendants,
            evidence: case.evidence,
            notes_count: case.notes.len(),
        }
    }
}

/// Create a new criminal case
#[utoipa::path(
    post,
    path = "/api/cases",
    tags = ["cases"],
    description = "Create a new criminal case",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = CreateCaseRequest,
        description = "Case details",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Case created successfully", body = CaseResponse),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    )
)]
pub fn create_case(req: Request, _p: Params) -> ApiResult<impl IntoResponse> {
    let request: CreateCaseRequest = serde_json::from_slice(req.body())?;

    // Validate input
    if request.title.trim().is_empty() {
        return Err(ApiError::BadRequest("Case title cannot be empty".to_string()));
    }

    // Create domain object
    let case = CriminalCase::new(
        request.title,
        request.description,
        request.crime_type,
        request.assigned_judge,
        request.location,
    );

    // Use repository to persist
    let repository = match RepositoryFactory::case_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    repository.save(&case)?;

    Ok(ResponseBuilder::new(201)
        .header("location", format!("/api/cases/{}", case.id))
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Get a case by ID
#[utoipa::path(
    get,
    path = "/api/cases/{id}",
    tags = ["cases"],
    description = "Retrieve a criminal case by ID",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    responses(
        (status = 200, description = "Case found", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid UUID")
    )
)]
pub fn get_case_by_id(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id_str)?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    match repository.find_by_id(id)? {
        Some(case) => Ok(ResponseBuilder::new(200)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&CaseResponse::from(case))?)
            .build()),
        None => Err(ApiError::NotFound(format!("Case with id {} not found", id))),
    }
}

/// Search cases with filters
#[utoipa::path(
    get,
    path = "/api/cases",
    tags = ["cases"],
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("judge" = Option<String>, Query, description = "Filter by assigned judge"),
        ("active" = Option<bool>, Query, description = "Filter by active status"),
        ("page" = Option<usize>, Query, description = "Page number", minimum = 1),
        ("limit" = Option<usize>, Query, description = "Items per page", minimum = 1, maximum = 100)
    ),
    description = "Search criminal cases with filters and pagination",
    responses(
        (status = 200, description = "List of cases", body = CaseSearchResponse),
        (status = 400, description = "Invalid query parameters")
    )
)]
pub fn search_cases(req: Request, _p: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let params = parse_case_query(query_string)?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let (cases, total) = repository.search(params)?;

    let response = CaseSearchResponse {
        cases: cases.into_iter().map(CaseResponse::from).collect(),
        total,
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Response for case search
#[derive(Debug, Serialize, ToSchema)]
pub struct CaseSearchResponse {
    pub cases: Vec<CaseResponse>,
    pub total: usize,
}

/// Get case statistics
#[utoipa::path(
    get,
    path = "/api/cases/statistics",
    tags = ["cases"],
    description = "Get criminal case statistics",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    responses(
        (status = 200, description = "Case statistics", body = crate::ports::case_repository::CaseStatistics)
    )
)]
pub fn get_case_statistics(req: Request, _p: Params) -> ApiResult<impl IntoResponse> {
    let repository = match RepositoryFactory::case_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };
    let stats = repository.get_statistics()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&stats)?)
        .build())
}

/// Update case status
#[utoipa::path(
    patch,
    path = "/api/cases/{id}/status",
    tags = ["cases"],
    description = "Update the status of a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = UpdateStatusRequest,
        description = "New status",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Status updated", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid status")
    )
)]
pub fn update_case_status(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id_str)?;
    let update_req: UpdateStatusRequest = serde_json::from_slice(req.body())?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.update_status(update_req.status);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Request to update case status
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStatusRequest {
    pub status: CaseStatus,
}

/// Request to update case priority
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePriorityRequest {
    pub priority: CasePriority,
}

/// Request to add a defendant
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({ "name": "John Doe" }))]
pub struct AddDefendantRequest {
    pub name: String,
}

/// Request to add evidence
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({ "description": "Fingerprints found at scene" }))]
pub struct AddEvidenceRequest {
    pub description: String,
}

/// Request to add a note
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({ "content": "Witness interviewed", "author": "Detective Smith" }))]
pub struct AddNoteRequest {
    pub content: String,
    pub author: String,
}

/// Get case by case number
#[utoipa::path(
    get,
    path = "/api/cases/by-number/{case_number}",
    tags = ["cases"],
    description = "Retrieve a criminal case by its case number",
    params(
        ("case_number" = String, Path, description = "Case number (e.g., 2024-123456)"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    responses(
        (status = 200, description = "Case found", body = CaseResponse),
        (status = 404, description = "Case not found")
    )
)]
pub fn get_case_by_number(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let case_number = p.get("case_number")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'case_number'".to_string()))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    match repository.find_by_case_number(case_number)? {
        Some(case) => Ok(ResponseBuilder::new(200)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&CaseResponse::from(case))?)
            .build()),
        None => Err(ApiError::NotFound(format!("Case with number {} not found", case_number))),
    }
}

/// Update case priority
#[utoipa::path(
    patch,
    path = "/api/cases/{id}/priority",
    tags = ["cases"],
    description = "Update the priority of a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = UpdatePriorityRequest,
        description = "New priority",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Priority updated", body = CaseResponse),
        (status = 404, description = "Case not found")
    )
)]
pub fn update_case_priority(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id_str)?;
    let update_req: UpdatePriorityRequest = serde_json::from_slice(req.body())?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.update_priority(update_req.priority);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Add defendant to case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/defendants",
    tags = ["cases"],
    description = "Add a defendant to a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = AddDefendantRequest,
        description = "Defendant information",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Defendant added", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub fn add_defendant(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id_str)?;
    let add_req: AddDefendantRequest = serde_json::from_slice(req.body())?;

    if add_req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("Defendant name cannot be empty".to_string()));
    }

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.add_defendant(add_req.name);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Add evidence to case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/evidence",
    tags = ["cases"],
    description = "Add evidence to a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = AddEvidenceRequest,
        description = "Evidence description",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Evidence added", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub fn add_evidence(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id_str)?;
    let add_req: AddEvidenceRequest = serde_json::from_slice(req.body())?;

    if add_req.description.trim().is_empty() {
        return Err(ApiError::BadRequest("Evidence description cannot be empty".to_string()));
    }

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.add_evidence(add_req.description);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Add note to case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/notes",
    tags = ["cases"],
    description = "Add a note to a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = AddNoteRequest,
        description = "Note content",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Note added", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub fn add_note(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id_str)?;
    let add_req: AddNoteRequest = serde_json::from_slice(req.body())?;

    if add_req.content.trim().is_empty() {
        return Err(ApiError::BadRequest("Note content cannot be empty".to_string()));
    }

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.add_note(add_req.content, add_req.author);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Delete a case
#[utoipa::path(
    delete,
    path = "/api/cases/{id}",
    tags = ["cases"],
    description = "Delete a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    responses(
        (status = 204, description = "Case deleted"),
        (status = 404, description = "Case not found")
    )
)]
pub fn delete_case(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id_str)?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };

    if repository.delete(id)? {
        Ok(ResponseBuilder::new(204).build())
    } else {
        Err(ApiError::NotFound(format!("Case with id {} not found", id)))
    }
}

/// Get cases by assigned judge
#[utoipa::path(
    get,
    path = "/api/cases/by-judge/{judge}",
    tags = ["cases"],
    description = "Get all cases assigned to a specific judge",
    params(
        ("judge" = String, Path, description = "Judge name"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    responses(
        (status = 200, description = "List of cases", body = CaseSearchResponse)
    )
)]
pub fn get_cases_by_judge(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let judge = p.get("judge")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'judge'".to_string()))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let cases = repository.find_by_judge(judge)?;

    let response = CaseSearchResponse {
        cases: cases.into_iter().map(CaseResponse::from).collect(),
        total: 0, // Will be set to actual count
    };

    // Update total
    let mut response = response;
    response.total = response.cases.len();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get case count by status
#[utoipa::path(
    get,
    path = "/api/cases/count-by-status/{status}",
    tags = ["cases"],
    description = "Get count of cases by status",
    params(
        ("status" = String, Path, description = "Case status"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    responses(
        (status = 200, description = "Count of cases", body = CountResponse)
    )
)]
pub fn count_by_status(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let status_str = p.get("status")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'status'".to_string()))?;

    let status: CaseStatus = serde_json::from_str(&format!("\"{}\"", status_str))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {


        Ok(r) => r,


        Err(e) => return Err(e),


    };
    let count = repository.count_by_status(status)?;

    let response = CountResponse { count };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Response for count queries
#[derive(Debug, Serialize, ToSchema)]
pub struct CountResponse {
    pub count: usize,
}

/// Request to enter a plea
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({ "defendantName": "John Doe", "plea": "not_guilty" }))]
pub struct EnterPleaRequest {
    #[serde(rename = "defendantName")]
    pub defendant_name: String,
    pub plea: PleaType,
}

/// Request to schedule a court event
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "eventType": "arraignment",
    "scheduledDate": "2024-02-15T10:00:00Z",
    "description": "Initial arraignment",
    "location": "Courtroom 3A"
}))]
pub struct ScheduleEventRequest {
    #[serde(rename = "eventType")]
    pub event_type: EventType,
    #[serde(rename = "scheduledDate")]
    pub scheduled_date: DateTime<Utc>,
    pub description: String,
    pub location: String,
}

/// Request to file a motion
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "motionType": "suppress_evidence",
    "filedBy": "Defense Counsel",
    "description": "Motion to suppress illegally obtained evidence"
}))]
pub struct FileMotionRequest {
    #[serde(rename = "motionType")]
    pub motion_type: MotionType,
    #[serde(rename = "filedBy")]
    pub filed_by: String,
    pub description: String,
}

/// Request to rule on a motion
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "motionId": "550e8400-e29b-41d4-a716-446655440000",
    "ruling": "granted"
}))]
pub struct RuleOnMotionRequest {
    #[serde(rename = "motionId")]
    pub motion_id: Uuid,
    pub ruling: String,
}


/// Enter plea for defendant
#[utoipa::path(
    post,
    path = "/api/cases/{id}/plea",
    tags = ["cases"],
    description = "Enter a plea for a defendant in a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = EnterPleaRequest,
        description = "Plea information",
    ),
    responses(
        (status = 200, description = "Plea entered", body = CaseResponse),
        (status = 404, description = "Case not found")
    )
)]
pub fn enter_plea(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let plea_req: EnterPleaRequest = serde_json::from_slice(req.body())?;
    let repository = match RepositoryFactory::case_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case {} not found", id)))?;

    case.enter_plea(plea_req.defendant_name, plea_req.plea);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Schedule court event
#[utoipa::path(
    post,
    path = "/api/cases/{id}/events",
    tags = ["cases"],
    description = "Schedule a court event for a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = ScheduleEventRequest,
        description = "Event information",
    ),
    responses(
        (status = 200, description = "Event scheduled", body = CaseResponse),
        (status = 404, description = "Case not found")
    )
)]
pub fn schedule_court_event(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let event_req: ScheduleEventRequest = serde_json::from_slice(req.body())?;
    let repository = match RepositoryFactory::case_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case {} not found", id)))?;

    case.schedule_event(event_req.event_type, event_req.scheduled_date, event_req.description, event_req.location);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// File a motion
#[utoipa::path(
    post,
    path = "/api/cases/{id}/motions",
    tags = ["cases"],
    description = "File a motion in a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = FileMotionRequest,
        description = "Motion information",
    ),
    responses(
        (status = 200, description = "Motion filed", body = CaseResponse),
        (status = 404, description = "Case not found")
    )
)]
pub fn file_motion(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let motion_req: FileMotionRequest = serde_json::from_slice(req.body())?;
    let repository = match RepositoryFactory::case_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case {} not found", id)))?;

    case.file_motion(motion_req.motion_type, motion_req.filed_by, motion_req.description);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Rule on a motion
#[utoipa::path(
    patch,
    path = "/api/cases/{id}/motions/ruling",
    tags = ["cases"],
    description = "Rule on a motion in a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = RuleOnMotionRequest,
        description = "Motion ruling",
    ),
    responses(
        (status = 200, description = "Motion ruled on", body = CaseResponse),
        (status = 404, description = "Case not found")
    )
)]
pub fn rule_on_motion(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let ruling_req: RuleOnMotionRequest = serde_json::from_slice(req.body())?;
    let repository = match RepositoryFactory::case_repo_validated(&req) {

        Ok(r) => r,

        Err(e) => return Err(e),

    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case {} not found", id)))?;

    case.rule_on_motion(ruling_req.motion_id, ruling_req.ruling);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Parse query parameters for case search
fn parse_case_query(query: &str) -> ApiResult<CaseQuery> {
    let mut params = CaseQuery {
        offset: 0,
        limit: 20,
        ..Default::default()
    };

    if query.is_empty() {
        return Ok(params);
    }

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() != 2 || parts[1].is_empty() {
            continue;
        }

        match parts[0] {
            "status" => {
                params.status = Some(serde_json::from_str(&format!("\"{}\"", parts[1]))?);
            }
            "priority" => {
                params.priority = Some(serde_json::from_str(&format!("\"{}\"", parts[1]))?);
            }
            "judge" => {
                params.judge = Some(parts[1].to_string());
            }
            "active" => {
                params.is_active = Some(parts[1].parse()
                    .map_err(|_| ApiError::BadRequest("Invalid active value".to_string()))?);
            }
            "page" => {
                let page: usize = parts[1].parse()
                    .map_err(|_| ApiError::BadRequest("Invalid page number".to_string()))?;
                params.offset = (page - 1) * params.limit;
            }
            "limit" => {
                params.limit = parts[1].parse()
                    .map_err(|_| ApiError::BadRequest("Invalid limit value".to_string()))?;
            }
            _ => {}
        }
    }

    Ok(params)
}