//! Attorney GET endpoint tests
//!
//! Tests for GET /api/attorneys/{id} endpoint

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

    // Use a unique bar number to avoid conflicts
    let unique_bar = format!("TEST{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let attorney_data = json!({
        "bar_number": unique_bar,
        "first_name": "Test",
        "last_name": "Attorney",
        "email": "test@law.com",
        "phone": "555-0100",
        "address": {
            "street1": "123 Test St",
            "city": "Test City",
            "state": "TS",
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

/// Helper to make GET request for attorney by ID
fn get_attorney_request(id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}", id))).unwrap();

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
fn test_get_attorney_by_id_success() {
    let _store = key_value::Store::open("district9");

    // First create an attorney
    let attorney_id = create_test_attorney("district9");

    // Now retrieve it by ID
    let (status, response) = get_attorney_request(&attorney_id, "district9");

    assert_eq!(status, 200, "Should return 200 for existing attorney");
    assert_eq!(response["id"], attorney_id, "Should return correct attorney ID");
    assert_eq!(response["first_name"], "Test");
    assert_eq!(response["last_name"], "Attorney");
}

#[spin_test]
fn test_get_attorney_by_id_not_found() {
    let _store = key_value::Store::open("district9");

    // Try to get a non-existent attorney
    let fake_id = format!("fake-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
    let (status, response) = get_attorney_request(&fake_id, "district9");

    assert_eq!(status, 404, "Should return 404 for non-existent attorney");

    // Check error message contains relevant info
    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error message should indicate attorney not found"
    );
}

#[spin_test]
fn test_get_attorney_requires_district_header() {
    // Create request WITHOUT district header
    let headers = Headers::new();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/attorneys/some-id")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_get_attorney_with_all_fields() {
    let _store = key_value::Store::open("district9");

    // Create an attorney with all optional fields
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let unique_bar = format!("FULL{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let attorney_data = json!({
        "bar_number": unique_bar,
        "first_name": "John",
        "middle_name": "Q",
        "last_name": "Lawyer",
        "firm_name": "Test Law Firm",
        "email": "john@lawfirm.com",
        "phone": "555-1234",
        "fax": "555-5678",
        "address": {
            "street1": "100 Legal Blvd",
            "street2": "Suite 200",
            "city": "Law City",
            "state": "LC",
            "zip_code": "54321",
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
    let created_attorney: Value = serde_json::from_str(&body).unwrap();
    let attorney_id = created_attorney["id"].as_str().unwrap();

    // Now retrieve and verify all fields
    let (status, response) = get_attorney_request(attorney_id, "district9");

    assert_eq!(status, 200);
    assert_eq!(response["middle_name"], "Q");
    assert_eq!(response["firm_name"], "Test Law Firm");
    assert_eq!(response["fax"], "555-5678");
    assert_eq!(response["address"]["street2"], "Suite 200");
}