//! URL-based criminal case management HTTP handlers
//!
//! These handlers wrap the existing criminal case handlers but extract tenant
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
// Wrapper functions that delegate to existing handlers
// ============================================================================

pub fn create_case(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::create_case(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_case_by_id(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::get_case_by_id(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn search_cases(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::search_cases(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_case_statistics(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::get_case_statistics(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_case_by_number(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::get_case_by_number(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn get_cases_by_judge(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::get_cases_by_judge(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn count_by_status(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::count_by_status(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn update_case_status(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::update_case_status(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn update_case_priority(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::update_case_priority(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn add_defendant(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::add_defendant(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn add_evidence(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::add_evidence(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn add_note(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::add_note(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn enter_plea(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::enter_plea(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn schedule_court_event(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::schedule_court_event(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn file_motion(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::file_motion(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn rule_on_motion(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::rule_on_motion(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn delete_case(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::criminal_case::delete_case(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}