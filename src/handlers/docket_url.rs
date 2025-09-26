//! URL-based docket management handlers that wrap the header-based handlers
//!
//! This module provides URL-based routing for docket, calendar, and speedy trial endpoints
//! by extracting the district from the URL path and adding it as a header.

use crate::error::ApiError;
use spin_sdk::http::{IntoResponse, Params, Request, Response};
use crate::utils::json_response as json;

/// Helper function to add district header from URL parameter
fn add_district_header(req: Request, params: &Params) -> Result<Request, ApiError> {
    let district = params.get("district")
        .ok_or_else(|| ApiError::BadRequest("District parameter is required in URL".to_string()))?;

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
// Docket Entry Management (10 endpoints)
// ============================================================================

pub fn create_docket_entry(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::create_docket_entry(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_case_docket(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_case_docket(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_docket_entry(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_docket_entry(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_attachment(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::add_attachment(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn search_docket(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::search_docket(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn generate_docket_sheet(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::generate_docket_sheet(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_entries_by_type(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_entries_by_type(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_sealed_entries(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_sealed_entries(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn search_entries(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::search_entries(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_entry(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::delete_entry(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_filing_statistics(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_filing_statistics(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_immediate_service(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::check_immediate_service(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// ============================================================================
// Calendar Management (9 endpoints)
// ============================================================================

pub fn schedule_event(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::schedule_event(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_case_calendar(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_case_calendar(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_judge_schedule(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_judge_schedule(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_event_status(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::update_event_status(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_available_slot(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::find_available_slot(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_courtroom_utilization(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_courtroom_utilization(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_events_by_courtroom(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_events_by_courtroom(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_event(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::delete_event(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn search_calendar(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::search_calendar(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// ============================================================================
// Speedy Trial Management (8 endpoints)
// ============================================================================

pub fn init_speedy_trial(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::init_speedy_trial(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_speedy_trial(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_speedy_trial(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_excludable_delay(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::add_excludable_delay(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_approaching_deadlines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_approaching_deadlines(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_violations(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::get_violations(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_clock(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::update_clock(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_deadline_approaching(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::docket::check_deadline_approaching(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}