//! Judge management integration tests
//!
//! Tests for judge CRUD operations, assignments, recusals, and conflicts

use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};
use spin_test_sdk::{
    bindings::{fermyon::spin_test_virt::key_value, wasi::http},
    spin_test,
};

// ============================================================================
// CREATE Judge Tests
// ============================================================================

#[spin_test]
fn test_create_judge_minimal() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/judges")).unwrap();

    let judge_data = json!({
        "name": "Judge Smith",
        "title": "district_judge",
        "district": "district9",
        "courtroom": "101A"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json["id"].is_string());
    assert_eq!(body_json["name"], "Judge Smith");
    assert_eq!(body_json["title"], "district_judge");
    assert_eq!(body_json["district"], "district9");
    assert_eq!(body_json["courtroom"], "101A");
    assert_eq!(body_json["status"], "active");
}

#[spin_test]
fn test_create_judge_with_all_titles() {
    let _store = key_value::Store::open("district12");

    let judge_titles = vec![
        "chief_judge",
        "district_judge",
        "senior_judge",
        "magistrate_judge",
        "bankruptcy_judge",
        "visiting_judge"
    ];

    for (idx, title) in judge_titles.iter().enumerate() {
        let headers = Headers::new();
        headers.append(&"X-Court-District".to_string(), b"district12").unwrap();
        headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

        let request = OutgoingRequest::new(headers);
        request.set_method(&Method::Post).unwrap();
        request.set_path_with_query(Some("/api/judges")).unwrap();

        let judge_data = json!({
            "name": format!("Judge {}", idx),
            "title": title,
            "district": "district12",
            "courtroom": format!("{}A", idx)
        });

        let request_body = request.body().unwrap();
        let stream = request_body.write().unwrap();
        stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
        drop(stream);
        http::types::OutgoingBody::finish(request_body, None).unwrap();

        let response = spin_test_sdk::perform_request(request);
        assert_eq!(response.status(), 201);

        let body = response.body_as_string().unwrap();
        let body_json: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(body_json["title"], title.to_string());
    }
}

// ============================================================================
// GET Judge Tests
// ============================================================================

#[spin_test]
fn test_get_judge_by_id() {
    let _store = key_value::Store::open("district9");

    // Create a judge first
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/judges")).unwrap();

    let judge_data = json!({
        "name": "Judge Brown",
        "title": "senior_judge",
        "district": "district9",
        "courtroom": "102B"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let created_judge: Value = serde_json::from_str(&body).unwrap();
    let judge_id = created_judge["id"].as_str().unwrap();

    // Now get the judge by ID
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/judges/{}", judge_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(body_json["id"], judge_id);
    assert_eq!(body_json["name"], "Judge Brown");
    assert_eq!(body_json["title"], "senior_judge");
}

#[spin_test]
fn test_get_nonexistent_judge() {
    let _store = key_value::Store::open("district12");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges/00000000-0000-0000-0000-000000000000")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 404);
}

// ============================================================================
// LIST Judges Tests
// ============================================================================

#[spin_test]
fn test_get_all_judges() {
    let _store = key_value::Store::open("district9");

    // Create multiple judges
    for i in 0..3 {
        let headers = Headers::new();
        headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
        headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

        let request = OutgoingRequest::new(headers);
        request.set_method(&Method::Post).unwrap();
        request.set_path_with_query(Some("/api/judges")).unwrap();

        let judge_data = json!({
            "name": format!("Judge List-{}", i),
            "title": "district_judge",
            "district": "district9",
            "courtroom": format!("{}C", i)
        });

        let request_body = request.body().unwrap();
        let stream = request_body.write().unwrap();
        stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
        drop(stream);
        http::types::OutgoingBody::finish(request_body, None).unwrap();

        let response = spin_test_sdk::perform_request(request);
        assert_eq!(response.status(), 201);
    }

    // Get all judges
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json.is_array());
    let judges = body_json.as_array().unwrap();
    assert!(judges.len() >= 3);
}

#[spin_test]
fn test_get_available_judges() {
    let _store = key_value::Store::open("district12");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges/available")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json.is_array());
}

// ============================================================================
// UPDATE Judge Status Tests
// ============================================================================

#[spin_test]
fn test_update_judge_status() {
    let _store = key_value::Store::open("district9");

    // Create a judge first
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/judges")).unwrap();

    let judge_data = json!({
        "name": "Judge Status",
        "title": "district_judge",
        "district": "district9",
        "courtroom": "103C"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let created_judge: Value = serde_json::from_str(&body).unwrap();
    let judge_id = created_judge["id"].as_str().unwrap();

    // Update status
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Patch).unwrap();
    request.set_path_with_query(Some(&format!("/api/judges/{}/status", judge_id))).unwrap();

    let status_data = json!({
        "status": "on_leave"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&status_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(body_json["status"], "on_leave");
}

// ============================================================================
// Conflict of Interest Tests
// ============================================================================

#[spin_test]
fn test_add_conflict_of_interest() {
    let _store = key_value::Store::open("district12");

    // Create a judge first
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/judges")).unwrap();

    let judge_data = json!({
        "name": "Judge Conflict",
        "title": "district_judge",
        "district": "district12",
        "courtroom": "104D"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let created_judge: Value = serde_json::from_str(&body).unwrap();
    let judge_id = created_judge["id"].as_str().unwrap();

    // Add conflict
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/judges/{}/conflicts", judge_id))).unwrap();

    let conflict_data = json!({
        "party_name": "Acme Corp",
        "conflict_type": "financial_interest",
        "notes": "Owns significant shares"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&conflict_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json["id"].is_string());
    assert_eq!(body_json["party_name"], "Acme Corp");
    assert_eq!(body_json["conflict_type"], "financial_interest");
}

#[spin_test]
fn test_check_conflicts() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges/conflicts/check/TestParty")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json["conflicts_found"].is_boolean());
    assert!(body_json["judges_with_conflicts"].is_array());
    assert!(body_json["conflict_details"].is_array());
}

// ============================================================================
// Workload Statistics Tests
// ============================================================================

#[spin_test]
fn test_get_workload_stats() {
    let _store = key_value::Store::open("district12");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges/workload")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json["total_judges"].is_number());
    assert!(body_json["active_judges"].is_number());
    assert!(body_json["average_caseload"].is_number());
    assert!(body_json["overloaded_judges"].is_number());
    assert!(body_json["available_capacity"].is_number());
}

// ============================================================================
// Search Judges Tests
// ============================================================================

#[spin_test]
fn test_search_judges() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges/search?q=Judge")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json["judges"].is_array());
    assert!(body_json["total"].is_number());
}

// ============================================================================
// DELETE Judge Tests
// ============================================================================

#[spin_test]
fn test_delete_judge() {
    let _store = key_value::Store::open("district12");

    // Create a judge first
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/judges")).unwrap();

    let judge_data = json!({
        "name": "Judge ToDelete",
        "title": "district_judge",
        "district": "district12",
        "courtroom": "105E"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let created_judge: Value = serde_json::from_str(&body).unwrap();
    let judge_id = created_judge["id"].as_str().unwrap();

    // Delete the judge
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some(&format!("/api/judges/{}", judge_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 204);

    // Verify judge is deleted
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/judges/{}", judge_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 404);
}

#[spin_test]
fn test_delete_nonexistent_judge() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some("/api/judges/00000000-0000-0000-0000-000000000000")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 404);
}

// ============================================================================
// Judge Recusal Tests
// ============================================================================

#[spin_test]
fn test_file_recusal() {
    let _store = key_value::Store::open("district9");

    // Create a judge first
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/judges")).unwrap();

    let judge_data = json!({
        "name": "Judge Recuse",
        "title": "district_judge",
        "district": "district9",
        "courtroom": "106F"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&judge_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let created_judge: Value = serde_json::from_str(&body).unwrap();
    let judge_id = created_judge["id"].as_str().unwrap();

    // File recusal
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/judges/{}/recusals", judge_id))).unwrap();

    let recusal_data = json!({
        "case_id": "00000000-0000-0000-0000-000000000001",
        "filed_by": "Defense Counsel",
        "reason": "financial_interest",
        "detailed_grounds": "Judge owns stock in plaintiff company"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&recusal_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 201);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json["id"].is_string());
    assert_eq!(body_json["filed_by"], "Defense Counsel");
    assert_eq!(body_json["reason"], "financial_interest");
    assert_eq!(body_json["status"], "pending");
}

// ============================================================================
// Filter by Status Tests
// ============================================================================

#[spin_test]
fn test_get_judges_by_status() {
    let _store = key_value::Store::open("district12");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges/status/active")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json.is_array());
    let judges = body_json.as_array().unwrap();

    // All judges should have active status
    for judge in judges {
        assert_eq!(judge["status"], "active");
    }
}

// ============================================================================
// Filter by District Tests
// ============================================================================

#[spin_test]
fn test_get_judges_by_district() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/judges/district/district9")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json.is_array());
    let judges = body_json.as_array().unwrap();

    // All judges should be from district9
    for judge in judges {
        assert_eq!(judge["district"], "district9");
    }
}