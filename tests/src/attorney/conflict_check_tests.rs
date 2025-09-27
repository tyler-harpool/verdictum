//! Comprehensive attorney conflict checking tests
//!
//! Tests cover various conflict scenarios to ensure attorneys can properly
//! identify potential conflicts of interest before taking on new cases.

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a conflict check request
fn create_conflict_check_request(
    attorney_id: &str,
    parties_to_check: Vec<&str>,
    adverse_parties: Vec<&str>,
    matter_description: &str,
    district: &str,
) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}/conflict-check", attorney_id))).unwrap();

    let request_data = json!({
        "parties_to_check": parties_to_check,
        "adverse_parties": adverse_parties,
        "matter_description": matter_description,
        "case_id": null,
        "jurisdiction": null
    });

    // Set body
    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&request_data).unwrap().as_bytes()).unwrap();
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

/// Helper to create a test attorney first
fn create_test_attorney(district: &str) -> Value {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let attorney_data = json!({
        "bar_number": "TEST123456",
        "first_name": "Test",
        "last_name": "Attorney",
        "email": "test.attorney@law.com",
        "phone": "555-0100",
        "address": {
            "street1": "123 Legal St",
            "city": "Test City",
            "state": "CA",
            "zip_code": "90210",
            "country": "US"
        }
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap_or_default();
    serde_json::from_str(&body).unwrap_or(json!({}))
}

/// Test conflict check with no conflicts found
#[spin_test]
fn test_conflict_check_no_conflicts() {
    let _store = key_value::Store::open("district9");

    // Create a test attorney first
    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    // Test conflict check with parties that won't trigger conflicts
    let (status, response) = create_conflict_check_request(
        attorney_id,
        vec!["John Doe", "Jane Smith"],
        vec!["ABC Corporation"],
        "Personal injury lawsuit against ABC Corporation",
        "SDNY",
    );

    assert_eq!(status, 200);
    assert_eq!(response["has_conflicts"], false);
    assert_eq!(response["recommendation"], "Proceed");
    assert!(response["conflicts"].as_array().unwrap().is_empty());
}

/// Test conflict check with direct representation conflict
#[spin_test]
fn test_conflict_check_direct_representation() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    // Test with a party name that triggers direct representation conflict
    let (status, response) = create_conflict_check_request(
        attorney_id,
        vec!["current_client_john"], // This should trigger a conflict
        vec!["Opposing Party"],
        "New case against opposing party",
        "SDNY",
    );

    assert_eq!(status, 200);
    assert_eq!(response["has_conflicts"], true);
    assert_eq!(response["recommendation"], "MustDecline");

    let conflicts = response["conflicts"].as_array().unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0]["conflict_type"], "DirectRepresentation");
    assert_eq!(conflicts[0]["conflicted_party"], "current_client_john");
    assert_eq!(conflicts[0]["severity"], "Critical");
    assert_eq!(conflicts[0]["waivable"], false);
}

/// Test conflict check with former client conflict
#[spin_test]
fn test_conflict_check_former_client() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    let (status, response) = create_conflict_check_request(
        attorney_id,
        vec!["former_client_jane"], // This should trigger a former client conflict
        vec!["New Adverse Party"],
        "Matter substantially related to previous representation",
        "SDNY",
    );

    assert_eq!(status, 200);
    assert_eq!(response["has_conflicts"], true);
    assert_eq!(response["recommendation"], "RequireWaivers");

    let conflicts = response["conflicts"].as_array().unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0]["conflict_type"], "FormerClient");
    assert_eq!(conflicts[0]["conflicted_party"], "former_client_jane");
    assert_eq!(conflicts[0]["severity"], "High");
    assert_eq!(conflicts[0]["waivable"], true);
}

/// Test conflict check with co-defendant conflict
#[spin_test]
fn test_conflict_check_codefendant() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    let (status, response) = create_conflict_check_request(
        attorney_id,
        vec!["codefendant_mike"], // This should trigger a co-defendant conflict
        vec!["Government"],
        "Criminal defense case with multiple defendants",
        "SDNY",
    );

    assert_eq!(status, 200);
    assert_eq!(response["has_conflicts"], true);
    assert_eq!(response["recommendation"], "ProceedWithCaution");

    let conflicts = response["conflicts"].as_array().unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0]["conflict_type"], "CoDefendant");
    assert_eq!(conflicts[0]["conflicted_party"], "codefendant_mike");
    assert_eq!(conflicts[0]["severity"], "Medium");
    assert_eq!(conflicts[0]["waivable"], true);
}

/// Test conflict check with multiple conflicts
#[spin_test]
fn test_conflict_check_multiple_conflicts() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    let (status, response) = create_conflict_check_request(
        attorney_id,
        vec![
            "current_client_active",    // Direct representation conflict
            "former_client_previous",   // Former client conflict
            "codefendant_related",      // Co-defendant conflict
        ],
        vec!["Government"],
        "Complex case with multiple potential conflicts",
        "SDNY",
    );

    assert_eq!(status, 200);
    assert_eq!(response["has_conflicts"], true);
    assert_eq!(response["recommendation"], "MustDecline");

    let conflicts = response["conflicts"].as_array().unwrap();
    assert_eq!(conflicts.len(), 3); // Should detect all 3 conflicts
}

/// Test conflict check with attorney not found
#[spin_test]
fn test_conflict_check_attorney_not_found() {
    let _store = key_value::Store::open("district9");

    let (status, _response) = create_conflict_check_request(
        "nonexistent-attorney-id",
        vec!["John Doe"],
        vec!["Jane Smith"],
        "Test matter",
        "SDNY",
    );

    assert_eq!(status, 404);
}

/// Test conflict check with empty parties
#[spin_test]
fn test_conflict_check_empty_parties() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    let (status, _response) = create_conflict_check_request(
        attorney_id,
        vec![], // Empty parties
        vec![], // Empty adverse parties
        "Matter with no parties specified",
        "SDNY",
    );

    assert_eq!(status, 400);
}

/// Test conflict check with malformed JSON
#[spin_test]
fn test_conflict_check_malformed_json() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"SDNY").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}/conflict-check", attorney_id))).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(b"invalid json").unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 400);
}

/// Test conflict check with case ID and jurisdiction
#[spin_test]
fn test_conflict_check_with_case_and_jurisdiction() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"SDNY").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}/conflict-check", attorney_id))).unwrap();

    let request_data = json!({
        "parties_to_check": ["Client A"],
        "adverse_parties": ["Client B"],
        "matter_description": "Contract dispute",
        "case_id": "case-123",
        "jurisdiction": "SDNY"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&request_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();
    let body_json: Value = serde_json::from_str(&body).unwrap_or(json!({}));

    assert_eq!(status, 200);
    assert_eq!(body_json["has_conflicts"], false);
    assert_eq!(body_json["recommendation"], "Proceed");
}

/// Test that conflict check results include proper metadata
#[spin_test]
fn test_conflict_check_result_metadata() {
    let _store = key_value::Store::open("district9");

    let attorney = create_test_attorney("SDNY");
    let attorney_id = attorney["id"].as_str().unwrap_or("test-attorney-id");

    let (status, response) = create_conflict_check_request(
        attorney_id,
        vec!["Test Party"],
        vec!["Adverse Party"],
        "Test matter description",
        "SDNY",
    );

    assert_eq!(status, 200);

    // Verify result has proper metadata
    assert!(!response["check_id"].as_str().unwrap_or("").is_empty());
    assert_eq!(response["attorney_id"], attorney_id);
    assert!(!response["check_date"].as_str().unwrap_or("").is_empty());
}