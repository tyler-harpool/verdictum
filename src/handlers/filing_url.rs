//! URL-based filing pipeline HTTP handlers
//!
//! These handlers wrap the existing filing handlers but extract tenant
//! information from the URL path instead of headers.

use spin_sdk::http::Response;
use spin_sdk::http::{IntoResponse, Params, Request};

/// Helper to create a new request with district header from URL parameter
fn add_district_header(req: Request, params: &Params) -> Result<Request, crate::error::ApiError> {
    let district = params.get("district").ok_or_else(|| {
        crate::error::ApiError::BadRequest("District parameter is required in URL".to_string())
    })?;

    let method = req.method().clone();
    let uri = req.uri().to_string();

    let headers = spin_sdk::http::Headers::new();

    for (name, value) in req.headers() {
        let _ = headers.append(&name.to_string(), &value.as_bytes().to_vec());
    }

    let _ = headers.set(
        &"x-court-district".to_string(),
        &vec![district.as_bytes().to_vec()],
    );

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
// Filing Pipeline Wrapper Functions
// ============================================================================

pub fn submit_filing(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::filing::submit_filing(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn validate_filing(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::filing::validate_filing(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}

pub fn list_jurisdictions(req: Request, params: Params) -> Response {
    let req = match add_district_header(req, &params) {
        Ok(r) => r,
        Err(e) => return crate::utils::json_response::error_response(&e),
    };
    match crate::handlers::filing::list_jurisdictions(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => crate::utils::json_response::error_response(&e),
    }
}
