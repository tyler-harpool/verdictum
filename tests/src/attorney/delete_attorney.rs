//! Attorney DELETE endpoint tests
//!
//! Tests for DELETE /api/attorneys/{id} endpoint

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create an attorney and return its ID
fn create_test_attorney(district: &str) -> String {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let unique_bar = format!("DEL{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let attorney_data = json!({
        "bar_number": unique_bar,
        "first_name": "Delete",
        "last_name": "Test",
        "email": "delete@law.com",
        "phone": "555-0100",
        "address": {
            "street1": "123 Delete St",
            "city": "Delete City",
            "state": "DC",
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

/// Helper to make DELETE request for attorney
fn delete_attorney_request(id: &str, district: &str) -> (u16, String) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}", id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    (status, body)
}

/// Helper to check if attorney exists
fn attorney_exists(id: &str, district: &str) -> bool {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}", id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    response.status() == 200
}

#[spin_test]
fn test_delete_attorney_success() {
    let _store = key_value::Store::open("district9");

    // Create an attorney to delete
    let attorney_id = create_test_attorney("district9");

    // Verify attorney exists
    assert!(attorney_exists(&attorney_id, "district9"), "Attorney should exist before deletion");

    // Delete the attorney
    let (status, _body) = delete_attorney_request(&attorney_id, "district9");

    assert_eq!(status, 204, "Delete should return 204 No Content");

    // Verify attorney no longer exists
    assert!(!attorney_exists(&attorney_id, "district9"), "Attorney should not exist after deletion");
}

#[spin_test]
fn test_delete_attorney_not_found() {
    let _store = key_value::Store::open("district9");

    // Try to delete non-existent attorney
    let fake_id = format!("fake-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let (status, body) = delete_attorney_request(&fake_id, "district9");

    assert_eq!(status, 404, "Should return 404 for non-existent attorney");
    assert!(
        body.contains("not found") || body.contains("NotFound"),
        "Error message should indicate attorney not found"
    );
}

#[spin_test]
fn test_delete_attorney_requires_district_header() {
    // Try to delete without district header
    let headers = Headers::new();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some("/api/attorneys/some-id")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_delete_attorney_idempotent() {
    let _store = key_value::Store::open("district9");

    // Create an attorney to delete
    let attorney_id = create_test_attorney("district9");

    // Delete the attorney first time
    let (status1, _) = delete_attorney_request(&attorney_id, "district9");
    assert_eq!(status1, 204, "First delete should return 204");

    // Try to delete the same attorney again (idempotent)
    let (status2, _) = delete_attorney_request(&attorney_id, "district9");
    assert_eq!(status2, 404, "Second delete should return 404 as attorney no longer exists");
}

#[spin_test]
fn test_delete_attorney_with_active_cases() {
    let _store = key_value::Store::open("district9");

    // Create an attorney
    let attorney_id = create_test_attorney("district9");

    // In a real system, we would associate cases with the attorney here
    // For now, we'll just test that deletion works

    let (status, _) = delete_attorney_request(&attorney_id, "district9");

    // In a real system with referential integrity, this might return 409 Conflict
    // if attorney has active cases. For now, it should succeed.
    assert_eq!(status, 204, "Delete should return 204");

    // Future enhancement: Add check for active cases and return 409 if they exist
}