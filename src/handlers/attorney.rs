//! Attorney and Party Management HTTP Handlers
//!
//! Provides comprehensive API endpoints for attorney registration, CJA panel management,
//! pro hac vice admissions, party representation, and conflict checking.

#[allow(unused_imports)] // These types are used in JSON serialization/deserialization
use crate::domain::attorney::{
    Attorney, AttorneyStatus, Party, PartyStatus, AttorneyRepresentation,
    ConflictCheck, ServiceRecord, AttorneyMetrics, BarAdmission, FederalAdmission,
    ProHacViceAdmission, CJAAppointment, ECFRegistration, DisciplinaryAction,
    Address, RepresentationType, WithdrawalReason, ServiceMethod,
    ConflictType, ConflictSeverity, ConflictResult,
    CreateAttorneyRequest, UpdateAttorneyRequest, CreatePartyRequest, PartyType, PartyRole, EntityType
};
use crate::domain::attorney_conflict::{
    ConflictCheckRequest, ConflictCheckResult, ConflictDetails, ConflictRecommendation,
};
use crate::domain::attorney_case::{
    AssignAttorneyRequest, AttorneyCaseAssignment, AttorneyCaseLoad, RemoveAttorneyRequest,
    AttorneyRepresentationHistory,
};
use crate::error::ApiError;
use crate::ports::attorney_repository::AttorneyRepository;
use crate::utils::{json_response as json, query_parser, repository_factory::RepositoryFactory};
use spin_sdk::http::{Params, Request, Response};

// Attorney Management Endpoints

/// Create a new attorney
#[utoipa::path(
    post,
    path = "/api/attorneys",
    request_body = CreateAttorneyRequest,
    responses(
        (status = 200, description = "Attorney created successfully", body = Attorney),
        (status = 400, description = "Invalid attorney data"),
        (status = 409, description = "Attorney with bar number already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_attorney(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let request: CreateAttorneyRequest = match json::parse_body(req.body()) {
        Ok(a) => a,
        Err(e) => return json::error_response(&e),
    };

    // Validate email format
    if !is_valid_email(&request.email) {
        return json::error_response(&ApiError::BadRequest(
            "Invalid email format".to_string()
        ));
    }

    // Check for duplicate bar number
    match repo.find_attorney_by_bar_number(&request.bar_number) {
        Ok(Some(_)) => {
            return json::error_response(&ApiError::Conflict(
                format!("Attorney with bar number {} already exists", request.bar_number)
            ));
        }
        Ok(None) => {}, // No duplicate, continue
        Err(e) => return json::error_response(&ApiError::StorageError(e.to_string())),
    }

    // Use the constructor to properly initialize the attorney
    let mut attorney = Attorney::new(
        request.bar_number,
        request.first_name,
        request.last_name,
        request.email,
        request.phone,
        request.address,
    );

    // Set optional fields
    attorney.middle_name = request.middle_name;
    attorney.firm_name = request.firm_name;
    attorney.fax = request.fax;

    match repo.save_attorney(attorney) {
        Ok(saved) => json::success_response(&saved),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Simple email validation
fn is_valid_email(email: &str) -> bool {
    // Basic validation: must contain @ and at least one . after @
    if let Some(at_pos) = email.find('@') {
        if at_pos > 0 && at_pos < email.len() - 1 {
            let domain = &email[at_pos + 1..];
            return domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.');
        }
    }
    false
}

/// Get attorney by ID
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "Attorney found", body = Attorney),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn get_attorney(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_attorney_by_id(id) {
        Ok(Some(attorney)) => json::success_response(&attorney),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Attorney {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorney by bar number
#[utoipa::path(
    get,
    path = "/api/attorneys/bar-number/{bar_number}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("bar_number" = String, Path, description = "Attorney Bar Number")
    ),
    responses(
        (status = 200, description = "Attorney found", body = Attorney),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn get_attorney_by_bar_number(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let bar_number = params.get("bar_number").unwrap_or_default();

    match repo.find_attorney_by_bar_number(bar_number) {
        Ok(Some(attorney)) => json::success_response(&attorney),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Attorney with bar number {} not found", bar_number))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Update attorney
#[utoipa::path(
    put,
    path = "/api/attorneys/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = UpdateAttorneyRequest,
    responses(
        (status = 200, description = "Attorney updated successfully", body = Attorney),
        (status = 400, description = "Invalid update data"),
        (status = 404, description = "Attorney not found"),
        (status = 409, description = "Conflict with existing attorney"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn update_attorney(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let attorney_id = params.get("id").unwrap_or_default().to_string();

    // Parse the update request
    let update_request: UpdateAttorneyRequest = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    // Get the existing attorney
    let mut existing = match repo.find_attorney_by_id(&attorney_id) {
        Ok(Some(attorney)) => attorney,
        Ok(None) => {
            return json::error_response(&ApiError::NotFound(
                format!("Attorney {} not found", attorney_id)
            ));
        }
        Err(e) => return json::error_response(&ApiError::StorageError(e.to_string())),
    };

    // Apply updates only for fields that are provided (Some values)
    if let Some(bar_number) = update_request.bar_number {
        // Check for duplicate if bar number is changing
        if existing.bar_number != bar_number {
            match repo.find_attorney_by_bar_number(&bar_number) {
                Ok(Some(_)) => {
                    return json::error_response(&ApiError::Conflict(
                        format!("Attorney with bar number {} already exists", bar_number)
                    ));
                }
                Ok(None) => existing.bar_number = bar_number,
                Err(e) => return json::error_response(&ApiError::StorageError(e.to_string())),
            }
        }
    }

    if let Some(first_name) = update_request.first_name {
        existing.first_name = first_name;
    }

    if let Some(last_name) = update_request.last_name {
        existing.last_name = last_name;
    }

    if let Some(middle_name) = update_request.middle_name {
        existing.middle_name = Some(middle_name);
    }

    if let Some(firm_name) = update_request.firm_name {
        existing.firm_name = Some(firm_name);
    }

    if let Some(email) = update_request.email {
        // Validate email format
        if !is_valid_email(&email) {
            return json::error_response(&ApiError::BadRequest(
                "Invalid email format".to_string()
            ));
        }
        existing.email = email;
    }

    if let Some(phone) = update_request.phone {
        existing.phone = phone;
    }

    if let Some(fax) = update_request.fax {
        existing.fax = Some(fax);
    }

    if let Some(address) = update_request.address {
        existing.address = address;
    }

    if let Some(bar_admissions) = update_request.bar_admissions {
        existing.bar_admissions = bar_admissions;
    }

    if let Some(federal_admissions) = update_request.federal_admissions {
        existing.federal_admissions = federal_admissions;
    }

    if let Some(pro_hac_vice_admissions) = update_request.pro_hac_vice_admissions {
        existing.pro_hac_vice_admissions = pro_hac_vice_admissions;
    }

    if let Some(status) = update_request.status {
        existing.status = status;
    }

    // Handle new optional fields from UpdateAttorneyRequest
    if let Some(ecf_registration) = update_request.ecf_registration {
        existing.ecf_registration = Some(ecf_registration);
    }

    if let Some(cja_panel_member) = update_request.cja_panel_member {
        existing.cja_panel_member = cja_panel_member;
    }

    if let Some(cja_panel_districts) = update_request.cja_panel_districts {
        existing.cja_panel_districts = cja_panel_districts;
    }

    if let Some(cja_appointments) = update_request.cja_appointments {
        existing.cja_appointments = cja_appointments;
    }

    if let Some(practice_areas) = update_request.practice_areas {
        existing.practice_areas = practice_areas;
    }

    if let Some(languages_spoken) = update_request.languages_spoken {
        existing.languages_spoken = languages_spoken;
    }

    if let Some(discipline_history) = update_request.discipline_history {
        existing.discipline_history = discipline_history;
    }

    if let Some(cases_handled) = update_request.cases_handled {
        existing.cases_handled = cases_handled;
    }

    if let Some(win_rate_percentage) = update_request.win_rate_percentage {
        existing.win_rate_percentage = Some(win_rate_percentage);
    }

    if let Some(avg_case_duration_days) = update_request.avg_case_duration_days {
        existing.avg_case_duration_days = Some(avg_case_duration_days);
    }

    // Update the attorney
    match repo.update_attorney(existing) {
        Ok(updated) => json::success_response(&updated),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Delete attorney
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 204, description = "Attorney deleted successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn delete_attorney(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let id = params.get("id").unwrap_or_default();

    // First check if attorney exists
    match repo.find_attorney_by_id(id) {
        Ok(Some(_)) => {
            // Attorney exists, proceed with deletion
            match repo.delete_attorney(id) {
                Ok(_) => Response::builder().status(204).build(),
                Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
            }
        }
        Ok(None) => {
            // Attorney doesn't exist
            json::error_response(&ApiError::NotFound(
                format!("Attorney {} not found", id)
            ))
        }
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// List all attorneys with pagination
#[utoipa::path(
    get,
    path = "/api/attorneys",
    responses(
        (status = 200, description = "Paginated list of attorneys"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("page" = usize, Query, description = "Page number (1-indexed)", example = 1),
        ("limit" = usize, Query, description = "Items per page (max 100)", example = 20)
    ),
)]
pub fn list_attorneys(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    // Parse pagination parameters from query string
    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let page = query_parser::get_string(&parsed, "page")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query_parser::get_string(&parsed, "limit")
        .and_then(|s| s.parse::<usize>().ok())
        .map(|l| if l == 0 { 20 } else { l.min(100) })
        .unwrap_or(20);

    match repo.find_all_attorneys() {
        Ok(attorneys) => {
            let paginated = crate::domain::pagination::PaginatedResponse::from_full_list(
                attorneys,
                page,
                limit
            );
            json::success_response(&paginated)
        },
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Search attorneys with pagination
#[utoipa::path(
    get,
    path = "/api/attorneys/search",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("q" = String, Query, description = "Search query"),
        ("page" = usize, Query, description = "Page number (1-indexed)", example = 1),
        ("limit" = usize, Query, description = "Items per page (max 100)", example = 20)
    ),
    responses(
        (status = 200, description = "Paginated search results"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn search_attorneys(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let search_query = query_parser::get_string(&parsed, "q").unwrap_or_default();
    let page = query_parser::get_string(&parsed, "page")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query_parser::get_string(&parsed, "limit")
        .and_then(|s| s.parse::<usize>().ok())
        .map(|l| if l == 0 { 20 } else { l.min(100) })
        .unwrap_or(20);

    match repo.search_attorneys(&search_query) {
        Ok(attorneys) => {
            let paginated = crate::domain::pagination::PaginatedResponse::from_full_list(
                attorneys,
                page,
                limit
            );
            json::success_response(&paginated)
        },
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorneys by status
#[utoipa::path(
    get,
    path = "/api/attorneys/status/{status}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("status" = String, Path, description = "Attorney status")
    ),
    responses(
        (status = 200, description = "List of attorneys", body = Vec<Attorney>),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn get_attorneys_by_status(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let status_str = params.get("status").unwrap_or_default();
    let status: AttorneyStatus = match serde_json::from_str(&format!("\"{}\"", status_str)) {
        Ok(s) => s,
        Err(e) => return json::error_response(&ApiError::BadRequest(e.to_string())),
    };

    match repo.find_attorneys_by_status(status) {
        Ok(attorneys) => json::success_response(&attorneys),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorneys by firm
#[utoipa::path(
    get,
    path = "/api/attorneys/firm/{firm_name}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("firm_name" = String, Path, description = "Law firm name")
    ),
    responses(
        (status = 200, description = "List of attorneys", body = Vec<Attorney>),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn get_attorneys_by_firm(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let firm_name = params.get("firm_name").unwrap_or_default();

    match repo.find_attorneys_by_firm(firm_name) {
        Ok(attorneys) => json::success_response(&attorneys),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Bar Admission Endpoints

/// Add bar admission
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/bar-admissions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = BarAdmission,
    responses(
        (status = 204, description = "Bar admission added successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "bar-admissions",
)]
pub fn add_bar_admission(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let admission: BarAdmission = match json::parse_body(req.body()) {
        Ok(a) => a,
        Err(e) => return json::error_response(&e),
    };

    match repo.add_bar_admission(id, admission) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Remove bar admission
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/bar-admissions/{state}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID"),
        ("state" = String, Path, description = "State abbreviation")
    ),
    responses(
        (status = 204, description = "Bar admission removed successfully"),
        (status = 404, description = "Attorney or admission not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "bar-admissions",
)]
pub fn remove_bar_admission(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let state = params.get("state").unwrap_or_default();

    match repo.remove_bar_admission(id, state) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorneys by bar state
#[utoipa::path(
    get,
    path = "/api/attorneys/bar-state/{state}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("state" = String, Path, description = "State abbreviation")
    ),
    responses(
        (status = 200, description = "List of attorneys", body = Vec<Attorney>),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn get_attorneys_by_bar_state(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let state = params.get("state").unwrap_or_default();

    match repo.find_attorneys_by_bar_state(state) {
        Ok(attorneys) => json::success_response(&attorneys),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Federal Admission Endpoints

/// Add federal court admission
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/federal-admissions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = FederalAdmission,
    responses(
        (status = 204, description = "Federal admission added successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "federal-admissions",
)]
pub fn add_federal_admission(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let admission: FederalAdmission = match json::parse_body(req.body()) {
        Ok(a) => a,
        Err(e) => return json::error_response(&e),
    };

    match repo.add_federal_admission(id, admission) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Remove federal court admission
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/federal-admissions/{court}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID"),
        ("court" = String, Path, description = "Federal court abbreviation")
    ),
    responses(
        (status = 204, description = "Federal admission removed successfully"),
        (status = 404, description = "Attorney or admission not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "federal-admissions",
)]
pub fn remove_federal_admission(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let court = params.get("court").unwrap_or_default();

    match repo.remove_federal_admission(id, court) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorneys admitted to federal court
#[utoipa::path(
    get,
    path = "/api/attorneys/federal-court/{court}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("court" = String, Path, description = "Federal court abbreviation")
    ),
    responses(
        (status = 200, description = "List of attorneys", body = Vec<Attorney>),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn get_attorneys_admitted_to_court(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let court = params.get("court").unwrap_or_default();

    match repo.find_attorneys_admitted_to_court(court) {
        Ok(attorneys) => json::success_response(&attorneys),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Pro Hac Vice Endpoints

/// Add pro hac vice admission
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/pro-hac-vice",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = ProHacViceAdmission,
    responses(
        (status = 204, description = "Pro hac vice admission added successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pro-hac-vice",
)]
pub fn add_pro_hac_vice(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let admission: ProHacViceAdmission = match json::parse_body(req.body()) {
        Ok(a) => a,
        Err(e) => return json::error_response(&e),
    };

    match repo.add_pro_hac_vice(id, admission) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Update pro hac vice status
#[utoipa::path(
    patch,
    path = "/api/attorneys/{id}/pro-hac-vice/{case_id}/status",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID"),
        ("case_id" = String, Path, description = "Case ID"),
        ("status" = String, Query, description = "New status")
    ),
    responses(
        (status = 204, description = "Status updated successfully"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pro-hac-vice",
)]
pub fn update_pro_hac_vice_status(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let case_id = params.get("case_id").unwrap_or_default();

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let status = query_parser::get_string(&parsed, "status").unwrap_or_default();

    match repo.update_pro_hac_vice_status(id, case_id, status) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get active pro hac vice admissions
#[utoipa::path(
    get,
    path = "/api/attorneys/pro-hac-vice/active",
    responses(
        (status = 200, description = "List of active admissions", body = Vec<ProHacViceAdmission>),
        (status = 500, description = "Internal server error")
    ),
    tag = "pro-hac-vice",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_active_pro_hac_vice(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    match repo.find_active_pro_hac_vice() {
        Ok(admissions) => json::success_response(&admissions),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get pro hac vice by case
#[utoipa::path(
    get,
    path = "/api/attorneys/pro-hac-vice/case/{case_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = String, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of admissions", body = Vec<ProHacViceAdmission>),
        (status = 500, description = "Internal server error")
    ),
    tag = "pro-hac-vice",
)]
pub fn get_pro_hac_vice_by_case(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let case_id = params.get("case_id").unwrap_or_default();

    match repo.find_pro_hac_vice_by_case(case_id) {
        Ok(admissions) => json::success_response(&admissions),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// CJA Panel Endpoints

/// Add attorney to CJA panel
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/cja-panel/{district}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID"),
        ("district" = String, Path, description = "District code")
    ),
    responses(
        (status = 204, description = "Added to CJA panel successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "cja",
)]
pub fn add_to_cja_panel(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let district = params.get("district").unwrap_or_default();

    match repo.add_to_cja_panel(id, district) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Remove attorney from CJA panel
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/cja-panel/{district}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID"),
        ("district" = String, Path, description = "District code")
    ),
    responses(
        (status = 204, description = "Removed from CJA panel successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "cja",
)]
pub fn remove_from_cja_panel(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let district = params.get("district").unwrap_or_default();

    match repo.remove_from_cja_panel(id, district) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get CJA panel attorneys for district
#[utoipa::path(
    get,
    path = "/api/attorneys/cja-panel/{district}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("district" = String, Path, description = "District code")
    ),
    responses(
        (status = 200, description = "List of CJA panel attorneys", body = Vec<Attorney>),
        (status = 500, description = "Internal server error")
    ),
    tag = "cja",
)]
pub fn get_cja_panel_attorneys(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let district = params.get("district").unwrap_or_default();

    match repo.find_cja_panel_attorneys(district) {
        Ok(attorneys) => json::success_response(&attorneys),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Add CJA appointment
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/cja-appointments",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = CJAAppointment,
    responses(
        (status = 204, description = "CJA appointment added successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "cja",
)]
pub fn add_cja_appointment(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let appointment: CJAAppointment = match json::parse_body(req.body()) {
        Ok(a) => a,
        Err(e) => return json::error_response(&e),
    };

    match repo.add_cja_appointment(id, appointment) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get CJA appointments for attorney
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/cja-appointments",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "List of CJA appointments", body = Vec<CJAAppointment>),
        (status = 500, description = "Internal server error")
    ),
    tag = "cja",
)]
pub fn get_cja_appointments(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_cja_appointments_by_attorney(id) {
        Ok(appointments) => json::success_response(&appointments),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get pending CJA vouchers
#[utoipa::path(
    get,
    path = "/api/attorneys/cja/pending-vouchers",
    responses(
        (status = 200, description = "List of pending vouchers", body = Vec<CJAAppointment>),
        (status = 500, description = "Internal server error")
    ),
    tag = "cja",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_pending_cja_vouchers(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    match repo.find_pending_cja_vouchers() {
        Ok(vouchers) => json::success_response(&vouchers),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// ECF Registration Endpoints

/// Update ECF registration
#[utoipa::path(
    put,
    path = "/api/attorneys/{id}/ecf-registration",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = ECFRegistration,
    responses(
        (status = 204, description = "ECF registration updated successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "ecf",
)]
pub fn update_ecf_registration(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let registration: ECFRegistration = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    match repo.update_ecf_registration(id, registration) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Check if attorney is in good standing
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/good-standing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "Standing status", body = bool),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn check_good_standing(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_attorney_by_id(id) {
        Ok(Some(attorney)) => json::success_response(&attorney.is_in_good_standing()),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Attorney {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Check if attorney can practice in federal court
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/can-practice/{court}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID"),
        ("court" = String, Path, description = "Federal court abbreviation")
    ),
    responses(
        (status = 200, description = "Practice eligibility", body = bool),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn check_federal_practice(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let court = params.get("court").unwrap_or_default();

    match repo.find_attorney_by_id(id) {
        Ok(Some(attorney)) => json::success_response(&attorney.can_practice_federal(court)),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Attorney {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Check if attorney has ECF privileges
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/has-ecf-privileges",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "ECF privilege status", body = bool),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "ecf",
)]
pub fn check_ecf_privileges(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_attorney_by_id(id) {
        Ok(Some(attorney)) => json::success_response(&attorney.has_ecf_privileges()),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Attorney {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorneys with ECF access
#[utoipa::path(
    get,
    path = "/api/attorneys/ecf-access",
    responses(
        (status = 200, description = "List of attorneys with ECF access", body = Vec<Attorney>),
        (status = 500, description = "Internal server error")
    ),
    tag = "ecf",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_attorneys_with_ecf(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    match repo.find_attorneys_with_ecf_access() {
        Ok(attorneys) => json::success_response(&attorneys),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Revoke ECF access
#[utoipa::path(
    delete,
    path = "/api/attorneys/{id}/ecf-access",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 204, description = "ECF access revoked successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "ecf",
)]
pub fn revoke_ecf_access(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.revoke_ecf_access(id) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Disciplinary Action Endpoints

/// Add disciplinary action
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/disciplinary-actions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = DisciplinaryAction,
    responses(
        (status = 204, description = "Disciplinary action added successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "discipline",
)]
pub fn add_disciplinary_action(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let action: DisciplinaryAction = match json::parse_body(req.body()) {
        Ok(a) => a,
        Err(e) => return json::error_response(&e),
    };

    match repo.add_disciplinary_action(id, action) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get disciplinary history
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/disciplinary-actions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "List of disciplinary actions", body = Vec<DisciplinaryAction>),
        (status = 500, description = "Internal server error")
    ),
    tag = "discipline",
)]
pub fn get_disciplinary_history(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_disciplinary_history(id) {
        Ok(actions) => json::success_response(&actions),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorneys with discipline
#[utoipa::path(
    get,
    path = "/api/attorneys/with-discipline",
    responses(
        (status = 200, description = "List of attorneys with disciplinary actions", body = Vec<Attorney>),
        (status = 500, description = "Internal server error")
    ),
    tag = "discipline",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_attorneys_with_discipline(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    match repo.find_attorneys_with_discipline() {
        Ok(attorneys) => json::success_response(&attorneys),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Party Management Endpoints

/// Create a party
#[utoipa::path(
    post,
    path = "/api/parties",
    request_body = CreatePartyRequest,
    responses(
        (status = 200, description = "Party created successfully", body = Party),
        (status = 400, description = "Invalid party data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_party(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let request: CreatePartyRequest = match json::parse_body(req.body()) {
        Ok(p) => p,
        Err(e) => return json::error_response(&e),
    };

    // Use the constructor to properly initialize the party
    let mut party = Party::new(
        request.case_id,
        request.party_type,
        request.name.clone(),
        request.entity_type,
    );

    // Set optional fields
    if let Some(role) = request.party_role {
        party.party_role = role;
    }
    party.first_name = request.first_name;
    party.middle_name = request.middle_name;
    party.last_name = request.last_name;
    party.date_of_birth = request.date_of_birth;
    party.ssn_last_four = request.ssn_last_four;
    party.organization_name = request.organization_name;
    party.ein = request.ein;
    party.address = request.address;
    party.phone = request.phone;
    party.email = request.email;

    match repo.save_party(party) {
        Ok(saved) => json::success_response(&saved),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get party by ID
#[utoipa::path(
    get,
    path = "/api/parties/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Party ID")
    ),
    responses(
        (status = 200, description = "Party found", body = Party),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn get_party(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_party_by_id(id) {
        Ok(Some(party)) => json::success_response(&party),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Party {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Update party
#[utoipa::path(
    put,
    path = "/api/parties/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Party ID")
    ),
    request_body = Party,
    responses(
        (status = 200, description = "Party updated successfully", body = Party),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn update_party(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let mut party: Party = match json::parse_body(req.body()) {
        Ok(p) => p,
        Err(e) => return json::error_response(&e),
    };

    party.id = params.get("id").unwrap_or_default().to_string();

    match repo.update_party(party) {
        Ok(updated) => json::success_response(&updated),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Delete party
#[utoipa::path(
    delete,
    path = "/api/parties/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Party ID")
    ),
    responses(
        (status = 204, description = "Party deleted successfully"),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn delete_party(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.delete_party(id) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// List parties by case
#[utoipa::path(
    get,
    path = "/api/parties/case/{case_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = String, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of parties", body = Vec<Party>),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn list_parties_by_case(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let case_id = params.get("case_id").unwrap_or_default();

    match repo.find_parties_by_case(case_id) {
        Ok(parties) => json::success_response(&parties),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// List parties by attorney
#[utoipa::path(
    get,
    path = "/api/parties/attorney/{attorney_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "List of parties", body = Vec<Party>),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn list_parties_by_attorney(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let attorney_id = params.get("attorney_id").unwrap_or_default();

    match repo.find_parties_by_attorney(attorney_id) {
        Ok(parties) => json::success_response(&parties),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Update party status
#[utoipa::path(
    patch,
    path = "/api/parties/{id}/status",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Party ID"),
        ("status" = String, Query, description = "New party status")
    ),
    responses(
        (status = 204, description = "Status updated successfully"),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn update_party_status(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let status_str = query_parser::get_string(&parsed, "status").unwrap_or_default();

    let status: PartyStatus = match serde_json::from_str(&format!("\"{}\"", status_str)) {
        Ok(s) => s,
        Err(e) => return json::error_response(&ApiError::BadRequest(e.to_string())),
    };

    match repo.update_party_status(id, status) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Check if party needs service
#[utoipa::path(
    get,
    path = "/api/parties/{id}/needs-service",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Party ID")
    ),
    responses(
        (status = 200, description = "Service requirement status", body = bool),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn check_party_needs_service(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_party_by_id(id) {
        Ok(Some(party)) => json::success_response(&party.needs_service()),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Party {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get party's lead counsel
#[utoipa::path(
    get,
    path = "/api/parties/{id}/lead-counsel",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Party ID")
    ),
    responses(
        (status = 200, description = "Lead counsel representation", body = Option<AttorneyRepresentation>),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn get_party_lead_counsel(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_party_by_id(id) {
        Ok(Some(party)) => json::success_response(&party.get_lead_counsel()),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Party {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Check if party is represented
#[utoipa::path(
    get,
    path = "/api/parties/{id}/is-represented",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Party ID")
    ),
    responses(
        (status = 200, description = "Representation status", body = bool),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
)]
pub fn check_party_represented(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_party_by_id(id) {
        Ok(Some(party)) => json::success_response(&party.is_represented()),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Party {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get unrepresented parties
#[utoipa::path(
    get,
    path = "/api/parties/unrepresented",
    responses(
        (status = 200, description = "List of unrepresented parties", body = Vec<Party>),
        (status = 500, description = "Internal server error")
    ),
    tag = "parties",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_unrepresented_parties(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    match repo.find_unrepresented_parties() {
        Ok(parties) => json::success_response(&parties),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Representation Endpoints

/// Add attorney representation
#[utoipa::path(
    post,
    path = "/api/representations",
    request_body = AttorneyRepresentation,
    responses(
        (status = 204, description = "Representation added successfully"),
        (status = 400, description = "Invalid representation data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "representation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn add_representation(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let representation: AttorneyRepresentation = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    match repo.add_representation(representation) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// End attorney representation
#[utoipa::path(
    post,
    path = "/api/representations/{id}/end",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Representation ID"),
        ("reason" = String, Query, description = "Withdrawal reason", nullable = true)
    ),
    responses(
        (status = 204, description = "Representation ended successfully"),
        (status = 404, description = "Representation not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "representation",
)]
pub fn end_representation(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let reason = query_parser::get_string(&parsed, "reason").map(|s| s.to_string());

    match repo.end_representation(id, reason) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get representation by ID
#[utoipa::path(
    get,
    path = "/api/representations/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Representation ID")
    ),
    responses(
        (status = 200, description = "Representation found", body = AttorneyRepresentation),
        (status = 404, description = "Representation not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "representation",
)]
pub fn get_representation(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.find_representation_by_id(id) {
        Ok(Some(rep)) => json::success_response(&rep),
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Representation {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get active representations for attorney
#[utoipa::path(
    get,
    path = "/api/representations/attorney/{attorney_id}/active",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "List of active representations", body = Vec<AttorneyRepresentation>),
        (status = 500, description = "Internal server error")
    ),
    tag = "representation",
)]
pub fn get_active_representations(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let attorney_id = params.get("attorney_id").unwrap_or_default();

    match repo.find_active_representations(attorney_id) {
        Ok(reps) => json::success_response(&reps),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get representations by case
#[utoipa::path(
    get,
    path = "/api/representations/case/{case_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = String, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of representations", body = Vec<AttorneyRepresentation>),
        (status = 500, description = "Internal server error")
    ),
    tag = "representation",
)]
pub fn get_case_representations(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let case_id = params.get("case_id").unwrap_or_default();

    match repo.find_representations_by_case(case_id) {
        Ok(reps) => json::success_response(&reps),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Substitute attorney
#[utoipa::path(
    post,
    path = "/api/representations/substitute",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("old_attorney_id" = String, Query, description = "Current attorney ID"),
        ("new_attorney_id" = String, Query, description = "New attorney ID"),
        ("case_id" = String, Query, description = "Case ID")
    ),
    responses(
        (status = 204, description = "Attorney substituted successfully"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "representation",
)]
pub fn substitute_attorney(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let old_attorney_id = query_parser::get_string(&parsed, "old_attorney_id").unwrap_or_default();
    let new_attorney_id = query_parser::get_string(&parsed, "new_attorney_id").unwrap_or_default();
    let case_id = query_parser::get_string(&parsed, "case_id").unwrap_or_default();

    match repo.substitute_attorney(&old_attorney_id, &new_attorney_id, &case_id) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Service Records Endpoints

/// Create service record
#[utoipa::path(
    post,
    path = "/api/service-records",
    request_body = ServiceRecord,
    responses(
        (status = 204, description = "Service record created successfully"),
        (status = 400, description = "Invalid service record data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "process-service",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_service_record(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let record: ServiceRecord = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    match repo.save_service_record(record) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get service records by document
#[utoipa::path(
    get,
    path = "/api/service-records/document/{document_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("document_id" = String, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "List of service records", body = Vec<ServiceRecord>),
        (status = 500, description = "Internal server error")
    ),
    tag = "process-service",
)]
pub fn get_service_by_document(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let document_id = params.get("document_id").unwrap_or_default();

    match repo.find_service_records_by_document(document_id) {
        Ok(records) => json::success_response(&records),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get service records by party
#[utoipa::path(
    get,
    path = "/api/service-records/party/{party_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("party_id" = String, Path, description = "Party ID")
    ),
    responses(
        (status = 200, description = "List of service records", body = Vec<ServiceRecord>),
        (status = 500, description = "Internal server error")
    ),
    tag = "process-service",
)]
pub fn get_service_by_party(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let party_id = params.get("party_id").unwrap_or_default();

    match repo.find_service_records_by_party(party_id) {
        Ok(records) => json::success_response(&records),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Mark service as completed
#[utoipa::path(
    post,
    path = "/api/service-records/{id}/complete",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Service record ID")
    ),
    responses(
        (status = 204, description = "Service marked as completed"),
        (status = 404, description = "Service record not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "process-service",
)]
pub fn mark_service_completed(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.mark_service_completed(id) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Conflict Checking Endpoints

/// Create conflict check
#[utoipa::path(
    post,
    path = "/api/conflict-checks",
    request_body = ConflictCheck,
    responses(
        (status = 204, description = "Conflict check created successfully"),
        (status = 400, description = "Invalid conflict check data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "conflicts",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_conflict_check(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let check: ConflictCheck = match json::parse_body(req.body()) {
        Ok(c) => c,
        Err(e) => return json::error_response(&e),
    };

    match repo.save_conflict_check(check) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get conflict checks for attorney
#[utoipa::path(
    get,
    path = "/api/conflict-checks/attorney/{attorney_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "List of conflict checks", body = Vec<ConflictCheck>),
        (status = 500, description = "Internal server error")
    ),
    tag = "conflicts",
)]
pub fn get_attorney_conflicts(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let attorney_id = params.get("attorney_id").unwrap_or_default();

    match repo.find_conflict_checks_by_attorney(attorney_id) {
        Ok(checks) => json::success_response(&checks),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Check conflicts for parties
#[utoipa::path(
    post,
    path = "/api/conflict-checks/check",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("attorney_id" = String, Query, description = "Attorney ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Conflict check results", body = Vec<ConflictCheck>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "conflicts",
)]
pub fn check_party_conflicts(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let attorney_id = query_parser::get_string(&parsed, "attorney_id").unwrap_or_default();

    let party_names: Vec<String> = match json::parse_body(req.body()) {
        Ok(names) => names,
        Err(e) => return json::error_response(&e),
    };

    match repo.find_conflicts_for_parties(&attorney_id, party_names) {
        Ok(conflicts) => json::success_response(&conflicts),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Clear conflict
#[utoipa::path(
    post,
    path = "/api/conflict-checks/{id}/clear",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Conflict check ID"),
        ("waiver" = bool, Query, description = "Whether waiver was obtained")
    ),
    responses(
        (status = 204, description = "Conflict cleared successfully"),
        (status = 404, description = "Conflict check not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "conflicts",
)]
pub fn clear_conflict(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let waiver = query_parser::get_string(&parsed, "waiver")
        .map(|s| s == "true")
        .unwrap_or(false);

    match repo.clear_conflict(id, waiver) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Metrics Endpoints

/// Get attorney metrics
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/metrics",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID"),
        ("start" = String, Query, description = "Start date (YYYY-MM-DD)"),
        ("end" = String, Query, description = "End date (YYYY-MM-DD)")
    ),
    responses(
        (status = 200, description = "Attorney metrics", body = AttorneyMetrics),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "metrics",
)]
pub fn get_attorney_metrics(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let start = query_parser::get_string(&parsed, "start").unwrap_or_else(|| "2024-01-01".to_string());
    let end = query_parser::get_string(&parsed, "end").unwrap_or_else(|| "2024-12-31".to_string());

    match repo.calculate_attorney_metrics(id, &start, &end) {
        Ok(metrics) => json::success_response(&metrics),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorney win rate
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/win-rate",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "Win rate percentage", body = f64),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "metrics",
)]
pub fn get_attorney_win_rate(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.get_attorney_win_rate(id) {
        Ok(rate) => json::success_response(&rate),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorney case count
#[utoipa::path(
    get,
    path = "/api/attorneys/{id}/case-count",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "Total case count", body = i32),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "metrics",
)]
pub fn get_attorney_case_count(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();

    match repo.get_attorney_case_count(id) {
        Ok(count) => json::success_response(&count),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get top performing attorneys
#[utoipa::path(
    get,
    path = "/api/attorneys/top-performers",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("limit" = usize, Query, description = "Number of attorneys to return (default: 10)")
    ),
    responses(
        (status = 200, description = "Top performing attorneys with metrics"),
        (status = 500, description = "Internal server error")
    ),
    tag = "metrics",
)]
pub fn get_top_attorneys(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let limit = query_parser::get_string(&parsed, "limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    match repo.get_top_performing_attorneys(limit) {
        Ok(results) => json::success_response(&results),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

// Bulk Operations Endpoints

/// Bulk update attorney status
#[utoipa::path(
    post,
    path = "/api/attorneys/bulk/update-status",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("status" = String, Query, description = "New status for all attorneys")
    ),
    request_body = Vec<String>,
    responses(
        (status = 204, description = "Statuses updated successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn bulk_update_status(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let status_str = query_parser::get_string(&parsed, "status").unwrap_or_default();

    let status: AttorneyStatus = match serde_json::from_str(&format!("\"{}\"", status_str)) {
        Ok(s) => s,
        Err(e) => return json::error_response(&ApiError::BadRequest(e.to_string())),
    };

    let attorney_ids: Vec<String> = match json::parse_body(req.body()) {
        Ok(ids) => ids,
        Err(e) => return json::error_response(&e),
    };

    match repo.bulk_update_attorney_status(attorney_ids, status) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Bulk add parties to service list
#[utoipa::path(
    post,
    path = "/api/service-records/bulk/{document_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("document_id" = String, Path, description = "Document ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 204, description = "Parties added to service list"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "process-service",
)]
pub fn bulk_add_to_service(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let document_id = params.get("document_id").unwrap_or_default();

    let party_ids: Vec<String> = match json::parse_body(req.body()) {
        Ok(ids) => ids,
        Err(e) => return json::error_response(&e),
    };

    match repo.bulk_add_to_service_list(document_id, party_ids) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Migrate all representations
#[utoipa::path(
    post,
    path = "/api/representations/migrate",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("from" = String, Query, description = "Current attorney ID"),
        ("to" = String, Query, description = "New attorney ID")
    ),
    responses(
        (status = 204, description = "Representations migrated successfully"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "representation",
)]
pub fn migrate_representations(req: Request, _params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let query = req.query();
    let parsed = query_parser::parse_query_string(query);
    let from = query_parser::get_string(&parsed, "from").unwrap_or_default();
    let to = query_parser::get_string(&parsed, "to").unwrap_or_default();

    match repo.migrate_representations(&from, &to) {
        Ok(_) => Response::builder().status(204).build(),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}


/// Request for calculating win rate
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct WinRateRequest {
    pub wins: i32,
    pub total: i32,
}

/// Calculate attorney win rate
#[utoipa::path(
    post,
    path = "/api/attorneys/{id}/calculate-win-rate",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Attorney ID")
    ),
    request_body = WinRateRequest,
    responses(
        (status = 200, description = "Calculated win rate", body = f64),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
)]
pub fn calculate_attorney_win_rate(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {

        Ok(r) => r,

        Err(e) => return json::error_response(&e),

    };

    let id = params.get("id").unwrap_or_default();
    let request: WinRateRequest = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    match repo.find_attorney_by_id(id) {
        Ok(Some(attorney)) => {
            let win_rate = attorney.calculate_win_rate(request.wins, request.total);
            json::success_response(&win_rate)
        }
        Ok(None) => json::error_response(&ApiError::NotFound(format!("Attorney {} not found", id))),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}


// Attorney-Case Relationship Endpoints

/// Assign an attorney to a case
#[utoipa::path(
    post,
    path = "/api/attorneys/{attorney_id}/cases",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID")
    ),
    request_body = AssignAttorneyRequest,
    responses(
        (status = 201, description = "Attorney assigned to case", body = AttorneyCaseAssignment),
        (status = 404, description = "Attorney or case not found"),
        (status = 409, description = "Attorney already assigned to this case"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorney-cases",
)]
pub fn assign_attorney_to_case(req: Request, params: Params) -> Response {
    let _repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let _attorney_id = params.get("attorney_id").unwrap_or_default();

    // Parse request body
    let body = req.body();
    let request: AssignAttorneyRequest = match json::parse_body(&body) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    // For now, create a mock assignment
    // In a real implementation, this would:
    // 1. Verify attorney exists
    // 2. Verify case exists
    // 3. Check for conflicts
    // 4. Create the assignment
    let assignment = AttorneyCaseAssignment::new(
        request.attorney_id,
        request.case_id,
        request.role,
        request.notes,
    );

    Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&assignment).unwrap())
        .build()
}

/// Get all cases for an attorney
#[utoipa::path(
    get,
    path = "/api/attorneys/{attorney_id}/cases",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID"),
        ("active" = bool, Query, description = "Filter by active status")
    ),
    responses(
        (status = 200, description = "Attorney's case assignments", body = Vec<AttorneyCaseAssignment>),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorney-cases",
)]
pub fn get_attorney_cases(req: Request, params: Params) -> Response {
    let _repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let _attorney_id = params.get("attorney_id").unwrap_or_default();

    // For now, return empty list
    // In a real implementation, this would fetch from storage
    let assignments: Vec<AttorneyCaseAssignment> = vec![];

    json::success_response(&assignments)
}

/// Remove an attorney from a case
#[utoipa::path(
    delete,
    path = "/api/attorneys/{attorney_id}/cases/{case_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID"),
        ("case_id" = String, Path, description = "Case ID")
    ),
    request_body = RemoveAttorneyRequest,
    responses(
        (status = 204, description = "Attorney removed from case"),
        (status = 404, description = "Assignment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorney-cases",
)]
pub fn remove_attorney_from_case(req: Request, params: Params) -> Response {
    let _repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let _attorney_id = params.get("attorney_id").unwrap_or_default();
    let _case_id = params.get("case_id").unwrap_or_default();

    // Parse optional request body for removal reason
    let body = req.body();
    let _request: Option<RemoveAttorneyRequest> = if !body.is_empty() {
        match json::parse_body(&body) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    } else {
        None
    };

    // For now, just return success
    // In a real implementation, this would update the assignment
    Response::builder().status(204).build()
}

/// Get attorney's case load summary
#[utoipa::path(
    get,
    path = "/api/attorneys/{attorney_id}/case-load",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID")
    ),
    responses(
        (status = 200, description = "Attorney's case load summary", body = AttorneyCaseLoad),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorney-cases",
)]
pub fn get_attorney_case_load(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let attorney_id = params.get("attorney_id").unwrap_or_default();

    // Get attorney to verify they exist
    match repo.find_attorney_by_id(attorney_id) {
        Ok(Some(attorney)) => {
            // Create mock case load summary
            let case_load = AttorneyCaseLoad {
                attorney_id: attorney.id,
                attorney_name: format!("{} {}", attorney.first_name, attorney.last_name),
                active_cases: 0,
                completed_cases: 0,
                active_assignments: vec![],
            };

            json::success_response(&case_load)
        },
        Ok(None) => json::error_response(&ApiError::NotFound("Attorney not found".to_string())),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Get attorney's complete representation history
#[utoipa::path(
    get,
    path = "/api/attorneys/{attorney_id}/representation-history",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID"),
        ("start_date" = Option<String>, Query, description = "Start date filter (ISO 8601)", example = "2023-01-01T00:00:00Z"),
        ("end_date" = Option<String>, Query, description = "End date filter (ISO 8601)", example = "2023-12-31T23:59:59Z"),
        ("status" = Option<String>, Query, description = "Filter by case status", example = "active"),
        ("role" = Option<String>, Query, description = "Filter by representation role", example = "lead_counsel"),
        ("active_only" = Option<bool>, Query, description = "Include only active assignments", example = "true"),
        ("page" = Option<usize>, Query, description = "Page number for pagination", example = "1"),
        ("page_size" = Option<usize>, Query, description = "Page size for pagination", example = "50")
    ),
    responses(
        (status = 200, description = "Attorney's representation history", body = AttorneyRepresentationHistory),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorney-cases",
)]
pub fn get_attorney_representation_history(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let attorney_id = params.get("attorney_id").unwrap_or_default();

    // Parse query parameters
    let query_string = req.query();
    let parsed_params = query_parser::parse_query_string(query_string);

    // Helper function to get string from parsed params
    let get_param = |key: &str| -> Option<String> {
        parsed_params.iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.to_string())
    };

    let query = crate::domain::attorney_case::RepresentationHistoryQuery {
        start_date: get_param("start_date"),
        end_date: get_param("end_date"),
        status: get_param("status"),
        role: get_param("role")
            .and_then(|r| serde_json::from_str(&format!("\"{}\"", r)).ok()),
        active_only: get_param("active_only")
            .and_then(|v| v.parse().ok()),
        page: get_param("page")
            .and_then(|v| v.parse().ok()),
        page_size: get_param("page_size")
            .and_then(|v| v.parse().ok()),
    };

    // Get attorney to verify they exist
    match repo.find_attorney_by_id(attorney_id) {
        Ok(Some(attorney)) => {
            // Create mock representation history
            // In a real implementation, this would fetch from storage and apply filters
            let mock_assignments = create_mock_representation_history(attorney_id, &attorney, &query);

            let history = crate::domain::attorney_case::AttorneyRepresentationHistory {
                attorney_id: attorney.id.clone(),
                attorney_name: format!("{} {}", attorney.first_name, attorney.last_name),
                assignments: mock_assignments.clone(),
                summary: create_representation_summary(&mock_assignments, &attorney),
            };

            json::success_response(&history)
        },
        Ok(None) => json::error_response(&ApiError::NotFound("Attorney not found".to_string())),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Helper function to create mock representation history data
fn create_mock_representation_history(
    attorney_id: &str,
    attorney: &crate::domain::attorney::Attorney,
    query: &crate::domain::attorney_case::RepresentationHistoryQuery,
) -> Vec<crate::domain::attorney_case::RepresentationHistoryEntry> {
    use chrono::{Duration, Utc};
    use crate::domain::attorney_case::{RepresentationRole, RepresentationHistoryEntry, RoleChange};

    let now = Utc::now();
    let mut assignments = vec![];

    // Create some sample historical assignments
    let sample_cases = vec![
        ("case_001", "CR-2023-001", "John Smith", RepresentationRole::LeadCounsel, Duration::days(365), Some("Plea agreement")),
        ("case_002", "CR-2023-015", "Jane Doe", RepresentationRole::CoCounsel, Duration::days(180), None),
        ("case_003", "CR-2022-087", "Bob Johnson", RepresentationRole::PublicDefender, Duration::days(450), Some("Dismissal")),
        ("case_004", "CR-2024-003", "Alice Brown", RepresentationRole::LeadCounsel, Duration::days(90), None),
        ("case_005", "CR-2021-234", "Charlie Wilson", RepresentationRole::AppellateCounsel, Duration::days(720), Some("Appeal successful")),
    ];

    for (i, (case_id, case_number, defendant, role, duration_ago, outcome)) in sample_cases.iter().enumerate() {
        let assigned_date = now - *duration_ago;
        let is_active = outcome.is_none();
        let removed_date = if is_active { None } else { Some(assigned_date + Duration::days(30 + i as i64 * 10)) };

        // Apply filters if specified
        if let Some(active_only) = query.active_only {
            if active_only && !is_active {
                continue;
            }
        }

        if let Some(ref role_filter) = query.role {
            if role_filter != role {
                continue;
            }
        }

        if let Some(ref status_filter) = query.status {
            let case_status = if is_active { "active" } else { "completed" };
            if status_filter != case_status {
                continue;
            }
        }

        // Apply date range filters
        if let Some(ref start_date_str) = query.start_date {
            if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(start_date_str) {
                if assigned_date < start_date.with_timezone(&Utc) {
                    continue;
                }
            }
        }

        if let Some(ref end_date_str) = query.end_date {
            if let Ok(end_date) = chrono::DateTime::parse_from_rfc3339(end_date_str) {
                if assigned_date > end_date.with_timezone(&Utc) {
                    continue;
                }
            }
        }

        // Create some mock role changes for certain cases
        let role_changes = if i == 1 {
            vec![RoleChange {
                from_role: RepresentationRole::CoCounsel,
                to_role: RepresentationRole::LeadCounsel,
                change_date: assigned_date + Duration::days(15),
                reason: Some("Lead counsel withdrew".to_string()),
            }]
        } else {
            vec![]
        };

        assignments.push(RepresentationHistoryEntry {
            assignment_id: format!("assignment_{}", i + 1),
            case_id: case_id.to_string(),
            case_number: case_number.to_string(),
            defendant_name: defendant.to_string(),
            role: role.clone(),
            assigned_date,
            removed_date,
            is_active,
            case_outcome: outcome.map(|o| o.to_string()),
            role_changes,
            notes: Some(format!("Assignment for {}", defendant)),
        });
    }

    // Apply pagination
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(50);
    let start_idx = (page - 1) * page_size;
    let end_idx = std::cmp::min(start_idx + page_size, assignments.len());

    if start_idx < assignments.len() {
        assignments[start_idx..end_idx].to_vec()
    } else {
        vec![]
    }
}

/// Helper function to create representation summary
fn create_representation_summary(
    assignments: &[crate::domain::attorney_case::RepresentationHistoryEntry],
    attorney: &crate::domain::attorney::Attorney,
) -> crate::domain::attorney_case::RepresentationSummary {
    use crate::domain::attorney_case::{RepresentationRole, RepresentationSummary, DateRange};
    use std::collections::HashMap;

    let total_cases = assignments.len();
    let active_cases = assignments.iter().filter(|a| a.is_active).count();
    let completed_cases = total_cases - active_cases;

    // Find most common role
    let mut role_counts = HashMap::new();
    for assignment in assignments {
        *role_counts.entry(&assignment.role).or_insert(0) += 1;
    }
    let primary_role = role_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(role, _)| (*role).clone())
        .unwrap_or(RepresentationRole::LeadCounsel);

    // Calculate date range
    let start_date = assignments
        .iter()
        .map(|a| a.assigned_date)
        .min()
        .unwrap_or_else(|| chrono::Utc::now());
    let end_date = assignments
        .iter()
        .map(|a| a.removed_date.unwrap_or_else(|| chrono::Utc::now()))
        .max()
        .unwrap_or_else(|| chrono::Utc::now());

    // Count outcomes
    let mut outcomes = HashMap::new();
    for assignment in assignments {
        if let Some(ref outcome) = assignment.case_outcome {
            *outcomes.entry(outcome.clone()).or_insert(0) += 1;
        }
    }

    RepresentationSummary {
        total_cases,
        active_cases,
        completed_cases,
        primary_role,
        date_range: DateRange {
            start_date,
            end_date,
        },
        outcomes,
    }
}

/// Check for conflicts of interest
#[utoipa::path(
    post,
    path = "/api/attorneys/{attorney_id}/conflict-check",
    request_body = ConflictCheckRequest,
    responses(
        (status = 200, description = "Conflict check completed", body = ConflictCheckResult),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Attorney not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "attorneys",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("attorney_id" = String, Path, description = "Attorney ID to check conflicts for")
    ),
)]
pub fn check_attorney_conflicts(req: Request, params: Params) -> Response {
    let repo = match RepositoryFactory::attorney_repo(&req) {
        Ok(r) => r,
        Err(e) => return json::error_response(&e),
    };

    let attorney_id = params.get("attorney_id").unwrap_or_default();

    // Verify attorney exists
    let attorney = match repo.find_attorney_by_id(attorney_id) {
        Ok(Some(attorney)) => attorney,
        Ok(None) => return json::error_response(&ApiError::NotFound("Attorney not found".to_string())),
        Err(e) => return json::error_response(&ApiError::StorageError(e.to_string())),
    };

    // Parse the conflict check request
    let request: ConflictCheckRequest = match json::parse_body(req.body()) {
        Ok(req) => req,
        Err(e) => return json::error_response(&e),
    };

    // Validate request data
    if request.parties_to_check.is_empty() && request.adverse_parties.is_empty() {
        return json::error_response(&ApiError::BadRequest(
            "At least one party must be specified for conflict checking".to_string()
        ));
    }

    // Perform the conflict check
    let result = perform_conflict_check(&attorney, &request, &repo);

    match result {
        Ok(conflict_result) => json::success_response(&conflict_result),
        Err(e) => json::error_response(&ApiError::StorageError(e.to_string())),
    }
}

/// Perform comprehensive conflict checking for an attorney
fn perform_conflict_check(
    attorney: &Attorney,
    request: &ConflictCheckRequest,
    repo: &dyn AttorneyRepository,
) -> Result<ConflictCheckResult, Box<dyn std::error::Error>> {
    let mut conflicts = Vec::new();
    let mut all_parties_to_check = request.parties_to_check.clone();
    all_parties_to_check.extend(request.adverse_parties.clone());

    // Check current representations
    let current_conflicts = check_current_representations(attorney, &all_parties_to_check, repo)?;
    conflicts.extend(current_conflicts);

    // Check former client conflicts
    let former_client_conflicts = check_former_clients(attorney, &all_parties_to_check, repo)?;
    conflicts.extend(former_client_conflicts);

    // Check co-defendant conflicts
    let codefendant_conflicts = check_codefendant_conflicts(attorney, &all_parties_to_check, repo)?;
    conflicts.extend(codefendant_conflicts);

    // Check related party conflicts (family, business relationships)
    let related_party_conflicts = check_related_parties(attorney, &all_parties_to_check)?;
    conflicts.extend(related_party_conflicts);

    // Determine overall recommendation
    let recommendation = determine_recommendation(&conflicts);

    // Create the result
    let result = if conflicts.is_empty() {
        ConflictCheckResult::no_conflicts(attorney.id.clone(), request)
    } else {
        ConflictCheckResult::with_conflicts(attorney.id.clone(), conflicts, recommendation)
    };

    Ok(result)
}

/// Check for direct representation conflicts (currently representing party or adverse party)
fn check_current_representations(
    attorney: &Attorney,
    parties: &[String],
    repo: &dyn AttorneyRepository,
) -> Result<Vec<ConflictDetails>, Box<dyn std::error::Error>> {
    let mut conflicts = Vec::new();

    // Get attorney's current active representations
    // In a real implementation, this would query the database for active representations
    // For now, we'll create a mock check
    for party in parties {
        if is_currently_representing(attorney, party, repo)? {
            let conflict = ConflictDetails::new(
                ConflictType::DirectRepresentation,
                party.clone(),
                format!("Currently representing {} in an active matter", party),
                ConflictSeverity::Critical,
                false, // Direct representation conflicts typically cannot be waived
            );
            conflicts.push(conflict);
        }
    }

    Ok(conflicts)
}

/// Check for former client conflicts in substantially related matters
fn check_former_clients(
    attorney: &Attorney,
    parties: &[String],
    repo: &dyn AttorneyRepository,
) -> Result<Vec<ConflictDetails>, Box<dyn std::error::Error>> {
    let mut conflicts = Vec::new();

    // Get attorney's representation history
    // In a real implementation, this would check for substantially related matters
    for party in parties {
        if is_former_client(attorney, party, repo)? {
            let conflict = ConflictDetails::new(
                ConflictType::FormerClient,
                party.clone(),
                format!("Previously represented {} in a substantially related matter", party),
                ConflictSeverity::High,
                true, // Former client conflicts can sometimes be waived
            );
            conflicts.push(conflict);
        }
    }

    Ok(conflicts)
}

/// Check for co-defendant conflicts
fn check_codefendant_conflicts(
    attorney: &Attorney,
    parties: &[String],
    repo: &dyn AttorneyRepository,
) -> Result<Vec<ConflictDetails>, Box<dyn std::error::Error>> {
    let mut conflicts = Vec::new();

    // Check if attorney is representing co-defendants in related matters
    for party in parties {
        if is_codefendant_conflict(attorney, party, repo)? {
            let conflict = ConflictDetails::new(
                ConflictType::CoDefendant,
                party.clone(),
                format!("Representing co-defendant {} in related criminal matter", party),
                ConflictSeverity::Medium,
                true, // Co-defendant conflicts may be waivable depending on circumstances
            );
            conflicts.push(conflict);
        }
    }

    Ok(conflicts)
}

/// Check for related party conflicts (family, business relationships)
fn check_related_parties(
    attorney: &Attorney,
    parties: &[String],
) -> Result<Vec<ConflictDetails>, Box<dyn std::error::Error>> {
    let mut conflicts = Vec::new();

    // In a real implementation, this would check known relationships
    // For demonstration, we'll create mock conflicts for certain scenarios
    for party in parties {
        // Mock check for family relationships
        if party.to_lowercase().contains("smith") && attorney.last_name.to_lowercase().contains("smith") {
            let conflict = ConflictDetails::new(
                ConflictType::FamilyRelationship,
                party.clone(),
                format!("Potential family relationship with {}", party),
                ConflictSeverity::Medium,
                false, // Family relationship conflicts typically cannot be waived
            );
            conflicts.push(conflict);
        }

        // Mock check for business relationships
        if let Some(firm_name) = &attorney.firm_name {
            if party.to_lowercase().contains(&firm_name.to_lowercase()) {
                let conflict = ConflictDetails::new(
                    ConflictType::BusinessRelationship,
                    party.clone(),
                    format!("Business relationship through law firm {}", firm_name),
                    ConflictSeverity::High,
                    false, // Business conflicts typically cannot be waived
                );
                conflicts.push(conflict);
            }
        }
    }

    Ok(conflicts)
}

/// Determine overall recommendation based on conflicts found
fn determine_recommendation(conflicts: &[ConflictDetails]) -> ConflictRecommendation {
    if conflicts.is_empty() {
        return ConflictRecommendation::Proceed;
    }

    let has_critical = conflicts.iter().any(|c| c.severity == ConflictSeverity::Critical);
    let has_non_waivable = conflicts.iter().any(|c| !c.waivable);
    let has_high_severity = conflicts.iter().any(|c| c.severity == ConflictSeverity::High);

    if has_critical || has_non_waivable {
        ConflictRecommendation::MustDecline
    } else if has_high_severity {
        ConflictRecommendation::RequireWaivers
    } else {
        ConflictRecommendation::ProceedWithCaution
    }
}

/// Mock helper function to check if attorney is currently representing a party
fn is_currently_representing(
    attorney: &Attorney,
    party: &str,
    _repo: &dyn AttorneyRepository,
) -> Result<bool, Box<dyn std::error::Error>> {
    // In a real implementation, this would query the database for active representations
    // For demonstration, we'll create mock conflicts for certain party names
    Ok(party.to_lowercase().contains("current_client") ||
       party.to_lowercase().contains("active_case"))
}

/// Mock helper function to check if a party is a former client
fn is_former_client(
    attorney: &Attorney,
    party: &str,
    _repo: &dyn AttorneyRepository,
) -> Result<bool, Box<dyn std::error::Error>> {
    // In a real implementation, this would check representation history
    Ok(party.to_lowercase().contains("former_client") ||
       party.to_lowercase().contains("previous_case"))
}

/// Mock helper function to check for co-defendant conflicts
fn is_codefendant_conflict(
    attorney: &Attorney,
    party: &str,
    _repo: &dyn AttorneyRepository,
) -> Result<bool, Box<dyn std::error::Error>> {
    // In a real implementation, this would check for co-defendants in related cases
    Ok(party.to_lowercase().contains("codefendant") ||
       party.to_lowercase().contains("co_defendant"))
}
