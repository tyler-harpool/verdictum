//! Attorney CREATE endpoint tests
//!
//! Tests for POST /api/attorneys endpoint as documented in Utoipa

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to make POST request to create attorney
fn create_attorney_request(attorney_data: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    // Set body
    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    // Perform request
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
fn test_create_attorney_success() {
    // Open the test district store
    let _store = key_value::Store::open("district9");

    let attorney_data = json!({
        "bar_number": "CA123456",
        "first_name": "John",
        "last_name": "Doe",
        "email": "john.doe@law.com",
        "phone": "415-555-0100",
        "address": {
            "street1": "123 Market St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, response) = create_attorney_request(attorney_data, "district9");

    // Should return 200 or 201 for successful creation
    assert!(
        status == 200 || status == 201,
        "Create attorney should return 200 or 201, got {}", status
    );

    // Response should contain attorney ID
    assert!(
        response.get("id").is_some(),
        "Response should contain attorney ID"
    );

    // Verify required fields are present
    assert_eq!(response["bar_number"], "CA123456");
    assert_eq!(response["first_name"], "John");
    assert_eq!(response["last_name"], "Doe");
}

#[spin_test]
fn test_create_attorney_with_optional_fields() {
    let _store = key_value::Store::open("district9");

    let attorney_data = json!({
        "bar_number": "CA789012",
        "first_name": "Jane",
        "last_name": "Smith",
        "middle_name": "Marie",
        "firm_name": "Smith & Associates",
        "email": "jane.smith@smithlaw.com",
        "phone": "415-555-0200",
        "fax": "415-555-0201",
        "address": {
            "street1": "456 Mission St",
            "street2": "Suite 500",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, response) = create_attorney_request(attorney_data, "district9");

    assert!(
        status == 200 || status == 201,
        "Create attorney with optional fields should succeed, got {}", status
    );

    // Verify optional fields were saved
    assert_eq!(response["middle_name"], "Marie");
    assert_eq!(response["firm_name"], "Smith & Associates");
    assert_eq!(response["fax"], "415-555-0201");
}

#[spin_test]
fn test_create_attorney_missing_required_field() {
    let _store = key_value::Store::open("district9");

    // Missing bar_number
    let attorney_data = json!({
        "first_name": "Invalid",
        "last_name": "Attorney",
        "email": "invalid@law.com",
        "phone": "415-555-9999",
        "address": {
            "street1": "999 Error St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, response) = create_attorney_request(attorney_data, "district9");

    assert_eq!(
        status, 400,
        "Should return 400 for missing required field"
    );

    // Error message should mention missing field
    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("bar_number") || body_str.contains("required"),
        "Error should mention missing bar_number field"
    );
}

#[spin_test]
fn test_create_attorney_invalid_email() {
    let _store = key_value::Store::open("district9");

    let attorney_data = json!({
        "bar_number": "CA345678",
        "first_name": "Test",
        "last_name": "Invalid",
        "email": "not-an-email",  // Invalid email format
        "phone": "415-555-0300",
        "address": {
            "street1": "789 Test St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, _response) = create_attorney_request(attorney_data, "district9");

    // Should validate email format
    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid email format, got {}", status
    );
}

#[spin_test]
fn test_create_attorney_duplicate_bar_number() {
    let _store = key_value::Store::open("district9");

    let attorney_data = json!({
        "bar_number": "CA999999",
        "first_name": "First",
        "last_name": "Attorney",
        "email": "first@law.com",
        "phone": "415-555-1111",
        "address": {
            "street1": "111 First St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    // Create first attorney
    let (status1, _) = create_attorney_request(attorney_data.clone(), "district9");
    assert!(status1 == 200 || status1 == 201, "First attorney should be created");

    // Try to create duplicate
    let duplicate_data = json!({
        "bar_number": "CA999999",  // Same bar number
        "first_name": "Second",
        "last_name": "Attorney",
        "email": "second@law.com",
        "phone": "415-555-2222",
        "address": {
            "street1": "222 Second St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status2, response2) = create_attorney_request(duplicate_data, "district9");

    // Debug: print the response if it's not the expected status
    if status2 != 409 && status2 != 400 {
        eprintln!("Unexpected status: {}. Response: {:?}", status2, response2);
    }

    assert!(
        status2 == 409 || status2 == 400,
        "Should return 409 Conflict or 400 for duplicate bar number, got {}", status2
    );
}

#[spin_test]
fn test_create_attorney_requires_district_header() {
    // Create request WITHOUT district header
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let attorney_data = json!({
        "bar_number": "CA777777",
        "first_name": "No",
        "last_name": "District",
        "email": "nodistrict@law.com",
        "phone": "415-555-7777",
        "address": {
            "street1": "777 No District St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}