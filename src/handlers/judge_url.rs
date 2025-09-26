//! URL-based judge management HTTP handlers
//!
//! These handlers wrap the existing judge handlers but extract tenant
//! information from the URL path instead of headers.

use spin_sdk::http::Response;
use spin_sdk::http::{IntoResponse, Params, Request};
use crate::utils::json_response;

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
// Judge Management Wrapper Functions
// ============================================================================

pub fn create_judge(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::create_judge(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_all_judges(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::get_all_judges(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_available_judges(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::get_available_judges(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_workload_stats(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::get_workload_stats(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn search_judges(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::search_judges(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_judge_by_id(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::get_judge_by_id(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn update_judge_status(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::update_judge_status(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn add_conflict(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::add_conflict(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn check_conflicts(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::check_conflicts(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

// ============================================================================
// Case Assignment Wrapper Functions
// ============================================================================

pub fn assign_case(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::assign_case(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_case_assignment(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::get_case_assignment(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

// ============================================================================
// Recusal Wrapper Functions
// ============================================================================

pub fn file_recusal(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::file_recusal(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn rule_on_recusal(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::rule_on_recusal(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_pending_recusals(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::judge::get_pending_recusals(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}