//! Evidence chain of custody tests
//!
//! Tests for the enhanced evidence system with types and custody tracking.

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
        "title": "Evidence Test Case",
        "description": "Case for evidence testing",
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

/// Helper to add evidence
fn add_evidence(case_id: &str, description: &str, evidence_type: Option<&str>, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/evidence", case_id))).unwrap();

    let mut data = json!({ "description": description });
    if let Some(et) = evidence_type {
        data["evidenceType"] = json!(et);
    }

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

/// Helper to add a custody transfer
fn add_custody_transfer(case_id: &str, evidence_id: &str, from: &str, to: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/evidence/{}/custody", case_id, evidence_id))).unwrap();

    let data = json!({
        "transferredFrom": from,
        "transferredTo": to,
        "location": "Evidence Room",
        "condition": "good"
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
fn test_add_evidence_returns_object() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, response) = add_evidence(&case_id, "Fingerprints", None, "district9");

    assert_eq!(status, 200);
    let evidence = &response["evidence"][0];
    assert!(evidence.get("id").is_some(), "Evidence should have an id");
    assert_eq!(evidence["description"], "Fingerprints");
    assert!(evidence.get("evidenceType").is_some(), "Evidence should have evidenceType");
}

#[spin_test]
fn test_add_evidence_with_type() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (status, response) = add_evidence(&case_id, "Hard drive", Some("digital"), "district12");

    assert_eq!(status, 200);
    let evidence = &response["evidence"][0];
    assert_eq!(evidence["description"], "Hard drive");
    assert_eq!(evidence["evidenceType"], "digital");
}

#[spin_test]
fn test_add_evidence_default_type() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, response) = add_evidence(&case_id, "Unknown item", None, "district9");

    assert_eq!(status, 200);
    let evidence = &response["evidence"][0];
    assert!(evidence.get("evidenceType").is_some(), "Should have a default evidence type");
}

#[spin_test]
fn test_add_custody_transfer_success() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    // Add evidence first
    let (_, response) = add_evidence(&case_id, "Test item", Some("physical"), "district12");
    let evidence_id = response["evidence"][0]["id"].as_str().unwrap();

    // Add custody transfer
    let (status, response) = add_custody_transfer(&case_id, evidence_id, "FBI Agent", "Court Clerk", "district12");

    assert_eq!(status, 200);
    assert_eq!(response["evidence"][0]["custodyTransfersCount"], 1);
}

#[spin_test]
fn test_add_custody_transfer_nonexistent_evidence() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let fake_evidence_id = "550e8400-e29b-41d4-a716-446655440000";
    let (status, _) = add_custody_transfer(&case_id, fake_evidence_id, "A", "B", "district9");

    assert_eq!(status, 404, "Should return 404 for nonexistent evidence");
}

#[spin_test]
fn test_add_multiple_custody_transfers() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (_, response) = add_evidence(&case_id, "Weapon", Some("physical"), "district12");
    let evidence_id = response["evidence"][0]["id"].as_str().unwrap();

    add_custody_transfer(&case_id, evidence_id, "Crime Scene", "Lab", "district12");
    let (status, response) = add_custody_transfer(&case_id, evidence_id, "Lab", "Evidence Room", "district12");

    assert_eq!(status, 200);
    assert_eq!(response["evidence"][0]["custodyTransfersCount"], 2);
}

#[spin_test]
fn test_evidence_count_in_case_response() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    add_evidence(&case_id, "Item 1", None, "district9");
    let (_, response) = add_evidence(&case_id, "Item 2", None, "district9");

    assert!(response.get("evidenceCount").is_some(), "Should have evidenceCount field");
    assert_eq!(response["evidenceCount"], 2);
}

#[spin_test]
fn test_add_evidence_object_empty_description() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (status, _) = add_evidence(&case_id, "", None, "district12");
    assert_eq!(status, 400, "Should return 400 for empty description");
}

#[spin_test]
fn test_custody_transfer_empty_fields() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (_, response) = add_evidence(&case_id, "Test", None, "district9");
    let evidence_id = response["evidence"][0]["id"].as_str().unwrap();

    let (status, _) = add_custody_transfer(&case_id, evidence_id, "", "B", "district9");
    assert_eq!(status, 400, "Should return 400 for empty transferred_from");
}

#[spin_test]
fn test_evidence_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    let case_id = create_test_case("district9");

    let (status, _) = add_evidence(&case_id, "Cross-district", None, "district12");
    assert_eq!(status, 404, "Should not find district9 case from district12");
}
