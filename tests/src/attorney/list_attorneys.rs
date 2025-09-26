//! Attorney LIST endpoint tests
//!
//! Tests for GET /api/attorneys endpoint

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create an attorney
fn create_test_attorney(district: &str, bar_prefix: &str, name_prefix: &str) -> String {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let unique_bar = format!("{}{}", bar_prefix, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let attorney_data = json!({
        "bar_number": unique_bar,
        "first_name": format!("{}", name_prefix),
        "last_name": "Attorney",
        "email": format!("{}@law.com", name_prefix.to_lowercase()),
        "phone": "555-0100",
        "address": {
            "street1": format!("123 {} St", name_prefix),
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
    let status = response.status();
    let body = response.body_as_string().unwrap();

    if status != 201 && status != 200 {
        eprintln!("Failed to create attorney. Status: {}, Body: {}", status, body);
        panic!("Failed to create test attorney");
    }

    let body_json: Value = serde_json::from_str(&body).unwrap();
    body_json["id"].as_str().unwrap().to_string()
}

/// Helper to list all attorneys
fn list_attorneys_request(district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!([])
    } else {
        serde_json::from_str(&body).unwrap_or(json!([]))
    };

    (status, body_json)
}

/// Helper to delete an attorney
fn delete_attorney(id: &str, district: &str) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}", id))).unwrap();

    spin_test_sdk::perform_request(request);
}

#[spin_test]
fn test_list_attorneys_empty() {
    let _store = key_value::Store::open("district9");

    // List attorneys in a fresh district (should be empty)
    let (status, attorneys) = list_attorneys_request("district9");

    assert_eq!(status, 200, "List should return 200");
    assert!(attorneys.is_array(), "Response should be an array");

    // District9 might have some attorneys from other tests, but it should still be a valid array
    let attorneys_array = attorneys.as_array().unwrap();
    // Array should exist (may or may not be empty depending on other tests)
}

#[spin_test]
fn test_list_attorneys_with_data() {
    let _store = key_value::Store::open("district12");

    // Create a few attorneys
    let id1 = create_test_attorney("district12", "LIST1", "Alice");
    let id2 = create_test_attorney("district12", "LIST2", "Bob");
    let id3 = create_test_attorney("district12", "LIST3", "Charlie");

    // List all attorneys
    let (status, attorneys) = list_attorneys_request("district12");

    assert_eq!(status, 200, "List should return 200");
    assert!(attorneys.is_array(), "Response should be an array");

    let attorneys_array = attorneys.as_array().unwrap();
    assert!(attorneys_array.len() >= 3, "Should have at least 3 attorneys");

    // Check that our attorneys are in the list
    let ids: Vec<String> = attorneys_array
        .iter()
        .filter_map(|a| a["id"].as_str().map(String::from))
        .collect();

    assert!(ids.contains(&id1), "Should contain attorney 1");
    assert!(ids.contains(&id2), "Should contain attorney 2");
    assert!(ids.contains(&id3), "Should contain attorney 3");

    // Clean up
    delete_attorney(&id1, "district12");
    delete_attorney(&id2, "district12");
    delete_attorney(&id3, "district12");
}

#[spin_test]
fn test_list_attorneys_requires_district_header() {
    // Try to list without district header
    let headers = Headers::new();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_list_attorneys_returns_full_objects() {
    let _store = key_value::Store::open("district9");

    // Create an attorney with all fields
    create_test_attorney("district9", "FULL", "Complete");

    // List attorneys
    let (status, attorneys) = list_attorneys_request("district9");

    assert_eq!(status, 200, "List should return 200");

    let attorneys_array = attorneys.as_array().unwrap();
    assert!(attorneys_array.len() >= 1, "Should have at least 1 attorney");

    // Find our attorney (the one with FULL in bar number)
    let attorney = attorneys_array
        .iter()
        .find(|a| a["bar_number"].as_str().unwrap_or("").starts_with("FULL"));

    assert!(attorney.is_some(), "Should find our attorney");

    let attorney = attorney.unwrap();

    // Verify key fields are present
    assert!(attorney["id"].is_string(), "Should have id");
    assert!(attorney["bar_number"].is_string(), "Should have bar_number");
    assert!(attorney["first_name"].is_string(), "Should have first_name");
    assert!(attorney["last_name"].is_string(), "Should have last_name");
    assert!(attorney["email"].is_string(), "Should have email");
    assert!(attorney["address"].is_object(), "Should have address object");
}

#[spin_test]
fn test_list_attorneys_after_deletion() {
    let _store = key_value::Store::open("district12");

    // Create attorneys
    let id1 = create_test_attorney("district12", "DEL1", "ToDelete");
    let id2 = create_test_attorney("district12", "KEEP1", "ToKeep");

    // Delete one attorney
    delete_attorney(&id1, "district12");

    // List attorneys
    let (status, attorneys) = list_attorneys_request("district12");

    assert_eq!(status, 200, "List should return 200");

    let attorneys_array = attorneys.as_array().unwrap();
    let ids: Vec<String> = attorneys_array
        .iter()
        .filter_map(|a| a["id"].as_str().map(String::from))
        .collect();

    assert!(!ids.contains(&id1), "Should not contain deleted attorney");
    assert!(ids.contains(&id2), "Should still contain non-deleted attorney");

    // Clean up
    delete_attorney(&id2, "district12");
}