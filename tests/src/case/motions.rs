//! Criminal case motions tests
//!
//! Tests for POST /api/cases/{id}/motions and PATCH /api/cases/{id}/motions/ruling endpoints

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
        "title": "Test Case for Motions",
        "description": "This is a test case created for motions testing",
        "crimeType": "financial_fraud",
        "assignedJudge": "Judge TestMotions",
        "location": "Motions City, MC"
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

/// Helper to file a motion
fn file_motion_request(case_id: &str, motion_type: &str, filed_by: &str, description: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/motions", case_id))).unwrap();

    let motion_data = json!({
        "motionType": motion_type,
        "filedBy": filed_by,
        "description": description
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&motion_data).unwrap().as_bytes()).unwrap();
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

/// Helper to rule on a motion
fn rule_on_motion_request(case_id: &str, motion_id: &str, ruling: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/motions/ruling", case_id))).unwrap();

    let ruling_data = json!({
        "motionId": motion_id,
        "ruling": ruling
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&ruling_data).unwrap().as_bytes()).unwrap();
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
fn test_file_motion_suppress_evidence() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = file_motion_request(
        &case_id,
        "suppress_evidence",
        "Defense Counsel",
        "Motion to suppress illegally obtained evidence during search",
        "district9"
    );

    assert_eq!(status, 200, "Should return 200 for successful motion filing");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_file_all_valid_motion_types() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let motion_types = vec![
        "dismiss",
        "suppress_evidence",
        "change_venue",
        "continuance",
        "bail_review",
        "discovery",
        "compel_discovery",
        "in_limine",
        "severance",
        "joinder"
    ];

    for motion_type in motion_types {
        let (status, response) = file_motion_request(
            &case_id,
            motion_type,
            "Attorney Smith",
            &format!("Test motion for {}", motion_type),
            "district12"
        );

        assert_eq!(
            status, 200,
            "Should file motion type {} successfully", motion_type
        );
        assert_eq!(response["id"], case_id);
    }
}

#[spin_test]
fn test_file_motion_invalid_type() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = file_motion_request(
        &case_id,
        "invalid_motion_type",
        "Defense Counsel",
        "Invalid motion test",
        "district9"
    );

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid motion type, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("motion") || body_str.contains("invalid"),
        "Error should mention invalid motion type"
    );
}

#[spin_test]
fn test_file_multiple_motions() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // File first motion
    let (status1, response1) = file_motion_request(
        &case_id,
        "suppress_evidence",
        "Defense Attorney A",
        "Motion to suppress search evidence",
        "district12"
    );
    assert_eq!(status1, 200);

    // File second motion
    let (status2, response2) = file_motion_request(
        &case_id,
        "dismiss",
        "Defense Attorney B",
        "Motion to dismiss charges",
        "district12"
    );
    assert_eq!(status2, 200);

    // Both should succeed and relate to the same case
    assert_eq!(response1["id"], case_id);
    assert_eq!(response2["id"], case_id);
}

#[spin_test]
fn test_file_motion_with_detailed_description() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let detailed_description = "Motion to suppress all evidence obtained during the warrantless search of defendant's vehicle on March 15, 2024. The search violated the defendant's Fourth Amendment rights as there was no probable cause, no consent, and no exigent circumstances justifying the search. The evidence includes a laptop computer, financial documents, and USB drives containing allegedly incriminating data.";

    let (status, response) = file_motion_request(
        &case_id,
        "suppress_evidence",
        "Defense Attorney Johnson",
        detailed_description,
        "district9"
    );

    assert_eq!(status, 200, "Should handle detailed motion descriptions");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_rule_on_motion_granted() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // First file a motion
    let (file_status, file_response) = file_motion_request(
        &case_id,
        "suppress_evidence",
        "Defense Counsel",
        "Motion to suppress evidence",
        "district12"
    );
    assert_eq!(file_status, 200);

    // Generate a motion ID (in real scenario, this would come from the filed motion)
    let motion_id = "550e8400-e29b-41d4-a716-446655440001";

    // Rule on the motion
    let (status, response) = rule_on_motion_request(
        &case_id,
        motion_id,
        "granted",
        "district12"
    );

    assert_eq!(status, 200, "Should return 200 for successful motion ruling");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_rule_on_motion_denied() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // File a motion first
    let (file_status, _) = file_motion_request(
        &case_id,
        "dismiss",
        "Defense Counsel",
        "Motion to dismiss",
        "district9"
    );
    assert_eq!(file_status, 200);

    let motion_id = "550e8400-e29b-41d4-a716-446655440002";

    // Rule on the motion
    let (status, response) = rule_on_motion_request(
        &case_id,
        motion_id,
        "denied",
        "district9"
    );

    assert_eq!(status, 200, "Should return 200 for motion denial");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_rule_on_motion_various_rulings() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let rulings = vec!["granted", "denied", "granted_in_part", "deferred"];

    for (i, ruling) in rulings.iter().enumerate() {
        // File a motion
        let (file_status, _) = file_motion_request(
            &case_id,
            "continuance",
            "Attorney",
            &format!("Motion for ruling test {}", i + 1),
            "district12"
        );
        assert_eq!(file_status, 200);

        // Generate unique motion ID
        let motion_id = format!("550e8400-e29b-41d4-a716-44665544000{}", i + 1);

        // Rule on the motion
        let (status, response) = rule_on_motion_request(
            &case_id,
            &motion_id,
            ruling,
            "district12"
        );

        assert_eq!(
            status, 200,
            "Should handle ruling type {} successfully", ruling
        );
        assert_eq!(response["id"], case_id);
    }
}

#[spin_test]
fn test_file_motion_nonexistent_case() {
    let _store = key_value::Store::open("district9");

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, response) = file_motion_request(
        fake_id,
        "dismiss",
        "Defense Counsel",
        "Test motion",
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
fn test_rule_on_motion_nonexistent_case() {
    let _store = key_value::Store::open("district12");

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";
    let motion_id = "550e8400-e29b-41d4-a716-446655440001";

    let (status, response) = rule_on_motion_request(
        fake_id,
        motion_id,
        "granted",
        "district12"
    );

    assert_eq!(status, 404, "Should return 404 for non-existent case");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("not found") || body_str.contains("NotFound"),
        "Error should indicate case not found"
    );
}

#[spin_test]
fn test_motion_operations_require_district_header() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // Test file motion without district header
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/motions", case_id))).unwrap();

    let motion_data = json!({
        "motionType": "dismiss",
        "filedBy": "Defense Counsel",
        "description": "Test motion"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&motion_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing for file motion"
    );
}

#[spin_test]
fn test_motion_operations_district_isolation() {
    // Create stores for both districts
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    // Create case in district9
    let case_id = create_test_case("district9");

    // Try to file motion from district12
    let (status, _) = file_motion_request(
        &case_id,
        "dismiss",
        "Defense Counsel",
        "Test motion",
        "district12"
    );

    assert_eq!(
        status, 404,
        "Should not be able to file motion for district9 case from district12"
    );

    // Verify filing works in correct district
    let (status_correct, _) = file_motion_request(
        &case_id,
        "dismiss",
        "Defense Counsel",
        "Test motion",
        "district9"
    );
    assert_eq!(status_correct, 200, "Should file motion in correct district");
}

#[spin_test]
fn test_file_motion_invalid_uuid() {
    let _store = key_value::Store::open("district12");

    let invalid_id = "not-a-valid-uuid";

    let (status, _) = file_motion_request(
        invalid_id,
        "dismiss",
        "Defense Counsel",
        "Test motion",
        "district12"
    );

    assert_eq!(status, 400, "Should return 400 for invalid UUID format");
}

#[spin_test]
fn test_rule_on_motion_invalid_motion_id() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");
    let invalid_motion_id = "not-a-valid-uuid";

    let (status, response) = rule_on_motion_request(
        &case_id,
        invalid_motion_id,
        "granted",
        "district9"
    );

    assert!(
        status == 400 || status == 404,
        "Should return 400 or 404 for invalid motion ID format"
    );
}

#[spin_test]
fn test_file_motion_missing_required_fields() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/motions", case_id))).unwrap();

    // Missing required fields
    let motion_data = json!({
        "motionType": "dismiss"
        // Missing filedBy and description
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&motion_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 400, "Should return 400 for missing required fields");
}

#[spin_test]
fn test_motion_workflow_complete() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // File a motion
    let (file_status, file_response) = file_motion_request(
        &case_id,
        "suppress_evidence",
        "Defense Attorney Brown",
        "Motion to suppress evidence obtained during illegal search",
        "district9"
    );

    assert_eq!(file_status, 200, "Motion should be filed successfully");
    assert_eq!(file_response["id"], case_id);

    // Rule on the motion
    let motion_id = "550e8400-e29b-41d4-a716-446655440123";
    let (rule_status, rule_response) = rule_on_motion_request(
        &case_id,
        motion_id,
        "granted",
        "district9"
    );

    assert_eq!(rule_status, 200, "Motion ruling should be successful");
    assert_eq!(rule_response["id"], case_id);
}

#[spin_test]
fn test_file_motion_with_special_characters() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let (status, response) = file_motion_request(
        &case_id,
        "suppress_evidence",
        "Attorney O'Malley-Smith",
        "Motion to suppress: Evidence obtained through search of défendant's property @#$%",
        "district12"
    );

    assert_eq!(status, 200, "Should handle special characters in motion details");
    assert_eq!(response["id"], case_id);
}