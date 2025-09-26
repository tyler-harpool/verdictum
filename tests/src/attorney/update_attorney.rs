//! Attorney UPDATE endpoint tests
//!
//! Tests for PUT /api/attorneys/{id} endpoint

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create an attorney and return the full attorney object
fn create_test_attorney(district: &str) -> Value {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let unique_bar = format!("UPDATE{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let attorney_data = json!({
        "bar_number": unique_bar,
        "first_name": "Original",
        "last_name": "Name",
        "email": "original@law.com",
        "phone": "555-0001",
        "address": {
            "street1": "100 Original St",
            "city": "Original City",
            "state": "OC",
            "zip_code": "11111",
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
    serde_json::from_str(&body).unwrap()
}

/// Helper to make PUT request to update attorney
fn update_attorney_request(id: &str, updates: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Put).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}", id))).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&updates).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

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
fn test_update_attorney_success() {
    let _store = key_value::Store::open("district9");

    // Create an attorney first
    let attorney = create_test_attorney("district9");
    let attorney_id = attorney["id"].as_str().unwrap();

    // Update some fields - only include fields we want to change for partial update
    let updates = json!({
        "first_name": "Updated",
        "last_name": "Lawyer",
        "email": "updated@law.com",
        "phone": "555-9999",
        "address": {
            "street1": "999 Updated Ave",
            "city": "New City",
            "state": "NC",
            "zip_code": "99999",
            "country": "USA"
        }
    });

    let (status, response) = update_attorney_request(attorney_id, updates, "district9");

    // Debug output
    if status != 200 {
        eprintln!("Update failed with status {}. Response: {:?}", status, response);
    }

    assert_eq!(status, 200, "Update should return 200");
    assert_eq!(response["id"], attorney_id);
    assert_eq!(response["first_name"], "Updated");
    assert_eq!(response["last_name"], "Lawyer");
    assert_eq!(response["email"], "updated@law.com");
    assert_eq!(response["address"]["city"], "New City");
}

#[spin_test]
fn test_update_attorney_add_optional_fields() {
    let _store = key_value::Store::open("district9");

    // Create a basic attorney
    let attorney = create_test_attorney("district9");
    let attorney_id = attorney["id"].as_str().unwrap();

    // Add optional fields through update - only include fields we want to change
    let updates = json!({
        "middle_name": "Middle",
        "firm_name": "Updated Law Firm",
        "fax": "555-1111"
    });

    let (status, response) = update_attorney_request(attorney_id, updates, "district9");

    assert_eq!(status, 200);
    assert_eq!(response["middle_name"], "Middle");
    assert_eq!(response["firm_name"], "Updated Law Firm");
    assert_eq!(response["fax"], "555-1111");
}

#[spin_test]
fn test_update_attorney_not_found() {
    let _store = key_value::Store::open("district9");

    let fake_id = format!("fake-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    // Try to update a non-existent attorney
    let updates = json!({
        "first_name": "Fake",
        "last_name": "Attorney"
    });

    let (status, response) = update_attorney_request(&fake_id, updates, "district9");

    assert_eq!(status, 404, "Should return 404 for non-existent attorney");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error should indicate attorney not found"
    );
}

#[spin_test]
fn test_update_attorney_invalid_email() {
    let _store = key_value::Store::open("district9");

    // Create an attorney first
    let attorney = create_test_attorney("district9");
    let attorney_id = attorney["id"].as_str().unwrap();

    // Try to update with invalid email
    let updates = json!({
        "email": "not-an-email"  // Invalid email format
    });

    let (status, _response) = update_attorney_request(attorney_id, updates, "district9");

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid email, got {}", status
    );
}

#[spin_test]
fn test_update_attorney_requires_district_header() {
    // Try to update without district header
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Put).unwrap();
    request.set_path_with_query(Some("/api/attorneys/some-id")).unwrap();

    let updates = json!({
        "first_name": "Test"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&updates).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_update_attorney_change_bar_number_to_duplicate() {
    let _store = key_value::Store::open("district9");

    // Create two attorneys
    let attorney1 = create_test_attorney("district9");
    let attorney2 = create_test_attorney("district9");

    let attorney2_id = attorney2["id"].as_str().unwrap();
    let attorney1_bar = attorney1["bar_number"].as_str().unwrap();

    // Try to update attorney2's bar number to attorney1's bar number
    let updates = json!({
        "bar_number": attorney1_bar  // This should conflict
    });

    let (status, _response) = update_attorney_request(attorney2_id, updates, "district9");

    assert!(
        status == 409 || status == 400,
        "Should return 409 or 400 for duplicate bar number, got {}", status
    );
}