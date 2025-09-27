//! Attorney-Case relationship tests
//!
//! Tests for attorney case assignment and management endpoints

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a test attorney and return its ID
fn create_attorney(district: &str) -> String {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let attorney_data = json!({
        "bar_number": format!("CASE{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "first_name": "Case",
        "last_name": "Attorney",
        "email": "case.attorney@law.com",
        "phone": "555-0100",
        "address": {
            "street1": "123 Main St",
            "city": "City",
            "state": "ST",
            "zip_code": "12345",
            "country": "USA"
        }
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    body_json["id"].as_str().unwrap().to_string()
}

#[spin_test]
fn test_assign_attorney_to_case() {
    let _store = key_value::Store::open("district9");

    // Create an attorney
    let attorney_id = create_attorney("district9");

    // Assign attorney to a case
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}/cases", attorney_id))).unwrap();

    let assignment_data = json!({
        "attorney_id": attorney_id,
        "case_id": "00000000-0000-0000-0000-000000000001",
        "role": "lead_counsel",
        "notes": "Primary defense attorney"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&assignment_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap();

    assert_eq!(status, 201, "Should create assignment successfully");

    let body_json: Value = serde_json::from_str(&body).unwrap();
    assert!(body_json["id"].is_string(), "Should have assignment ID");
    assert_eq!(body_json["attorney_id"], attorney_id);
    assert_eq!(body_json["case_id"], "00000000-0000-0000-0000-000000000001");
    assert_eq!(body_json["role"], "lead_counsel");
    assert!(body_json["is_active"].as_bool().unwrap(), "Should be active");
}

#[spin_test]
fn test_get_attorney_cases() {
    let _store = key_value::Store::open("district12");

    // Create an attorney
    let attorney_id = create_attorney("district12");

    // Get attorney's cases
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}/cases", attorney_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap();

    assert_eq!(status, 200, "Should return 200");

    let body_json: Value = serde_json::from_str(&body).unwrap();
    assert!(body_json.is_array(), "Should return array of assignments");
}

#[spin_test]
fn test_remove_attorney_from_case() {
    let _store = key_value::Store::open("district9");

    // Create an attorney
    let attorney_id = create_attorney("district9");
    let case_id = "00000000-0000-0000-0000-000000000001";

    // Remove attorney from case
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}/cases/{}", attorney_id, case_id))).unwrap();

    let removal_data = json!({
        "reason": "Conflict of interest"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&removal_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();

    assert_eq!(status, 204, "Should return 204 No Content");
}

#[spin_test]
fn test_get_attorney_case_load() {
    let _store = key_value::Store::open("district12");

    // Create an attorney
    let attorney_id = create_attorney("district12");

    // Get attorney's case load
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}/case-load", attorney_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap();

    assert_eq!(status, 200, "Should return 200");

    let body_json: Value = serde_json::from_str(&body).unwrap();
    assert!(body_json["attorney_id"].is_string(), "Should have attorney_id");
    assert!(body_json["attorney_name"].is_string(), "Should have attorney_name");
    assert!(body_json["active_cases"].is_number(), "Should have active_cases count");
    assert!(body_json["completed_cases"].is_number(), "Should have completed_cases count");
    assert!(body_json["active_assignments"].is_array(), "Should have active_assignments array");
}

#[spin_test]
fn test_attorney_case_endpoints_require_district_header() {
    // Test assign endpoint
    let headers = Headers::new();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys/test-id/cases")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 400, "Assign should require district header");

    // Test get cases endpoint
    let headers2 = Headers::new();
    let request2 = OutgoingRequest::new(headers2);
    request2.set_method(&Method::Get).unwrap();
    request2.set_path_with_query(Some("/api/attorneys/test-id/cases")).unwrap();

    let response2 = spin_test_sdk::perform_request(request2);
    assert_eq!(response2.status(), 400, "Get cases should require district header");

    // Test case load endpoint
    let headers3 = Headers::new();
    let request3 = OutgoingRequest::new(headers3);
    request3.set_method(&Method::Get).unwrap();
    request3.set_path_with_query(Some("/api/attorneys/test-id/case-load")).unwrap();

    let response3 = spin_test_sdk::perform_request(request3);
    assert_eq!(response3.status(), 400, "Case load should require district header");
}

#[spin_test]
fn test_get_case_load_for_nonexistent_attorney() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/attorneys/00000000-0000-0000-0000-000000000000/case-load")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();

    assert_eq!(status, 404, "Should return 404 for non-existent attorney");
}