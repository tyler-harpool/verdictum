//! URL-based order management handlers that wrap the header-based handlers
//!
//! This module provides URL-based routing for judicial order endpoints
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
// Order Management (14 endpoints)
// ============================================================================

pub fn create_order(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::create_order(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn list_orders(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::list_orders(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_order(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::get_order(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_order(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::update_order(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_order(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::delete_order(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn sign_order(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::sign_order(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn issue_order(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::issue_order(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_service_record(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::add_service_record(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_orders_by_case(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::get_orders_by_case(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_orders_by_judge(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::get_orders_by_judge(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_pending_signatures(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::get_pending_signatures(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_expiring_orders(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::get_expiring_orders(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_order_statistics(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::get_order_statistics(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn create_from_template(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::create_from_template(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// ============================================================================
// Order Template Management (7 endpoints)
// ============================================================================

pub fn create_template(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::create_template(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn list_templates(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::list_templates(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn find_active_templates(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::find_active_templates(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_template(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::get_template(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_template(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::update_template(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_template(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::delete_template(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn generate_template_content(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::generate_template_content(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// ============================================================================
// Order Status Checks (2 endpoints)
// ============================================================================

pub fn check_order_expired(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::check_order_expired(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn check_requires_attention(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::order::check_requires_attention(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}