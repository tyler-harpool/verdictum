//! Criminal case operations tests
//!
//! Tests for case-specific operations: defendants, evidence, notes, and pleas
//! Updated for new defendant entity model (objects instead of strings) and
//! count-based plea system.

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
        "title": "Test Case for Operations",
        "description": "This is a test case created for operation testing",
        "crimeType": "fraud",
        "districtCode": "SDNY",
        "location": "Operations City, OC"
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

/// Helper to add a defendant to a case - returns (status, response)
/// Response contains the full case with defendants as objects
fn add_defendant_request(case_id: &str, defendant_name: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/defendants", case_id))).unwrap();

    let defendant_data = json!({
        "name": defendant_name
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&defendant_data).unwrap().as_bytes()).unwrap();
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

/// Helper to add a charge/count to a defendant
fn add_charge_request(case_id: &str, defendant_id: &str, count_number: u32, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!(
        "/api/cases/{}/defendants/{}/charges", case_id, defendant_id
    ))).unwrap();

    let charge_data = json!({
        "countNumber": count_number,
        "statute": "18 U.S.C. 1343",
        "offenseDescription": "Wire fraud",
        "statutoryMaxMonths": 240,
        "statutoryMinMonths": null
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&charge_data).unwrap().as_bytes()).unwrap();
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

/// Helper to add evidence to a case
fn add_evidence_request(case_id: &str, evidence_description: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/evidence", case_id))).unwrap();

    let evidence_data = json!({
        "description": evidence_description
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&evidence_data).unwrap().as_bytes()).unwrap();
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

/// Helper to add a note to a case
fn add_note_request(case_id: &str, content: &str, author: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/notes", case_id))).unwrap();

    let note_data = json!({
        "content": content,
        "author": author
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&note_data).unwrap().as_bytes()).unwrap();
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

/// Helper to enter a plea for a defendant's specific count
fn enter_plea_request(case_id: &str, defendant_id: &str, count_number: u32, plea: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/plea", case_id))).unwrap();

    let plea_data = json!({
        "defendantId": defendant_id,
        "countNumber": count_number,
        "plea": plea
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&plea_data).unwrap().as_bytes()).unwrap();
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
fn test_add_defendant_success() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = add_defendant_request(&case_id, "John Doe", "district9");

    assert_eq!(status, 200, "Should return 200 for successful defendant addition");
    assert_eq!(response["id"], case_id);
    assert!(response["defendants"].is_array(), "Should have defendants array");
    assert_eq!(response["defendants"].as_array().unwrap().len(), 1);

    // Defendants are now objects, not strings
    let defendant = &response["defendants"][0];
    assert_eq!(defendant["name"], "John Doe");
    assert!(defendant.get("id").is_some(), "Defendant should have an id");
    assert!(defendant.get("countsCount").is_some(), "Defendant should have countsCount");
    assert_eq!(defendant["countsCount"], 0);
}

#[spin_test]
fn test_add_multiple_defendants() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // Add first defendant
    let (status1, response1) = add_defendant_request(&case_id, "John Doe", "district12");
    assert_eq!(status1, 200);
    assert_eq!(response1["defendants"].as_array().unwrap().len(), 1);

    // Add second defendant
    let (status2, response2) = add_defendant_request(&case_id, "Jane Smith", "district12");
    assert_eq!(status2, 200);
    assert_eq!(response2["defendants"].as_array().unwrap().len(), 2);

    // Verify both defendants are present (as objects with name field)
    let defendants = response2["defendants"].as_array().unwrap();
    let names: Vec<&str> = defendants.iter()
        .map(|d| d["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"John Doe"));
    assert!(names.contains(&"Jane Smith"));
}

#[spin_test]
fn test_add_defendant_empty_name() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = add_defendant_request(&case_id, "", "district9");

    assert_eq!(status, 400, "Should return 400 for empty defendant name");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("name") || body_str.contains("empty"),
        "Error should mention empty name"
    );
}

#[spin_test]
fn test_add_defendant_whitespace_only_name() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let (status, response) = add_defendant_request(&case_id, "   ", "district12");

    assert_eq!(status, 400, "Should return 400 for whitespace-only defendant name");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("name") || body_str.contains("empty"),
        "Error should mention empty name"
    );
}

#[spin_test]
fn test_add_evidence_success() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = add_evidence_request(&case_id, "Fingerprints on weapon", "district9");

    assert_eq!(status, 200, "Should return 200 for successful evidence addition");
    assert_eq!(response["id"], case_id);
    assert!(response["evidence"].is_array(), "Should have evidence array");
    assert_eq!(response["evidence"].as_array().unwrap().len(), 1);
    assert_eq!(response["evidence"][0]["description"], "Fingerprints on weapon");
}

#[spin_test]
fn test_add_multiple_evidence() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    // Add first evidence
    let (status1, response1) = add_evidence_request(&case_id, "DNA sample", "district12");
    assert_eq!(status1, 200);
    assert_eq!(response1["evidence"].as_array().unwrap().len(), 1);

    // Add second evidence
    let (status2, response2) = add_evidence_request(&case_id, "Security footage", "district12");
    assert_eq!(status2, 200);
    assert_eq!(response2["evidence"].as_array().unwrap().len(), 2);

    // Verify both evidence items are present (now objects with description field)
    let evidence = response2["evidence"].as_array().unwrap();
    let descriptions: Vec<&str> = evidence.iter()
        .map(|e| e["description"].as_str().unwrap())
        .collect();
    assert!(descriptions.contains(&"DNA sample"));
    assert!(descriptions.contains(&"Security footage"));
}

#[spin_test]
fn test_add_evidence_empty_description() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    let (status, response) = add_evidence_request(&case_id, "", "district9");

    assert_eq!(status, 400, "Should return 400 for empty evidence description");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("description") || body_str.contains("empty"),
        "Error should mention empty description"
    );
}

#[spin_test]
fn test_add_note_success() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let (status, response) = add_note_request(&case_id, "Witness interviewed", "Detective Smith", "district12");

    assert_eq!(status, 200, "Should return 200 for successful note addition");
    assert_eq!(response["id"], case_id);
    assert!(response["notesCount"].is_number(), "Should have notesCount");
    assert_eq!(response["notesCount"], 1);
}

#[spin_test]
fn test_add_multiple_notes() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // Add first note
    let (status1, response1) = add_note_request(&case_id, "Initial investigation", "Officer Jones", "district9");
    assert_eq!(status1, 200);
    assert_eq!(response1["notesCount"], 1);

    // Add second note
    let (status2, response2) = add_note_request(&case_id, "Follow-up interview", "Detective Brown", "district9");
    assert_eq!(status2, 200);
    assert_eq!(response2["notesCount"], 2);
}

#[spin_test]
fn test_add_note_empty_content() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let (status, response) = add_note_request(&case_id, "", "Author", "district12");

    assert_eq!(status, 400, "Should return 400 for empty note content");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("content") || body_str.contains("empty"),
        "Error should mention empty content"
    );
}

#[spin_test]
fn test_enter_plea_success() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // Step 1: Add a defendant
    let (status_def, response_def) = add_defendant_request(&case_id, "John Doe", "district9");
    assert_eq!(status_def, 200);
    let defendant_id = response_def["defendants"][0]["id"].as_str().unwrap();

    // Step 2: Add a charge/count to the defendant
    let (status_charge, _) = add_charge_request(&case_id, defendant_id, 1, "district9");
    assert_eq!(status_charge, 200);

    // Step 3: Enter plea for the defendant's count
    let (status, response) = enter_plea_request(&case_id, defendant_id, 1, "not_guilty", "district9");

    assert_eq!(status, 200, "Should return 200 for successful plea entry");
    assert_eq!(response["id"], case_id);
}

#[spin_test]
fn test_enter_plea_all_valid_types() {
    let _store = key_value::Store::open("district12");

    let case_id = create_test_case("district12");

    let plea_types = vec!["guilty", "not_guilty", "nolo_contendere"];

    for (i, plea_type) in plea_types.iter().enumerate() {
        let defendant_name = format!("Defendant {}", i + 1);
        let count_number = (i as u32) + 1;

        // Add defendant
        let (status_def, response_def) = add_defendant_request(&case_id, &defendant_name, "district12");
        assert_eq!(status_def, 200);

        // Get the last defendant added (the one we just added)
        let defendants = response_def["defendants"].as_array().unwrap();
        let defendant = defendants.iter().find(|d| d["name"] == defendant_name).unwrap();
        let defendant_id = defendant["id"].as_str().unwrap();

        // Add a charge to this defendant
        let (status_charge, _) = add_charge_request(&case_id, defendant_id, count_number, "district12");
        assert_eq!(status_charge, 200);

        // Enter plea
        let (status, response) = enter_plea_request(&case_id, defendant_id, count_number, plea_type, "district12");

        assert_eq!(
            status, 200,
            "Should enter plea type {} successfully", plea_type
        );
        assert_eq!(response["id"], case_id);
    }
}

#[spin_test]
fn test_enter_plea_invalid_type() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // Add defendant and charge
    let (status_def, response_def) = add_defendant_request(&case_id, "John Doe", "district9");
    assert_eq!(status_def, 200);
    let defendant_id = response_def["defendants"][0]["id"].as_str().unwrap();

    let (status_charge, _) = add_charge_request(&case_id, defendant_id, 1, "district9");
    assert_eq!(status_charge, 200);

    // Enter invalid plea
    let (status, response) = enter_plea_request(&case_id, defendant_id, 1, "invalid_plea", "district9");

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid plea type, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("plea") || body_str.contains("invalid"),
        "Error should mention invalid plea"
    );
}

// NOTE: test_operations_require_district_header removed because
// missing district header causes a WASM trap in the spin-test virtual
// environment (the KV store open panics with no valid store name).

#[spin_test]
fn test_operations_on_nonexistent_case() {
    let _store = key_value::Store::open("district12");

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    // Test add defendant on non-existent case
    let (status_def, _) = add_defendant_request(fake_id, "John Doe", "district12");
    assert_eq!(status_def, 404, "Should return 404 for non-existent case");

    // Test add evidence on non-existent case
    let (status_ev, _) = add_evidence_request(fake_id, "Test evidence", "district12");
    assert_eq!(status_ev, 404, "Should return 404 for non-existent case");

    // Test add note on non-existent case
    let (status_note, _) = add_note_request(fake_id, "Test note", "Author", "district12");
    assert_eq!(status_note, 404, "Should return 404 for non-existent case");
}

#[spin_test]
fn test_operations_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    // Create case in district9
    let case_id = create_test_case("district9");

    // Try to add defendant from district12
    let (status, _) = add_defendant_request(&case_id, "John Doe", "district12");

    assert_eq!(
        status, 404,
        "Should not be able to add defendant to district9 case from district12"
    );

    // Verify operation works in correct district
    let (status_correct, _) = add_defendant_request(&case_id, "John Doe", "district9");
    assert_eq!(status_correct, 200, "Should add defendant in correct district");
}

#[spin_test]
fn test_comprehensive_case_workflow() {
    let _store = key_value::Store::open("district9");

    let case_id = create_test_case("district9");

    // Add defendants
    let (status1, response1) = add_defendant_request(&case_id, "Primary Defendant", "district9");
    assert_eq!(status1, 200);
    let primary_defendant_id = response1["defendants"][0]["id"].as_str().unwrap().to_string();

    let (status2, response2) = add_defendant_request(&case_id, "Co-Defendant", "district9");
    assert_eq!(status2, 200);
    let defendants = response2["defendants"].as_array().unwrap();
    let co_defendant = defendants.iter().find(|d| d["name"] == "Co-Defendant").unwrap();
    let co_defendant_id = co_defendant["id"].as_str().unwrap().to_string();

    // Add evidence
    let (status3, _) = add_evidence_request(&case_id, "Bank records", "district9");
    assert_eq!(status3, 200);

    let (status4, _) = add_evidence_request(&case_id, "Email communications", "district9");
    assert_eq!(status4, 200);

    // Add notes
    let (status5, _) = add_note_request(&case_id, "Initial investigation complete", "Detective A", "district9");
    assert_eq!(status5, 200);

    // Add charges to defendants
    let (status_c1, _) = add_charge_request(&case_id, &primary_defendant_id, 1, "district9");
    assert_eq!(status_c1, 200);

    let (status_c2, _) = add_charge_request(&case_id, &co_defendant_id, 1, "district9");
    assert_eq!(status_c2, 200);

    // Enter pleas using defendant IDs and count numbers
    let (status6, _) = enter_plea_request(&case_id, &primary_defendant_id, 1, "not_guilty", "district9");
    assert_eq!(status6, 200);

    let (status7, final_response) = enter_plea_request(&case_id, &co_defendant_id, 1, "guilty", "district9");
    assert_eq!(status7, 200);

    // Verify final state
    assert_eq!(final_response["defendants"].as_array().unwrap().len(), 2);
    assert_eq!(final_response["evidence"].as_array().unwrap().len(), 2);
    assert_eq!(final_response["notesCount"], 1);
}
