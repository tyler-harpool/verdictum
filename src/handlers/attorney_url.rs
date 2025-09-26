//! URL-based attorney management HTTP handlers
//!
//! These handlers wrap the existing attorney handlers but extract tenant
//! information from the URL path instead of headers.

use spin_sdk::http::{IntoResponse, Params, Request, Response};
use crate::utils::json_response as json;

/// Helper to create a new request with district header from URL parameter
fn add_district_header(req: Request, params: &Params) -> Result<Request, crate::error::ApiError> {
    let district = params.get("district")
        .ok_or_else(|| crate::error::ApiError::BadRequest(
            "District parameter is required in URL".to_string()
        ))?;

    // Extract method and URI before consuming the request
    let method = req.method().clone();
    let uri = req.uri().to_string();

    // Create a new request with the district header
    let headers = spin_sdk::http::Headers::new();

    // Copy existing headers
    for (name, value) in req.headers() {
        let _ = headers.append(&name.to_string(), &value.as_bytes().to_vec());
    }

    // Add the district header
    let _ = headers.set(&"x-court-district".to_string(), &vec![district.as_bytes().to_vec()]);

    let body = req.into_body();
    let new_req = Request::builder()
        .method(method)
        .uri(uri)
        .headers(headers)
        .body(body)
        .build();

    Ok(new_req)
}

// ============================================================================
// Attorney Management Wrapper Functions - Generated from 67 endpoints
// ============================================================================

pub fn add_bar_admission(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::add_bar_admission(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_cja_appointment(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::add_cja_appointment(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_disciplinary_action(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::add_disciplinary_action(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_federal_admission(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::add_federal_admission(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_pro_hac_vice(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::add_pro_hac_vice(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_representation(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::add_representation(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_to_cja_panel(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::add_to_cja_panel(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn bulk_add_to_service(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::bulk_add_to_service(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn bulk_update_status(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::bulk_update_status(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn calculate_attorney_win_rate(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::calculate_attorney_win_rate(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_ecf_privileges(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::check_ecf_privileges(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_federal_practice(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::check_federal_practice(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_good_standing(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::check_good_standing(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_party_conflicts(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::check_party_conflicts(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_party_needs_service(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::check_party_needs_service(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_party_represented(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::check_party_represented(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn clear_conflict(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::clear_conflict(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn create_attorney(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::create_attorney(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn create_conflict_check(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::create_conflict_check(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn create_party(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::create_party(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn create_service_record(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::create_service_record(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_attorney(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::delete_attorney(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_party(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::delete_party(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn end_representation(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::end_representation(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_active_pro_hac_vice(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_active_pro_hac_vice(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_active_representations(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_active_representations(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorney(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorney(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorney_by_bar_number(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorney_by_bar_number(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorney_case_count(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorney_case_count(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorney_conflicts(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorney_conflicts(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorney_metrics(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorney_metrics(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorney_win_rate(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorney_win_rate(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorneys_admitted_to_court(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorneys_admitted_to_court(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorneys_by_bar_state(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorneys_by_bar_state(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorneys_by_firm(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorneys_by_firm(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorneys_by_status(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorneys_by_status(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorneys_with_discipline(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorneys_with_discipline(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_attorneys_with_ecf(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_attorneys_with_ecf(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_case_representations(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_case_representations(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_cja_appointments(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_cja_appointments(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_cja_panel_attorneys(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_cja_panel_attorneys(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_disciplinary_history(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_disciplinary_history(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_party(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_party(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_party_lead_counsel(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_party_lead_counsel(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_pending_cja_vouchers(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_pending_cja_vouchers(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_pro_hac_vice_by_case(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_pro_hac_vice_by_case(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_representation(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_representation(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_service_by_document(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_service_by_document(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_service_by_party(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_service_by_party(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_top_attorneys(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_top_attorneys(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_unrepresented_parties(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::get_unrepresented_parties(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn list_attorneys(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::list_attorneys(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn list_parties_by_attorney(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::list_parties_by_attorney(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn list_parties_by_case(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::list_parties_by_case(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn mark_service_completed(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::mark_service_completed(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn migrate_representations(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::migrate_representations(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn remove_bar_admission(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::remove_bar_admission(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn remove_federal_admission(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::remove_federal_admission(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn remove_from_cja_panel(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::remove_from_cja_panel(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn revoke_ecf_access(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::revoke_ecf_access(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn search_attorneys(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::search_attorneys(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn substitute_attorney(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::substitute_attorney(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_attorney(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::update_attorney(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_ecf_registration(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::update_ecf_registration(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_party(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::update_party(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_party_status(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::update_party_status(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_pro_hac_vice_status(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::attorney::update_pro_hac_vice_status(req, params),
        Err(e) => json::error_response(&e),
    }
}

