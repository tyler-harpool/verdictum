//! Criminal case UPDATE endpoint tests
//!
//! Tests for PATCH /api/cases/{id}/status and PATCH /api/cases/{id}/priority endpoints

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a test case and return its ID
fn create_test_case(district: &str) -> String {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/cases")).unwrap();

    let case_data = json!({
        "title": "Test Case for Updates",
        "description": "This is a test case created for update testing",
        "crimeType": "financial_fraud",
        "assignedJudge": "Judge TestUpdate",
        "location": "Update City, UC"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    body_json["id"].as_str().unwrap().to_string()
}

/// Helper to update case status
fn update_case_status_request(case_id: &str, status: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/status", case_id))).unwrap();

    let update_data = json!({
        "status": status
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&update_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status_code = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!(null)
    } else {
        serde_json::from_str(&body).unwrap_or(json!({"raw": body}))
    };

    (status_code, body_json)
}

/// Helper to update case priority
fn update_case_priority_request(case_id: &str, priority: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/priority", case_id))).unwrap();

    let update_data = json!({
        "priority": priority
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&update_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status_code = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!(null)
    } else {
        serde_json::from_str(&body).unwrap_or(json!({"raw": body}))
    };

    (status_code, body_json)
}

#[spin_test]
fn test_update_case_status_to_active() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = update_case_status_request(&case_id, "active", "district9");

    assert_eq!(status, 200, "Should return 200 for successful status update");
    assert_eq!(response["id"], case_id);
    assert_eq!(response["status"], "active");

    // Updated timestamp should be changed
    assert!(response.get("updated_at").is_some(), "Should have updated_at timestamp");
}

#[spin_test]
fn test_update_case_status_to_closed() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let (status, response) = update_case_status_request(&case_id, "closed", "district12");

    assert_eq!(status, 200, "Should return 200 for successful status update");
    assert_eq!(response["status"], "closed");

    // Should have closed_at timestamp when status is closed
    assert!(response.get("closed_at").is_some(), "Should have closed_at timestamp for closed case");
    assert!(!response["closed_at"].is_null(), "closed_at should not be null for closed case");
}

#[spin_test]
fn test_update_case_status_all_valid_statuses() {
    let _store = key_value::Store::open("district9");

    let valid_statuses = vec!["open", "active", "pending", "closed", "dismissed"];

    for status_value in valid_statuses {
        let case_id = create_test_case("district9");

        let (status, response) = update_case_status_request(&case_id, status_value, "district9");

        assert_eq!(
            status, 200,
            "Should update status to {} successfully", status_value
        );
        assert_eq!(response["status"], status_value);
    }
}

#[spin_test]
fn test_update_case_status_invalid_status() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = update_case_status_request(&case_id, "invalid_status", "district9");

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid status, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("status") || body_str.contains("invalid"),
        "Error should mention invalid status"
    );
}

#[spin_test]
fn test_update_case_status_nonexistent_case() {
    let _store = key_value::Store::open("district12");

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, response) = update_case_status_request(fake_id, "active", "district12");

    assert_eq!(status, 404, "Should return 404 for non-existent case");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error should indicate case not found"
    );
}

#[spin_test]
fn test_update_case_priority_to_high() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = update_case_priority_request(&case_id, "high", "district9");

    assert_eq!(status, 200, "Should return 200 for successful priority update");
    assert_eq!(response["id"], case_id);
    assert_eq!(response["priority"], "high");
}

#[spin_test]
fn test_update_case_priority_all_valid_priorities() {
    let _store = key_value::Store::open("district12");

    let valid_priorities = vec!["low", "medium", "high", "urgent"];

    for priority_value in valid_priorities {
        let case_id = create_test_case("district12");

        let (status, response) = update_case_priority_request(&case_id, priority_value, "district12");

        assert_eq!(
            status, 200,
            "Should update priority to {} successfully", priority_value
        );
        assert_eq!(response["priority"], priority_value);
    }
}

#[spin_test]
fn test_update_case_priority_invalid_priority() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = update_case_priority_request(&case_id, "invalid_priority", "district9");

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid priority, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("priority") || body_str.contains("invalid"),
        "Error should mention invalid priority"
    );
}

#[spin_test]
fn test_update_case_priority_nonexistent_case() {
    let _store = key_value::Store::open("district12");

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, response) = update_case_priority_request(fake_id, "high", "district12");

    assert_eq!(status, 404, "Should return 404 for non-existent case");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error should indicate case not found"
    );
}

#[spin_test]
fn test_update_case_status_requires_district_header() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // Create request WITHOUT district header
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/status", case_id))).unwrap();

    let update_data = json!({"status": "active"});

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&update_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_update_case_priority_requires_district_header() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // Create request WITHOUT district header
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/priority", case_id))).unwrap();

    let update_data = json!({"priority": "high"});

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&update_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_update_case_district_isolation() {
    // Create stores for both districts
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    // Create case in district9
    let case_id = create_test_case("district9");

    // Try to update case from district12
    let (status, _response) = update_case_status_request(&case_id, "active", "district12");

    assert_eq!(
        status, 404,
        "Should not be able to update district9 case from district12"
    );

    // Verify update works in correct district
    let (status_correct, _) = update_case_status_request(&case_id, "active", "district9");
    assert_eq!(status_correct, 200, "Should update case in its own district");
}

#[spin_test]
fn test_update_case_status_invalid_uuid() {
    let _store = key_value::Store::open("district9");

    let invalid_id = "not-a-valid-uuid";

    let (status, _response) = update_case_status_request(invalid_id, "active", "district9");

    assert_eq!(status, 400, "Should return 400 for invalid UUID format");
}

#[spin_test]
fn test_update_case_priority_invalid_uuid() {
    let _store = key_value::Store::open("district12");

    let invalid_id = "not-a-valid-uuid";

    let (status, _response) = update_case_priority_request(invalid_id, "high", "district12");

    assert_eq!(status, 400, "Should return 400 for invalid UUID format");
}

#[spin_test]
fn test_update_case_status_malformed_json() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/status", case_id))).unwrap();

    // Malformed JSON
    let malformed_json = r#"{"status": "active""#;

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(malformed_json.as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 400, "Should return 400 for malformed JSON");
}

#[spin_test]
fn test_update_case_status_missing_status_field() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/status", case_id))).unwrap();

    // Missing status field
    let update_data = json!({});

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&update_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 400, "Should return 400 for missing status field");
}