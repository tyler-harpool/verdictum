//! URL-based routing wrappers for opinion handlers
//!
//! This module provides wrapper functions that extract the district from URL parameters
//! and add it as a header before calling the original opinion handlers.

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

// Opinion Management - 11 endpoints

pub fn create_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::create_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn update_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::update_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn delete_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::delete_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn list_opinions(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::list_opinions(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn file_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::file_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn publish_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::publish_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_judge_vote(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::add_judge_vote(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_citation(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::add_citation(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_headnote(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::add_headnote(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn search_opinions(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::search_opinions(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// Cross-Entity Opinion Queries - 3 endpoints

pub fn get_opinions_by_case(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_opinions_by_case(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_opinions_by_author(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_opinions_by_author(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_precedential_opinions(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_precedential_opinions(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// Draft Management - 5 endpoints

pub fn create_draft(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::create_draft(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_drafts(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_drafts(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_current_draft(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_current_draft(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn add_draft_comment(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::add_draft_comment(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn resolve_draft_comment(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::resolve_draft_comment(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

// Statistics & Validation - 5 endpoints

pub fn get_opinion_statistics(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_opinion_statistics(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn get_citation_statistics(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::get_citation_statistics(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn is_majority_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::is_majority_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn is_binding_opinion(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::is_binding_opinion(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}

pub fn calculate_opinion_statistics(req: Request, params: Params) -> Response {
    match add_district_header(req, &params) {
        Ok(req) => crate::handlers::opinion::calculate_opinion_statistics(req, params)
            .map(|r| r.into_response())
            .unwrap_or_else(|e| json::error_response(&e)),
        Err(e) => json::error_response(&e),
    }
}