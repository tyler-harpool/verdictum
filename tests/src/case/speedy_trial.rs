//! Speedy Trial Act clock tests
//!
//! Tests for starting the speedy trial clock, adding excludable delays,
//! and checking the status.

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
        "title": "Speedy Trial Test Case",
        "description": "Case for speedy trial testing",
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

/// Helper to start speedy trial clock
fn start_speedy_trial(case_id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/speedy-trial/start", case_id))).unwrap();

    let data = json!({
        "arrestDate": "2026-01-15T00:00:00Z",
        "indictmentDate": "2026-01-20T00:00:00Z",
        "arraignmentDate": "2026-01-25T00:00:00Z"
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

/// Helper to add excludable delay
fn add_excludable_delay(case_id: &str, days: i64, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/speedy-trial/exclude", case_id))).unwrap();

    let data = json!({
        "startDate": "2026-02-01T00:00:00Z",
        "endDate": "2026-02-15T00:00:00Z",
        "reason": "pretrial_motions",
        "statutoryReference": "18 U.S.C. § 3161(h)(1)(D)",
        "daysExcluded": days
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

/// Helper to get speedy trial status
fn get_speedy_trial(case_id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/speedy-trial", case_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap_or_default()).unwrap_or(json!(null));
    (status, body)
}

#[spin_test]
fn test_start_speedy_trial_success() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, response) = start_speedy_trial(&case_id, "district9");

    assert_eq!(status, 200);
    assert!(response.get("speedyTrialStatus").is_some(), "Should have speedyTrialStatus");
    let st = &response["speedyTrialStatus"];
    assert_eq!(st["daysRemaining"], 70);
    assert!(st["trialStartDeadline"].is_string());
}

#[spin_test]
fn test_start_speedy_trial_already_initialized() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    start_speedy_trial(&case_id, "district12");
    let (status, _) = start_speedy_trial(&case_id, "district12");

    assert_eq!(status, 400, "Should return 400 for already initialized clock");
}

#[spin_test]
fn test_get_speedy_trial_status() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    start_speedy_trial(&case_id, "district9");

    let (status, response) = get_speedy_trial(&case_id, "district9");

    assert_eq!(status, 200);
    assert!(response.get("daysRemaining").is_some());
    assert!(response.get("daysElapsed").is_some());
    assert!(response.get("trialStartDeadline").is_some());
}

#[spin_test]
fn test_get_speedy_trial_not_initialized() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (status, _) = get_speedy_trial(&case_id, "district12");
    assert_eq!(status, 404, "Should return 404 when clock not initialized");
}

#[spin_test]
fn test_add_excludable_delay_success() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    start_speedy_trial(&case_id, "district9");
    let (status, response) = add_excludable_delay(&case_id, 14, "district9");

    assert_eq!(status, 200);
    let st = &response["speedyTrialStatus"];
    assert_eq!(st["excludableDelaysCount"], 1);
}

#[spin_test]
fn test_add_excludable_delay_no_clock() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (status, _) = add_excludable_delay(&case_id, 10, "district12");
    assert_eq!(status, 400, "Should return 400 when clock not started");
}

#[spin_test]
fn test_speedy_trial_in_case_response() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (_, response) = start_speedy_trial(&case_id, "district9");

    assert!(response.get("speedyTrialStatus").is_some(), "CaseResponse should include speedyTrialStatus");
    let st = &response["speedyTrialStatus"];
    assert!(st.get("daysRemaining").is_some());
    assert!(st.get("trialStartDeadline").is_some());
}

#[spin_test]
fn test_speedy_trial_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    let case_id = create_test_case("district9");

    let (status, _) = start_speedy_trial(&case_id, "district12");
    assert_eq!(status, 404, "Should not find district9 case from district12");
}
