//! URL-based routing wrappers for sentencing handlers
//!
//! This module provides wrapper functions that extract the district from URL parameters
//! and add it as a header before calling the original sentencing handlers.

use crate::utils::json_response as json;
use spin_sdk::http::{Params, Request, Response};
use crate::error::ApiError;

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

// Core Sentencing Management (8 endpoints)

pub fn create_sentencing(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::create_sentencing(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_sentencing(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_sentencing(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_sentencing(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::update_sentencing(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_sentencing(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::delete_sentencing(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_by_case(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_by_case(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_by_defendant(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_by_defendant(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_by_judge(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_by_judge(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_pending(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_pending(req, params),
        Err(e) => json::error_response(&e),
    }
}

// Guidelines Calculation (5 endpoints)

pub fn calculate_guidelines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::calculate_guidelines(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn calculate_criminal_history_points(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::calculate_criminal_history_points(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn calculate_offense_level(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::calculate_offense_level(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn lookup_guidelines_range(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::lookup_guidelines_range(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_safety_valve_eligible(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::check_safety_valve_eligible(req, params),
        Err(e) => json::error_response(&e),
    }
}

// Departures & Variances (4 endpoints)

pub fn get_departure_stats(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_departure_stats(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_variance_stats(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_variance_stats(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_departure(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::add_departure(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_variance(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::add_variance(req, params),
        Err(e) => json::error_response(&e),
    }
}

// Substantial Assistance & Special Conditions (3 endpoints)

pub fn get_substantial_assistance(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_substantial_assistance(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_special_condition(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::add_special_condition(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_prior_sentence(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::add_prior_sentence(req, params),
        Err(e) => json::error_response(&e),
    }
}

// Supervised Release & BOP (4 endpoints)

pub fn update_supervised_release(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::update_supervised_release(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_active_supervision(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_active_supervision(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_bop_designation(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::add_bop_designation(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_rdap_eligible(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_rdap_eligible(req, params),
        Err(e) => json::error_response(&e),
    }
}

// Statistics & Reporting (5 endpoints)

pub fn get_judge_stats(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_judge_stats(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_district_stats(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_district_stats(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_trial_penalty(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_trial_penalty(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_offense_type_stats(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::get_offense_type_stats(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_by_date_range(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_by_date_range(req, params),
        Err(e) => json::error_response(&e),
    }
}

// Upcoming & Appeals (2 endpoints)

pub fn find_upcoming(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_upcoming(req, params),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_appeal_deadlines(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::sentencing::find_appeal_deadlines(req, params),
        Err(e) => json::error_response(&e),
    }
}