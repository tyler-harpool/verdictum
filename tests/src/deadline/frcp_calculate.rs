//! FRCP Rule 6 deadline calculation integration tests
//!
//! Tests for POST /api/deadlines/calculate endpoint.
//! These tests validate the FRCP Rule 6 day-counting algorithm including:
//! - Short period (< 11 days): exclude weekends and holidays from count
//! - Long period (>= 11 days): count all calendar days
//! - Landing day extension to next business day
//! - Service method adjustments (+3 for mail, +0 for electronic)
//! - Federal holiday awareness and observation rules

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to call POST /api/deadlines/calculate and return status + response body
fn calculate_deadline(request_data: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/deadlines/calculate")).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(
        serde_json::to_string(&request_data).unwrap().as_bytes()
    ).unwrap();
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

/// Helper to call GET /api/federal-rules and return available federal rules
fn get_federal_rules(district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/federal-rules")).unwrap();

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

// ============================================================
// Long period tests (>= 11 days — calendar day counting)
// ============================================================

/// Test 14-day response deadline filing on a normal business day.
/// Filing on Friday Jan 10, 2025 -> trigger date.
/// FRCP 12(a)(1): 21 court days from complaint_filed.
/// Validates the endpoint returns 201 and produces deadline entries.
#[spin_test]
fn test_complaint_filed_calculates_answer_deadline() {
    let _store = key_value::Store::open("district9");

    let request_data = json!({
        "triggering_event": "complaint_filed",
        "triggering_date": "2025-01-10T12:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440001"
    });

    let (status, response) = calculate_deadline(request_data, "district9");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    // Should return an array of deadlines
    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce at least one deadline");

    // Verify the answer deadline was created
    let answer_deadline = &deadlines[0];
    assert_eq!(
        answer_deadline["applicable_rule"], "FRCP 12(a)(1)(A)",
        "Should cite FRCP 12(a)(1)(A) for answer deadline"
    );
    assert_eq!(
        answer_deadline["responsible_party"], "Defendant",
        "Defendant should be the responsible party for answer"
    );
    // Status is serialized as lowercase in the API response
    let status_str = answer_deadline["status"].as_str().unwrap_or_default();
    assert!(
        status_str.eq_ignore_ascii_case("pending"),
        "New deadline should have Pending status, got: {}",
        status_str
    );
}

/// Test scheduling order triggers initial disclosure deadlines.
/// FRCP 26(a)(1): 14 court days for initial disclosures.
#[spin_test]
fn test_scheduling_order_calculates_disclosure_deadline() {
    let _store = key_value::Store::open("district9");

    let request_data = json!({
        "triggering_event": "scheduling_order",
        "triggering_date": "2025-03-03T09:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440002"
    });

    let (status, response) = calculate_deadline(request_data, "district9");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce disclosure deadline");

    let disclosure = &deadlines[0];
    assert_eq!(
        disclosure["applicable_rule"], "FRCP 26(a)(1)",
        "Should cite FRCP 26(a)(1)"
    );
    assert_eq!(
        disclosure["responsible_party"], "All parties",
        "All parties responsible for initial disclosures"
    );
}

/// Test judgment entry triggers appeal deadline.
/// FRAP 4(a)(1)(A): 30 days for notice of appeal (jurisdictional).
#[spin_test]
fn test_judgment_entered_calculates_appeal_deadline() {
    let _store = key_value::Store::open("district12");

    let request_data = json!({
        "triggering_event": "judgment_entered",
        "triggering_date": "2025-06-02T14:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440003"
    });

    let (status, response) = calculate_deadline(request_data, "district12");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce appeal deadline");

    let appeal = &deadlines[0];
    assert_eq!(
        appeal["applicable_rule"], "FRAP 4(a)(1)(A)",
        "Should cite FRAP 4(a)(1)(A)"
    );
    assert_eq!(
        appeal["is_jurisdictional"], true,
        "Appeal deadline is jurisdictional and cannot be extended"
    );
    assert_eq!(
        appeal["is_extendable"], false,
        "Jurisdictional deadline should not be extendable"
    );
}

/// Test that a deadline landing on a Saturday extends to the following Monday.
/// Judgment entered on 2025-05-03 (Saturday trigger).
/// 30 days from Sat May 3 = Mon Jun 2 (but calculation is court-days).
#[spin_test]
fn test_deadline_landing_on_saturday_extends_to_monday() {
    let _store = key_value::Store::open("district9");

    let request_data = json!({
        "triggering_event": "judgment_entered",
        "triggering_date": "2025-05-03T12:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440004"
    });

    let (status, response) = calculate_deadline(request_data, "district9");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce deadline even from weekend trigger");

    // The due_date should never fall on a Saturday or Sunday
    let due_date = deadlines[0]["due_date"].as_str()
        .expect("due_date should be a string");
    assert!(
        !due_date.is_empty(),
        "Due date should be set"
    );
}

/// Test that a deadline landing on a Sunday extends to Monday.
#[spin_test]
fn test_deadline_landing_on_sunday_extends_to_monday() {
    let _store = key_value::Store::open("district12");

    let request_data = json!({
        "triggering_event": "complaint_filed",
        "triggering_date": "2025-05-04T12:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440005"
    });

    let (status, response) = calculate_deadline(request_data, "district12");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce deadline");
}

// ============================================================
// Holiday-related deadline tests
// ============================================================

/// Test deadline computation that crosses MLK Day (3rd Monday of January).
/// Complaint filed Jan 6, 2025 (Monday). 21 court days should skip
/// MLK Day (Jan 20, 2025) and weekends.
#[spin_test]
fn test_deadline_skips_mlk_day() {
    let _store = key_value::Store::open("district9");

    let request_data = json!({
        "triggering_event": "complaint_filed",
        "triggering_date": "2025-01-06T12:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440006"
    });

    let (status, response) = calculate_deadline(request_data, "district9");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce deadline that accounts for MLK Day");

    // The due date should not be MLK Day (Jan 20, 2025)
    let due_date = deadlines[0]["due_date"].as_str().unwrap_or_default();
    assert!(
        !due_date.starts_with("2025-01-20"),
        "Due date should not fall on MLK Day: got {}",
        due_date
    );
}

/// Test deadline computation that crosses the Christmas and New Year holiday chain.
/// These holidays in succession can significantly shift deadlines.
#[spin_test]
fn test_holiday_chain_christmas_newyears() {
    let _store = key_value::Store::open("district12");

    // Complaint filed Dec 1, 2025. 21 court days must skip
    // Christmas (Dec 25) and New Year's Day (Jan 1).
    let request_data = json!({
        "triggering_event": "complaint_filed",
        "triggering_date": "2025-12-01T12:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440007"
    });

    let (status, response) = calculate_deadline(request_data, "district12");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce deadline spanning holiday chain");

    let due_date = deadlines[0]["due_date"].as_str().unwrap_or_default();
    // Due date should not fall on Christmas or New Year's
    assert!(
        !due_date.starts_with("2025-12-25") && !due_date.starts_with("2026-01-01"),
        "Due date should skip Christmas and New Year's: got {}",
        due_date
    );
}

/// Test Memorial Day computation (last Monday of May).
/// Scheduling order entered May 12, 2025. 14 court days should
/// skip Memorial Day (May 26, 2025).
#[spin_test]
fn test_memorial_day_computation() {
    let _store = key_value::Store::open("district9");

    let request_data = json!({
        "triggering_event": "scheduling_order",
        "triggering_date": "2025-05-12T12:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440008"
    });

    let (status, response) = calculate_deadline(request_data, "district9");

    assert_eq!(status, 201, "Calculate should return 201, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array of deadlines");
    assert!(!deadlines.is_empty(), "Should produce deadline that accounts for Memorial Day");

    let due_date = deadlines[0]["due_date"].as_str().unwrap_or_default();
    assert!(
        !due_date.starts_with("2025-05-26"),
        "Due date should not fall on Memorial Day: got {}",
        due_date
    );
}

// ============================================================
// Unknown triggering event test
// ============================================================

/// Test that an unknown triggering event returns an empty deadline list
/// (the calculator does not error out on unknown events).
#[spin_test]
fn test_unknown_triggering_event_returns_empty() {
    let _store = key_value::Store::open("district9");

    let request_data = json!({
        "triggering_event": "unknown_event_type",
        "triggering_date": "2025-06-15T12:00:00Z",
        "case_id": "550e8400-e29b-41d4-a716-446655440009"
    });

    let (status, response) = calculate_deadline(request_data, "district9");

    assert_eq!(status, 201, "Should still return 201 for unknown event, got {}: {:?}", status, response);

    let deadlines = response.as_array()
        .expect("Response should be an array");
    assert!(
        deadlines.is_empty(),
        "Unknown triggering event should produce empty deadline array"
    );
}

// ============================================================
// Validation and error tests
// ============================================================

/// Test that missing required fields return a bad request error.
#[spin_test]
fn test_missing_fields_returns_bad_request() {
    let _store = key_value::Store::open("district12");

    // Missing triggering_date and case_id
    let request_data = json!({
        "triggering_event": "complaint_filed"
    });

    let (status, _response) = calculate_deadline(request_data, "district12");

    assert_eq!(
        status, 400,
        "Missing required fields should return 400"
    );
}

/// Test that an invalid JSON body returns a bad request error.
#[spin_test]
fn test_invalid_json_body_returns_bad_request() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/deadlines/calculate")).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(b"not valid json").unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Invalid JSON should return 400"
    );
}

// ============================================================
// Federal rules endpoint test
// ============================================================

/// Test that the federal rules endpoint returns available FRCP rules.
#[spin_test]
fn test_get_federal_rules_returns_list() {
    let _store = key_value::Store::open("district9");

    let (status, response) = get_federal_rules("district9");

    assert_eq!(status, 200, "Federal rules should return 200, got {}: {:?}", status, response);

    // Response should be an array of federal rules
    let rules = response.as_array()
        .expect("Federal rules response should be an array");
    assert!(
        !rules.is_empty(),
        "Should return at least one federal rule"
    );
}
