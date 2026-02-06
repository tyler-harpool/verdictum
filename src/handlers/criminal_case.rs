//! Criminal case management API handlers
//!
//! This module demonstrates hexagonal architecture by using repository pattern
//! for all data access, keeping the handlers decoupled from storage implementation.

use crate::adapters::rules_engine_impl::SpinRulesEngine;
use crate::domain::criminal_case::{CaseStatus, CasePriority, CrimeType, CriminalCase, EventType, MotionType, EvidenceType, EvidenceCondition};
use crate::domain::common::MotionStatus;
use crate::domain::defendant::{CreateDefendantRequest, PleaType, AddCountRequest};
use crate::domain::docket::{DocketEntryType, DelayReason};
use crate::domain::filing_pipeline::{ComplianceReport, FilingContext};
use crate::domain::rule::TriggerEvent;
use crate::domain::victim::{CreateVictimRequest, SendNotificationRequest, VictimType, NotificationMethod, NotificationType};
use crate::error::{ApiError, ApiResult};
use crate::ports::case_repository::{CaseRepository, CaseQuery, CaseQueryRepository};
use crate::ports::rules_engine::RulesEngine;
use crate::ports::rules_repository::RulesRepository;
use crate::utils::repository_factory::RepositoryFactory;
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
    "districtCode": "SDNY",
    "assignedJudgeId": "550e8400-e29b-41d4-a716-446655440000",
    "judgeInitials": "JMS",
    "location": "New York, NY"
}))]
pub struct CreateCaseRequest {
    pub title: String,
    pub description: String,
    #[serde(rename = "crimeType")]
    pub crime_type: CrimeType,
    #[serde(rename = "districtCode")]
    pub district_code: String,
    #[serde(rename = "assignedJudgeId")]
    pub assigned_judge_id: Option<Uuid>,
    #[serde(rename = "judgeInitials", default = "default_judge_initials")]
    pub judge_initials: String,
    pub location: String,
}

fn default_judge_initials() -> String {
    "UNK".to_string()
}

/// Response model for a defendant in a case
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DefendantResponse {
    pub id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub usm_number: Option<String>,
    pub fbi_number: Option<String>,
    pub custody_status: String,
    pub counts_count: usize,
}

/// Response model for a docket entry
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocketEntryResponse {
    pub id: Uuid,
    pub entry_number: u32,
    pub date_filed: String,
    pub entry_type: DocketEntryType,
    pub description: String,
    pub filed_by: Option<String>,
    pub is_sealed: bool,
}

/// Response model for evidence
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceResponse {
    pub id: Uuid,
    pub description: String,
    pub evidence_type: EvidenceType,
    pub is_sealed: bool,
    pub custody_transfers_count: usize,
    pub created_at: String,
}

/// Response model for speedy trial status
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpeedyTrialStatusResponse {
    pub days_elapsed: i64,
    pub days_remaining: i64,
    pub is_tolled: bool,
    pub waived: bool,
    pub trial_start_deadline: String,
    pub excludable_delays_count: usize,
}

/// Response model for a victim
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VictimResponse {
    pub id: Uuid,
    pub name: String,
    pub victim_type: VictimType,
    pub notifications_count: usize,
    pub opted_out: bool,
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
    pub assigned_judge_id: Option<Uuid>,
    pub district_code: String,
    pub location: String,
    pub opened_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub defendants: Vec<DefendantResponse>,
    pub evidence: Vec<EvidenceResponse>,
    pub evidence_count: usize,
    pub notes_count: usize,
    pub docket_entries_count: usize,
    pub is_sealed: bool,
    pub sealed_date: Option<String>,
    pub sealed_by: Option<String>,
    pub seal_reason: Option<String>,
    pub speedy_trial_status: Option<SpeedyTrialStatusResponse>,
    pub victims_count: usize,
}

impl From<CriminalCase> for CaseResponse {
    fn from(case: CriminalCase) -> Self {
        let defendants = case.defendants.iter().map(|d| DefendantResponse {
            id: d.id,
            name: d.name.clone(),
            aliases: d.aliases.clone(),
            usm_number: d.usm_number.clone(),
            fbi_number: d.fbi_number.clone(),
            custody_status: format!("{:?}", d.custody_status),
            counts_count: d.counts.len(),
        }).collect();

        let speedy_trial_status = case.speedy_trial.as_ref().map(|st| SpeedyTrialStatusResponse {
            days_elapsed: st.days_elapsed,
            days_remaining: st.days_remaining,
            is_tolled: st.is_tolled,
            waived: st.waived,
            trial_start_deadline: st.trial_start_deadline.to_rfc3339(),
            excludable_delays_count: st.excludable_delays.len(),
        });

        Self {
            id: case.id,
            case_number: case.case_number,
            title: case.title,
            description: case.description,
            crime_type: case.crime_type,
            status: case.status,
            priority: case.priority,
            assigned_judge_id: case.assigned_judge_id,
            district_code: case.district_code,
            location: case.location,
            opened_at: case.opened_at.to_rfc3339(),
            updated_at: case.updated_at.to_rfc3339(),
            closed_at: case.closed_at.map(|dt| dt.to_rfc3339()),
            defendants,
            evidence: case.evidence.iter().map(|e| EvidenceResponse {
                id: e.id,
                description: e.description.clone(),
                evidence_type: e.evidence_type.clone(),
                is_sealed: e.is_sealed,
                custody_transfers_count: e.chain_of_custody.len(),
                created_at: e.created_at.to_rfc3339(),
            }).collect(),
            evidence_count: case.evidence.len(),
            notes_count: case.notes.len(),
            docket_entries_count: case.docket_entries.len(),
            is_sealed: case.is_sealed,
            sealed_date: case.sealed_date.map(|dt| dt.to_rfc3339()),
            sealed_by: case.sealed_by,
            seal_reason: case.seal_reason,
            speedy_trial_status,
            victims_count: case.victims.len(),
        }
    }
}

/// Response for case creation with optional compliance check
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCaseWithComplianceResponse {
    /// The created case
    #[serde(flatten)]
    pub case: CaseResponse,
    /// Compliance report, present only when ?compliance=true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compliance_report: Option<ComplianceReport>,
}

/// Create a new criminal case
///
/// Optionally runs a compliance check when the query parameter `compliance=true`
/// is provided. The compliance check validates the case filing against configured
/// court rules but does not block case creation.
#[utoipa::path(
    post,
    path = "/api/cases",
    tags = ["cases"],
    description = "Create a new criminal case. Pass ?compliance=true to include a compliance report.",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("compliance" = Option<bool>, Query, description = "Run compliance check on case creation")
    ),
    request_body(
        content = CreateCaseRequest,
        description = "Case details",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Case created successfully", body = CreateCaseWithComplianceResponse),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    )
)]
pub fn create_case(req: Request, _p: Params) -> ApiResult<impl IntoResponse> {
    let run_compliance = parse_compliance_flag(req.query());

    let request: CreateCaseRequest = serde_json::from_slice(req.body())?;

    if request.title.trim().is_empty() {
        return Err(ApiError::BadRequest("Case title cannot be empty".to_string()));
    }

    if request.district_code.trim().is_empty() {
        return Err(ApiError::BadRequest("District code is required".to_string()));
    }

    let case = CriminalCase::new(
        request.title,
        request.description,
        request.crime_type,
        request.district_code,
        request.assigned_judge_id,
        &request.judge_initials,
        request.location,
    );

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    repository.save(&case)?;

    // Optional compliance check: evaluate rules without blocking creation
    let compliance_report = if run_compliance {
        run_case_compliance_check(&req, &case).ok()
    } else {
        None
    };

    let response = CreateCaseWithComplianceResponse {
        case: CaseResponse::from(case),
        compliance_report,
    };

    Ok(ResponseBuilder::new(201)
        .header("location", format!("/api/cases/{}", response.case.id))
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Parse the ?compliance=true query parameter
fn parse_compliance_flag(query: &str) -> bool {
    for pair in query.split('&') {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 && parts[0] == "compliance" {
            return parts[1].eq_ignore_ascii_case("true");
        }
    }
    false
}

/// Run compliance check against configured rules for a newly created case.
/// Returns a ComplianceReport or an error. Errors are non-fatal and logged.
fn run_case_compliance_check(req: &Request, case: &CriminalCase) -> Result<ComplianceReport, ApiError> {
    let rules_repo = RepositoryFactory::rules_repo_validated(req)?;

    let context = FilingContext {
        case_type: format!("{:?}", case.crime_type).to_lowercase(),
        document_type: "case_initiation".to_string(),
        filer_role: "clerk".to_string(),
        jurisdiction_id: case.district_code.to_lowercase(),
        division: None,
        assigned_judge: case.assigned_judge_id.map(|id| id.to_string()),
        service_method: None,
        metadata: serde_json::json!({
            "case_number": case.case_number,
            "case_type": "criminal"
        }),
    };

    let all_rules = rules_repo.find_all_rules()
        .map_err(|e| ApiError::Internal(format!("Failed to load rules: {}", e)))?;

    let rules_engine = SpinRulesEngine::new();
    let applicable = rules_engine.select_rules(
        &context.jurisdiction_id,
        &TriggerEvent::CaseFiled,
        &all_rules,
    );
    let sorted = rules_engine.resolve_priority(applicable);
    let compliance_report = rules_engine.evaluate(&context, &sorted)?;

    Ok(compliance_report)
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
        ("judge" = Option<String>, Query, description = "Filter by assigned judge ID"),
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

/// Request to add a defendant (uses domain CreateDefendantRequest)
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "John Doe",
    "aliases": ["JD", "Johnny D"],
    "usmNumber": "12345-678",
    "custodyStatus": "in_custody"
}))]
pub struct AddDefendantRequest {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(rename = "usmNumber")]
    pub usm_number: Option<String>,
    #[serde(rename = "fbiNumber")]
    pub fbi_number: Option<String>,
}

/// Request to add evidence
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({ "description": "Fingerprints found at scene", "evidenceType": "physical" }))]
pub struct AddEvidenceRequest {
    pub description: String,
    #[serde(rename = "evidenceType")]
    pub evidence_type: Option<EvidenceType>,
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
        ("case_number" = String, Path, description = "Case number (e.g., SDNY:26-CR-00123-JMS)"),
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

    let domain_request = CreateDefendantRequest {
        name: add_req.name,
        aliases: add_req.aliases,
        usm_number: add_req.usm_number,
        fbi_number: add_req.fbi_number,
        date_of_birth: None,
        citizenship_status: None,
        custody_status: None,
        bond_info: None,
    };

    case.add_defendant(domain_request);
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

    case.add_evidence(add_req.description, add_req.evidence_type);
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
    path = "/api/cases/by-judge/{judge_id}",
    tags = ["cases"],
    description = "Get all cases assigned to a specific judge by judge ID",
    params(
        ("judge_id" = Uuid, Path, description = "Judge ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    responses(
        (status = 200, description = "List of cases", body = CaseSearchResponse)
    )
)]
pub fn get_cases_by_judge(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let judge_id_str = p.get("judge_id")
        .or_else(|| p.get("judge"))
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'judge_id'".to_string()))?;

    let judge_id = Uuid::parse_str(judge_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid judge ID format".to_string()))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let cases = repository.find_by_judge(judge_id)?;

    let total = cases.len();
    let response = CaseSearchResponse {
        cases: cases.into_iter().map(CaseResponse::from).collect(),
        total,
    };

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

/// Request to enter a plea for a defendant's specific count
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "defendantId": "550e8400-e29b-41d4-a716-446655440000",
    "countNumber": 1,
    "plea": "not_guilty"
}))]
pub struct EnterPleaRequest {
    #[serde(rename = "defendantId")]
    pub defendant_id: Uuid,
    #[serde(rename = "countNumber")]
    pub count_number: u32,
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
    pub ruling: MotionStatus,
}

/// Request to add a count/charge to a defendant
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "countNumber": 1,
    "statute": "18 U.S.C. 1343",
    "offenseDescription": "Wire fraud",
    "statutoryMaxMonths": 240,
    "statutoryMinMonths": null
}))]
pub struct AddChargeRequest {
    #[serde(rename = "countNumber")]
    pub count_number: u32,
    pub statute: String,
    #[serde(rename = "offenseDescription")]
    pub offense_description: String,
    #[serde(rename = "statutoryMaxMonths")]
    pub statutory_max_months: Option<u32>,
    #[serde(rename = "statutoryMinMonths")]
    pub statutory_min_months: Option<u32>,
}


/// Enter plea for defendant's count
#[utoipa::path(
    post,
    path = "/api/cases/{id}/plea",
    tags = ["cases"],
    description = "Enter a plea for a defendant's count in a criminal case",
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
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request")
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

    case.enter_plea(plea_req.defendant_id, plea_req.count_number, plea_req.plea)
        .map_err(|e| ApiError::BadRequest(e))?;
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Add a charge/count to a defendant
#[utoipa::path(
    post,
    path = "/api/cases/{id}/defendants/{defendant_id}/charges",
    tags = ["cases"],
    description = "Add a criminal charge/count to a defendant in a case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("defendant_id" = Uuid, Path, description = "Defendant ID"),
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
    request_body(
        content = AddChargeRequest,
        description = "Charge/count details",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Charge added", body = CaseResponse),
        (status = 404, description = "Case or defendant not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub fn add_charge(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let defendant_id_str = p.get("defendant_id")
        .ok_or_else(|| ApiError::BadRequest("Missing defendant ID".to_string()))?;
    let defendant_id = Uuid::parse_str(defendant_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid defendant ID format".to_string()))?;

    let charge_req: AddChargeRequest = serde_json::from_slice(req.body())?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case {} not found", id)))?;

    let defendant = case.defendants.iter_mut().find(|d| d.id == defendant_id)
        .ok_or_else(|| ApiError::NotFound(format!("Defendant {} not found in case {}", defendant_id, id)))?;

    defendant.add_count(AddCountRequest {
        count_number: charge_req.count_number,
        statute: charge_req.statute,
        offense_description: charge_req.offense_description,
        statutory_max_months: charge_req.statutory_max_months,
        statutory_min_months: charge_req.statutory_min_months,
    });

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
                params.judge_id = Some(Uuid::parse_str(parts[1])
                    .map_err(|_| ApiError::BadRequest("Invalid judge ID format".to_string()))?);
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

// ============================================================================
// Phase 2: Evidence Chain of Custody Handlers
// ============================================================================

/// Request to add a custody transfer
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "transferredFrom": "FBI Evidence Room",
    "transferredTo": "Court Clerk",
    "location": "Federal Courthouse",
    "condition": "good",
    "notes": "Evidence sealed and intact"
}))]
pub struct AddCustodyTransferRequest {
    #[serde(rename = "transferredFrom")]
    pub transferred_from: String,
    #[serde(rename = "transferredTo")]
    pub transferred_to: String,
    pub location: String,
    pub condition: EvidenceCondition,
    pub notes: Option<String>,
}

/// Add a custody transfer to an evidence item
#[utoipa::path(
    post,
    path = "/api/cases/{id}/evidence/{evidence_id}/custody",
    tags = ["cases"],
    description = "Add a custody transfer to an evidence item",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("evidence_id" = Uuid, Path, description = "Evidence ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = AddCustodyTransferRequest, description = "Custody transfer details"),
    responses(
        (status = 200, description = "Transfer added", body = CaseResponse),
        (status = 404, description = "Case or evidence not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub fn add_custody_transfer(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let evidence_id_str = p.get("evidence_id")
        .ok_or_else(|| ApiError::BadRequest("Missing evidence ID".to_string()))?;
    let evidence_id = Uuid::parse_str(evidence_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid evidence ID format".to_string()))?;

    let transfer_req: AddCustodyTransferRequest = serde_json::from_slice(req.body())?;

    if transfer_req.transferred_from.trim().is_empty() {
        return Err(ApiError::BadRequest("Transferred from cannot be empty".to_string()));
    }
    if transfer_req.transferred_to.trim().is_empty() {
        return Err(ApiError::BadRequest("Transferred to cannot be empty".to_string()));
    }

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.add_custody_transfer(
        evidence_id,
        transfer_req.transferred_from,
        transfer_req.transferred_to,
        transfer_req.location,
        transfer_req.condition,
        transfer_req.notes,
    ).map_err(|e| ApiError::NotFound(e))?;

    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

// ============================================================================
// Phase 1: Docket Entry Handlers
// ============================================================================

/// Request to add a docket entry
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "entryType": "motion",
    "description": "Motion to Suppress Evidence",
    "filedBy": "Defense Counsel"
}))]
pub struct AddDocketEntryRequest {
    #[serde(rename = "entryType")]
    pub entry_type: DocketEntryType,
    pub description: String,
    #[serde(rename = "filedBy")]
    pub filed_by: Option<String>,
}

/// Add a docket entry to a case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/docket",
    tags = ["cases"],
    description = "Add a docket entry to a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = AddDocketEntryRequest, description = "Docket entry details"),
    responses(
        (status = 200, description = "Docket entry added", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub fn add_docket_entry(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let add_req: AddDocketEntryRequest = serde_json::from_slice(req.body())?;

    if add_req.description.trim().is_empty() {
        return Err(ApiError::BadRequest("Docket entry description cannot be empty".to_string()));
    }

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.add_docket_entry(add_req.entry_type, add_req.description, add_req.filed_by);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Get docket entries for a case
#[utoipa::path(
    get,
    path = "/api/cases/{id}/docket",
    tags = ["cases"],
    description = "Get all docket entries for a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    responses(
        (status = 200, description = "List of docket entries", body = Vec<DocketEntryResponse>),
        (status = 404, description = "Case not found")
    )
)]
pub fn get_docket_entries(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    let entries: Vec<DocketEntryResponse> = case.get_docket_entries().iter().map(|e| DocketEntryResponse {
        id: e.id,
        entry_number: e.entry_number,
        date_filed: e.date_filed.to_rfc3339(),
        entry_type: e.entry_type.clone(),
        description: e.description.clone(),
        filed_by: e.filed_by.clone(),
        is_sealed: e.is_sealed,
    }).collect();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&entries)?)
        .build())
}

// ============================================================================
// Phase 3: Sealed Case Handlers
// ============================================================================

/// Request to seal a case
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "reason": "Ongoing investigation - witness safety",
    "sealedBy": "Judge Smith"
}))]
pub struct SealCaseRequest {
    pub reason: String,
    #[serde(rename = "sealedBy")]
    pub sealed_by: String,
}

/// Request to unseal a case
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "reason": "Investigation complete",
    "unsealedBy": "Judge Smith",
    "courtOrderId": "ORD-2026-001"
}))]
pub struct UnsealCaseRequest {
    pub reason: String,
    #[serde(rename = "unsealedBy")]
    pub unsealed_by: String,
    #[serde(rename = "courtOrderId")]
    pub court_order_id: Option<String>,
}

/// Seal a criminal case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/seal",
    tags = ["cases"],
    description = "Seal a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = SealCaseRequest, description = "Seal request details"),
    responses(
        (status = 200, description = "Case sealed", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request or already sealed")
    )
)]
pub fn seal_case(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let seal_req: SealCaseRequest = serde_json::from_slice(req.body())?;

    if seal_req.reason.trim().is_empty() {
        return Err(ApiError::BadRequest("Seal reason cannot be empty".to_string()));
    }
    if seal_req.sealed_by.trim().is_empty() {
        return Err(ApiError::BadRequest("Sealed by cannot be empty".to_string()));
    }

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.seal(seal_req.reason, seal_req.sealed_by)
        .map_err(|e| ApiError::BadRequest(e))?;
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Unseal a criminal case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/unseal",
    tags = ["cases"],
    description = "Unseal a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = UnsealCaseRequest, description = "Unseal request details"),
    responses(
        (status = 200, description = "Case unsealed", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request or not sealed")
    )
)]
pub fn unseal_case(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let unseal_req: UnsealCaseRequest = serde_json::from_slice(req.body())?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.unseal(unseal_req.reason, unseal_req.unsealed_by, unseal_req.court_order_id)
        .map_err(|e| ApiError::BadRequest(e))?;
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

// ============================================================================
// Phase 4: Speedy Trial Clock Handlers
// ============================================================================

/// Request to start the speedy trial clock
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "arrestDate": "2026-01-15T00:00:00Z",
    "indictmentDate": "2026-01-20T00:00:00Z",
    "arraignmentDate": "2026-01-25T00:00:00Z"
}))]
pub struct StartSpeedyTrialRequest {
    #[serde(rename = "arrestDate")]
    pub arrest_date: Option<String>,
    #[serde(rename = "indictmentDate")]
    pub indictment_date: Option<String>,
    #[serde(rename = "arraignmentDate")]
    pub arraignment_date: Option<String>,
}

/// Request to add an excludable delay to speedy trial clock
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "startDate": "2026-02-01T00:00:00Z",
    "endDate": "2026-02-15T00:00:00Z",
    "reason": "pretrial_motions",
    "statutoryReference": "18 U.S.C. § 3161(h)(1)(D)",
    "daysExcluded": 14,
    "orderReference": "ORD-2026-001"
}))]
pub struct AddCaseExcludableDelayRequest {
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    pub reason: DelayReason,
    #[serde(rename = "statutoryReference")]
    pub statutory_reference: String,
    #[serde(rename = "daysExcluded")]
    pub days_excluded: i64,
    #[serde(rename = "orderReference")]
    pub order_reference: Option<String>,
}

/// Start the speedy trial clock for a case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/speedy-trial/start",
    tags = ["cases"],
    description = "Start the speedy trial clock for a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = StartSpeedyTrialRequest, description = "Speedy trial start details"),
    responses(
        (status = 200, description = "Clock started", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Clock already initialized")
    )
)]
pub fn start_speedy_trial(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let st_req: StartSpeedyTrialRequest = serde_json::from_slice(req.body())?;

    let arrest_date = st_req.arrest_date
        .map(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
        .transpose()
        .map_err(|_| ApiError::BadRequest("Invalid arrest date format".to_string()))?;
    let indictment_date = st_req.indictment_date
        .map(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
        .transpose()
        .map_err(|_| ApiError::BadRequest("Invalid indictment date format".to_string()))?;
    let arraignment_date = st_req.arraignment_date
        .map(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
        .transpose()
        .map_err(|_| ApiError::BadRequest("Invalid arraignment date format".to_string()))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.start_speedy_trial(arrest_date, indictment_date, arraignment_date)
        .map_err(|e| ApiError::BadRequest(e))?;
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Add an excludable delay to the speedy trial clock
#[utoipa::path(
    post,
    path = "/api/cases/{id}/speedy-trial/exclude",
    tags = ["cases"],
    description = "Add an excludable delay to the speedy trial clock",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = AddCaseExcludableDelayRequest, description = "Excludable delay details"),
    responses(
        (status = 200, description = "Delay added", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Clock not initialized")
    )
)]
pub fn add_case_excludable_delay(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let delay_req: AddCaseExcludableDelayRequest = serde_json::from_slice(req.body())?;

    let start_date = DateTime::parse_from_rfc3339(&delay_req.start_date)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| ApiError::BadRequest("Invalid start date format".to_string()))?;
    let end_date = delay_req.end_date
        .map(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
        .transpose()
        .map_err(|_| ApiError::BadRequest("Invalid end date format".to_string()))?;

    let delay = crate::domain::docket::ExcludableDelay {
        start_date,
        end_date,
        reason: delay_req.reason,
        statutory_reference: delay_req.statutory_reference,
        days_excluded: delay_req.days_excluded,
        order_reference: delay_req.order_reference,
    };

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    case.add_excludable_delay(delay)
        .map_err(|e| ApiError::BadRequest(e))?;
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Get speedy trial status for a case
#[utoipa::path(
    get,
    path = "/api/cases/{id}/speedy-trial",
    tags = ["cases"],
    description = "Get speedy trial clock status for a case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    responses(
        (status = 200, description = "Speedy trial status", body = SpeedyTrialStatusResponse),
        (status = 404, description = "Case not found or clock not initialized")
    )
)]
pub fn get_case_speedy_trial(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    let clock = case.speedy_trial
        .ok_or_else(|| ApiError::NotFound("Speedy trial clock not initialized for this case".to_string()))?;

    let status = SpeedyTrialStatusResponse {
        days_elapsed: clock.days_elapsed,
        days_remaining: clock.days_remaining,
        is_tolled: clock.is_tolled,
        waived: clock.waived,
        trial_start_deadline: clock.trial_start_deadline.to_rfc3339(),
        excludable_delays_count: clock.excludable_delays.len(),
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&status)?)
        .build())
}

// ============================================================================
// Phase 5: CVRA Victim Handlers
// ============================================================================

/// Request to add a victim
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "Jane Victim",
    "victimType": "individual",
    "preferredMethod": "email",
    "email": "jane@example.com"
}))]
pub struct AddVictimRequest {
    pub name: String,
    #[serde(rename = "victimType", default = "default_victim_type")]
    pub victim_type: VictimType,
    #[serde(rename = "preferredMethod", default = "default_notification_method")]
    pub preferred_method: NotificationMethod,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(rename = "mailingAddress")]
    pub mailing_address: Option<String>,
    #[serde(rename = "victimAdvocate")]
    pub victim_advocate: Option<String>,
}

fn default_victim_type() -> VictimType {
    VictimType::Individual
}

fn default_notification_method() -> NotificationMethod {
    NotificationMethod::Email
}

/// Request to send a victim notification
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "notificationType": "hearing_scheduled",
    "contentSummary": "Status conference scheduled for Feb 20, 2026"
}))]
pub struct SendVictimNotificationRequest {
    #[serde(rename = "notificationType")]
    pub notification_type: NotificationType,
    #[serde(rename = "contentSummary")]
    pub content_summary: String,
}

/// Add a victim to a case
#[utoipa::path(
    post,
    path = "/api/cases/{id}/victims",
    tags = ["cases"],
    description = "Add a CVRA victim to a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = AddVictimRequest, description = "Victim details"),
    responses(
        (status = 200, description = "Victim added", body = CaseResponse),
        (status = 404, description = "Case not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub fn add_victim(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let victim_req: AddVictimRequest = serde_json::from_slice(req.body())?;

    if victim_req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("Victim name cannot be empty".to_string()));
    }

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    let domain_request = CreateVictimRequest {
        name: victim_req.name,
        victim_type: victim_req.victim_type,
        preferred_method: victim_req.preferred_method,
        email: victim_req.email,
        phone: victim_req.phone,
        mailing_address: victim_req.mailing_address,
        victim_advocate: victim_req.victim_advocate,
    };

    case.add_victim(domain_request);
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}

/// Get victims for a case
#[utoipa::path(
    get,
    path = "/api/cases/{id}/victims",
    tags = ["cases"],
    description = "Get all CVRA victims for a criminal case",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    responses(
        (status = 200, description = "List of victims", body = Vec<VictimResponse>),
        (status = 404, description = "Case not found")
    )
)]
pub fn get_victims(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    let victims: Vec<VictimResponse> = case.victims.iter().map(|v| VictimResponse {
        id: v.id,
        name: v.name.clone(),
        victim_type: v.victim_type.clone(),
        notifications_count: v.notifications.len(),
        opted_out: v.notification_preferences.opt_out,
    }).collect();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&victims)?)
        .build())
}

/// Send a notification to a victim
#[utoipa::path(
    post,
    path = "/api/cases/{id}/victims/{victim_id}/notifications",
    tags = ["cases"],
    description = "Send a notification to a CVRA victim",
    params(
        ("id" = Uuid, Path, description = "Case ID"),
        ("victim_id" = Uuid, Path, description = "Victim ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
    request_body(content = SendVictimNotificationRequest, description = "Notification details"),
    responses(
        (status = 200, description = "Notification sent", body = CaseResponse),
        (status = 404, description = "Case or victim not found"),
        (status = 400, description = "Victim opted out")
    )
)]
pub fn send_victim_notification(req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id_str = p.get("id").ok_or_else(|| ApiError::BadRequest("Missing case ID".to_string()))?;
    let id = Uuid::parse_str(id_str).map_err(|_| ApiError::BadRequest("Invalid case ID format".to_string()))?;

    let victim_id_str = p.get("victim_id")
        .ok_or_else(|| ApiError::BadRequest("Missing victim ID".to_string()))?;
    let victim_id = Uuid::parse_str(victim_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid victim ID format".to_string()))?;

    let notif_req: SendVictimNotificationRequest = serde_json::from_slice(req.body())?;

    let repository = match RepositoryFactory::case_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let mut case = repository.find_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("Case with id {} not found", id)))?;

    let domain_request = SendNotificationRequest {
        notification_type: notif_req.notification_type,
        content_summary: notif_req.content_summary,
    };

    case.send_victim_notification(victim_id, domain_request)
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(e)
            } else {
                ApiError::BadRequest(e)
            }
        })?;
    repository.save(&case)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&CaseResponse::from(case))?)
        .build())
}
