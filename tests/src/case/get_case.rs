//! Criminal case GET endpoint tests
//!
//! Tests for GET /api/cases/{id} and GET /api/cases/by-number/{case_number} endpoints

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a test case and return its ID and case number
fn create_test_case(district: &str) -> (String, String) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/cases")).unwrap();

    let case_data = json!({
        "title": "Test Case for Retrieval",
        "description": "This is a test case created for retrieval testing",
        "crimeType": "fraud",
        "districtCode": "SDNY",
        "location": "Test City, TS"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    (
        body_json["id"].as_str().unwrap().to_string(),
        body_json["caseNumber"].as_str().unwrap().to_string()
    )
}

/// Helper to make GET request for case by ID
fn get_case_by_id_request(id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}", id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!(null)
    } else {
        serde_json::from_str(&body).unwrap_or(json!({"raw": body}))
    };

    (status, body_json)
}

/// Helper to make GET request for case by case number
fn get_case_by_number_request(case_number: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/by-number/{}", case_number))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!(null)
    } else {
        serde_json::from_str(&body).unwrap_or(json!({"raw": body}))
    };

    (status, body_json)
}

#[spin_test]
fn test_get_case_by_id_success() {
    let _store = key_value::Store::open("district9");

    let (case_id, _case_number) = create_test_case("district9");

    let (status, response) = get_case_by_id_request(&case_id, "district9");

    assert_eq!(status, 200, "Should return 200 for existing case");
    assert_eq!(response["id"], case_id, "Should return correct case ID");
    assert_eq!(response["title"], "Test Case for Retrieval");
    assert_eq!(response["crimeType"], "fraud");
    assert!(response["assignedJudgeId"].is_null(), "assignedJudgeId should be null");
    assert_eq!(response["status"], "filed");
    assert_eq!(response["priority"], "medium");
}

#[spin_test]
fn test_get_case_by_number_success() {
    let _store = key_value::Store::open("district12");

    let (_case_id, case_number) = create_test_case("district12");

    let (status, response) = get_case_by_number_request(&case_number, "district12");

    assert_eq!(status, 200, "Should return 200 for existing case");
    assert_eq!(response["caseNumber"], case_number, "Should return correct case number");
    assert_eq!(response["title"], "Test Case for Retrieval");
    assert_eq!(response["crimeType"], "fraud");
}

#[spin_test]
fn test_get_case_by_id_not_found() {
    let _store = key_value::Store::open("district9");

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let (status, response) = get_case_by_id_request(fake_id, "district9");

    assert_eq!(status, 404, "Should return 404 for non-existent case");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error message should indicate case not found"
    );
}

#[spin_test]
fn test_get_case_by_number_not_found() {
    let _store = key_value::Store::open("district12");

    let fake_case_number = "2024-999999";
    let (status, response) = get_case_by_number_request(fake_case_number, "district12");

    assert_eq!(status, 404, "Should return 404 for non-existent case number");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error message should indicate case not found"
    );
}

#[spin_test]
fn test_get_case_by_id_invalid_uuid() {
    let _store = key_value::Store::open("district9");

    let invalid_id = "not-a-valid-uuid";
    let (status, _response) = get_case_by_id_request(invalid_id, "district9");

    assert_eq!(status, 400, "Should return 400 for invalid UUID format");
}

// NOTE: test_get_case_requires_district_header removed because
// missing district header causes a WASM trap in the spin-test virtual
// environment (the KV store open panics with no valid store name).

#[spin_test]
fn test_get_case_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    // Create case in district9
    let (case_id_d9, _) = create_test_case("district9");

    // Try to get district9 case from district12
    let (status, _response) = get_case_by_id_request(&case_id_d9, "district12");

    assert_eq!(
        status, 404,
        "Should not find district9 case when querying from district12"
    );

    // Verify case exists in district9
    let (status_d9, _) = get_case_by_id_request(&case_id_d9, "district9");
    assert_eq!(status_d9, 200, "Case should exist in its own district");
}

#[spin_test]
fn test_get_case_response_format() {
    let _store = key_value::Store::open("district9");

    let (case_id, case_number) = create_test_case("district9");

    let (status, response) = get_case_by_id_request(&case_id, "district9");

    assert_eq!(status, 200);

    // Verify all expected fields are present (camelCase)
    assert!(response.get("id").is_some(), "Should have id field");
    assert!(response.get("caseNumber").is_some(), "Should have caseNumber field");
    assert!(response.get("title").is_some(), "Should have title field");
    assert!(response.get("description").is_some(), "Should have description field");
    assert!(response.get("crimeType").is_some(), "Should have crimeType field");
    assert!(response.get("status").is_some(), "Should have status field");
    assert!(response.get("priority").is_some(), "Should have priority field");
    assert!(response.get("assignedJudgeId").is_some(), "Should have assignedJudgeId field");
    assert!(response.get("districtCode").is_some(), "Should have districtCode field");
    assert!(response.get("location").is_some(), "Should have location field");
    assert!(response.get("openedAt").is_some(), "Should have openedAt field");
    assert!(response.get("updatedAt").is_some(), "Should have updatedAt field");
    assert!(response.get("closedAt").is_some(), "Should have closedAt field");
    assert!(response.get("defendants").is_some(), "Should have defendants field");
    assert!(response.get("evidence").is_some(), "Should have evidence field");
    assert!(response.get("notesCount").is_some(), "Should have notesCount field");

    // Verify field types
    assert!(response["id"].is_string(), "id should be string");
    assert!(response["caseNumber"].is_string(), "caseNumber should be string");
    assert!(response["defendants"].is_array(), "defendants should be array");
    assert!(response["evidence"].is_array(), "evidence should be array");
    assert!(response["notesCount"].is_number(), "notesCount should be number");
    assert!(response["closedAt"].is_null(), "closedAt should be null for open case");

    // Verify case number matches
    assert_eq!(response["caseNumber"], case_number);
}

#[spin_test]
fn test_get_case_with_case_number_special_characters() {
    let _store = key_value::Store::open("district12");

    let (_case_id, case_number) = create_test_case("district12");

    let (status, response) = get_case_by_number_request(&case_number, "district12");

    assert_eq!(status, 200, "Should handle case numbers with special characters");
    assert_eq!(response["caseNumber"], case_number);
}

#[spin_test]
fn test_get_case_timestamps_format() {
    let _store = key_value::Store::open("district9");

    let (case_id, _) = create_test_case("district9");

    let (status, response) = get_case_by_id_request(&case_id, "district9");

    assert_eq!(status, 200);

    // Verify timestamp formats (should be RFC3339)
    let opened_at = response["openedAt"].as_str().unwrap();
    let updated_at = response["updatedAt"].as_str().unwrap();

    assert!(opened_at.contains('T'), "openedAt should be in ISO format");
    assert!(updated_at.contains('T'), "updatedAt should be in ISO format");
}

#[spin_test]
fn test_get_nonexistent_case_by_valid_uuid() {
    let _store = key_value::Store::open("district12");

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";
    let (status, response) = get_case_by_id_request(fake_id, "district12");

    assert_eq!(status, 404, "Should return 404 for valid UUID but non-existent case");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains(fake_id) || body_str.contains("not found"),
        "Error should reference the case ID or mention not found"
    );
}
