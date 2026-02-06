//! Sealed case handling tests
//!
//! Tests for sealing and unsealing criminal cases.

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
        "title": "Sealed Case Test",
        "description": "Case for seal testing",
        "crimeType": "fraud",
        "districtCode": "SDNY",
        "location": "New York, NY"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap()).unwrap();
    body["id"].as_str().unwrap().to_string()
}

/// Helper to seal a case
fn seal_case(case_id: &str, reason: &str, sealed_by: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/seal", case_id))).unwrap();

    let data = json!({
        "reason": reason,
        "sealedBy": sealed_by
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap_or_default()).unwrap_or(json!(null));
    (status, body)
}

/// Helper to unseal a case
fn unseal_case(case_id: &str, reason: &str, unsealed_by: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/unseal", case_id))).unwrap();

    let data = json!({
        "reason": reason,
        "unsealedBy": unsealed_by
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap_or_default()).unwrap_or(json!(null));
    (status, body)
}

#[spin_test]
fn test_seal_case_success() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, response) = seal_case(&case_id, "Ongoing investigation", "Judge Smith", "district9");

    assert_eq!(status, 200);
    assert_eq!(response["isSealed"], true);
    assert!(response["sealedDate"].is_string(), "Should have sealed date");
    assert_eq!(response["sealedBy"], "Judge Smith");
    assert_eq!(response["sealReason"], "Ongoing investigation");
}

#[spin_test]
fn test_seal_already_sealed_case() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    // Seal once
    let (status1, _) = seal_case(&case_id, "First seal", "Judge A", "district12");
    assert_eq!(status1, 200);

    // Try to seal again
    let (status2, _) = seal_case(&case_id, "Second seal", "Judge B", "district12");
    assert_eq!(status2, 400, "Should return 400 for already sealed case");
}

#[spin_test]
fn test_unseal_case_success() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    // Seal first
    seal_case(&case_id, "Investigation", "Judge Smith", "district9");

    // Then unseal
    let (status, response) = unseal_case(&case_id, "Investigation complete", "Judge Smith", "district9");

    assert_eq!(status, 200);
    assert_eq!(response["isSealed"], false);
    assert!(response["sealedDate"].is_null(), "Sealed date should be null after unseal");
}

#[spin_test]
fn test_unseal_not_sealed_case() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (status, _) = unseal_case(&case_id, "No reason", "Judge A", "district12");
    assert_eq!(status, 400, "Should return 400 for unsealing a non-sealed case");
}

#[spin_test]
fn test_unseal_adds_case_note() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    // Get initial notes count
    seal_case(&case_id, "Investigation", "Judge Smith", "district9");

    let (_, response) = unseal_case(&case_id, "Complete", "Judge Smith", "district9");

    assert!(response["notesCount"].as_u64().unwrap() >= 1, "Unseal should add a case note");
}

#[spin_test]
fn test_seal_requires_reason() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (status, _) = seal_case(&case_id, "", "Judge A", "district12");
    assert_eq!(status, 400, "Should return 400 for empty reason");
}

#[spin_test]
fn test_seal_requires_sealed_by() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, _) = seal_case(&case_id, "Valid reason", "", "district9");
    assert_eq!(status, 400, "Should return 400 for empty sealed_by");
}

#[spin_test]
fn test_seal_case_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    let case_id = create_test_case("district9");

    let (status, _) = seal_case(&case_id, "Test", "Judge A", "district12");
    assert_eq!(status, 404, "Should not find district9 case from district12");
}
