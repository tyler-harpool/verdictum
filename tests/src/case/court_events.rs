//! Criminal case court events tests
//!
//! Tests for POST /api/cases/{id}/events endpoint

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
        "title": "Test Case for Events",
        "description": "This is a test case created for court events testing",
        "crimeType": "financial_fraud",
        "assignedJudge": "Judge TestEvents",
        "location": "Events City, EC"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    body_json["id"].as_str().unwrap().to_string()
}

/// Helper to schedule a court event
fn schedule_event_request(case_id: &str, event_type: &str, scheduled_date: &str, description: &str, location: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/events", case_id))).unwrap();

    let event_data = json!({
        "eventType": event_type,
        "scheduledDate": scheduled_date,
        "description": description,
        "location": location
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&event_data).unwrap().as_bytes()).unwrap();
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
fn test_schedule_arraignment_event() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = schedule_event_request(
        &case_id,
        "arraignment",
        "2024-03-15T10:00:00Z",
        "Initial arraignment hearing",
        "Courtroom 3A",
        "district9"
    );

    assert_eq!(status, 200, "Should return 200 for successful event scheduling");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_schedule_all_valid_event_types() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let event_types = vec![
        "arraignment",
        "preliminary_hearing",
        "trial",
        "sentencing",
        "status_conference",
        "motion_hearing",
        "plea_hearing",
        "discovery_conference"
    ];

    for event_type in event_types {
        let (status, response) = schedule_event_request(
            &case_id,
            event_type,
            "2024-04-01T14:00:00Z",
            &format!("Test {} event", event_type),
            "Courtroom 1",
            "district12"
        );

        assert_eq!(
            status, 200,
            "Should schedule event type {} successfully", event_type
        );
        assert_eq!(response["id"], case_id);
    }
}

#[spin_test]
fn test_schedule_event_invalid_type() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = schedule_event_request(
        &case_id,
        "invalid_event_type",
        "2024-03-15T10:00:00Z",
        "Invalid event",
        "Courtroom 1",
        "district9"
    );

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid event type, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("event") || body_str.contains("invalid"),
        "Error should mention invalid event type"
    );
}

#[spin_test]
fn test_schedule_event_invalid_date_format() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let (status, response) = schedule_event_request(
        &case_id,
        "arraignment",
        "invalid-date-format",  // Invalid ISO 8601 format
        "Test event",
        "Courtroom 1",
        "district12"
    );

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid date format, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("date") || body_str.contains("format") || body_str.contains("invalid"),
        "Error should mention invalid date format"
    );
}

#[spin_test]
fn test_schedule_event_past_date() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // Use a date in the past
    let (status, response) = schedule_event_request(
        &case_id,
        "arraignment",
        "2020-01-01T10:00:00Z",  // Past date
        "Past event",
        "Courtroom 1",
        "district9"
    );

    // This might be allowed or rejected depending on business rules
    // The test should verify the actual behavior
    assert!(
        status == 200 || status == 400,
        "Should either allow or reject past dates, got {}", status
    );

    if status == 400 {
        let body_str = serde_json::to_string(&response).unwrap();
        assert!(
            body_str.contains("past") || body_str.contains("date"),
            "Error should mention past date if rejected"
        );
    }
}

#[spin_test]
fn test_schedule_multiple_events() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // Schedule first event
    let (status1, response1) = schedule_event_request(
        &case_id,
        "arraignment",
        "2024-03-15T10:00:00Z",
        "Initial arraignment",
        "Courtroom 3A",
        "district12"
    );
    assert_eq!(status1, 200);

    // Schedule second event
    let (status2, response2) = schedule_event_request(
        &case_id,
        "trial",
        "2024-06-01T09:00:00Z",
        "Trial proceedings",
        "Courtroom 1B",
        "district12"
    );
    assert_eq!(status2, 200);

    // Both should succeed and return the same case
    assert_eq!(response1["id"], case_id);
    assert_eq!(response2["id"], case_id);
}

#[spin_test]
fn test_schedule_event_with_detailed_description() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let detailed_description = "Pre-trial conference to discuss discovery motions, witness lists, and scheduling for the upcoming trial. All parties required to attend with prepared exhibits and witness testimony summaries.";

    let (status, response) = schedule_event_request(
        &case_id,
        "status_conference",
        "2024-04-10T14:30:00Z",
        detailed_description,
        "Conference Room 2",
        "district9"
    );

    assert_eq!(status, 200, "Should handle detailed descriptions");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_schedule_event_with_special_characters() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let (status, response) = schedule_event_request(
        &case_id,
        "arraignment",
        "2024-03-20T10:00:00Z",
        "Arraignment for défendant with special chars: @#$%",
        "Courtroom 3-A (Level 2)",
        "district12"
    );

    assert_eq!(status, 200, "Should handle special characters");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_schedule_event_nonexistent_case() {
    let _store = key_value::Store::open("district9");

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, response) = schedule_event_request(
        fake_id,
        "arraignment",
        "2024-03-15T10:00:00Z",
        "Test event",
        "Courtroom 1",
        "district9"
    );

    assert_eq!(status, 404, "Should return 404 for non-existent case");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error should indicate case not found"
    );
}

#[spin_test]
fn test_schedule_event_requires_district_header() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // Create request WITHOUT district header
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/events", case_id))).unwrap();

    let event_data = json!({
        "eventType": "arraignment",
        "scheduledDate": "2024-03-15T10:00:00Z",
        "description": "Test event",
        "location": "Courtroom 1"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&event_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_schedule_event_district_isolation() {
    // Create stores for both districts
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    // Create case in district9
    let case_id = create_test_case("district9");

    // Try to schedule event from district12
    let (status, _) = schedule_event_request(
        &case_id,
        "arraignment",
        "2024-03-15T10:00:00Z",
        "Test event",
        "Courtroom 1",
        "district12"
    );

    assert_eq!(
        status, 404,
        "Should not be able to schedule event for district9 case from district12"
    );

    // Verify scheduling works in correct district
    let (status_correct, _) = schedule_event_request(
        &case_id,
        "arraignment",
        "2024-03-15T10:00:00Z",
        "Test event",
        "Courtroom 1",
        "district9"
    );
    assert_eq!(status_correct, 200, "Should schedule event in correct district");
}

#[spin_test]
fn test_schedule_event_invalid_uuid() {
    let _store = key_value::Store::open("district9");

    let invalid_id = "not-a-valid-uuid";

    let (status, _) = schedule_event_request(
        invalid_id,
        "arraignment",
        "2024-03-15T10:00:00Z",
        "Test event",
        "Courtroom 1",
        "district9"
    );

    assert_eq!(status, 400, "Should return 400 for invalid UUID format");
}

#[spin_test]
fn test_schedule_event_missing_required_fields() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/events", case_id))).unwrap();

    // Missing required fields
    let event_data = json!({
        "eventType": "arraignment"
        // Missing scheduledDate, description, location
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&event_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 400, "Should return 400 for missing required fields");
}

#[spin_test]
fn test_schedule_event_malformed_json() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/events", case_id))).unwrap();

    // Malformed JSON
    let malformed_json = r#"{"eventType": "arraignment", "scheduledDate": "2024-03-15T10:00:00Z""#;

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(malformed_json.as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 400, "Should return 400 for malformed JSON");
}

#[spin_test]
fn test_schedule_event_with_timezone_variants() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // Test different timezone formats
    let time_formats = vec![
        "2024-03-15T10:00:00Z",           // UTC
        "2024-03-15T10:00:00.000Z",       // UTC with milliseconds
        "2024-03-15T10:00:00+00:00",      // UTC with offset
        "2024-03-15T10:00:00-05:00",      // EST offset
    ];

    for (i, time_format) in time_formats.iter().enumerate() {
        let (status, response) = schedule_event_request(
            &case_id,
            "status_conference",
            time_format,
            &format!("Event with timezone format {}", i + 1),
            "Courtroom 1",
            "district12"
        );

        assert_eq!(
            status, 200,
            "Should handle timezone format: {}", time_format
        );
        assert_eq!(response["id"], case_id);
    }
}