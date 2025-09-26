//! URL-based routing wrappers for deadline handlers
//!
//! This module provides wrapper functions that extract the district from URL parameters
//! and add it as a header before calling the original deadline handlers.

use crate::error::ApiError;
use crate::utils::json_response as json;
use spin_sdk::http::{IntoResponse, Params, Request, Response};

/// Helper function to extract district from URL params and add as header
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

// Core Deadline Management (8 endpoints)

pub fn create_deadline(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::create_deadline(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_case_deadlines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_case_deadlines(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_deadline(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_deadline(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn complete_deadline(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::complete_deadline(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_upcoming_deadlines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_upcoming_deadlines(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_urgent_deadlines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_urgent_deadlines(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn search_deadlines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::search_deadlines(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn calculate_frcp_deadlines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::calculate_frcp_deadlines(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// Extension Management (5 endpoints)

pub fn request_extension(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::request_extension(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn rule_on_extension(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::rule_on_extension(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_extension_by_id(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_extension_by_id(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_extensions_by_deadline(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_extensions_by_deadline(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_pending_extensions(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_pending_extensions(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// Compliance & Reporting (4 endpoints)

pub fn get_compliance_stats(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_compliance_stats(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn generate_compliance_report(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::generate_compliance_report(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_performance_metrics(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_performance_metrics(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_missed_jurisdictional(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_missed_jurisdictional(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// Reminder Management (5 endpoints)

pub fn get_pending_reminders(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_pending_reminders(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn send_reminders(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::send_reminders(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_reminders_by_deadline(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_reminders_by_deadline(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_reminders_by_recipient(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_reminders_by_recipient(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn acknowledge_reminder(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::acknowledge_reminder(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// Additional Deadline Operations (4 endpoints)

pub fn get_deadlines_by_type(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_deadlines_by_type(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_deadline_status(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::update_deadline_status(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_deadline(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::delete_deadline(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_federal_rules(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::deadline::get_federal_rules(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}