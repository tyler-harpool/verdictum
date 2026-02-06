//! Rules engine DELETE endpoint tests
//!
//! Tests for DELETE /api/rules/:id endpoint

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a rule and return the response
fn create_test_rule(district: &str) -> Value {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/rules")).unwrap();

    let rule_data = json!({
        "name": "Rule to Delete",
        "description": "This rule will be deleted",
        "source": "local_rule",
        "category": "procedural"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&rule_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap_or_default();
    serde_json::from_str(&body).unwrap()
}

/// Helper to DELETE a rule
fn delete_rule(id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some(&format!("/api/rules/{}", id))).unwrap();

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

/// Helper to GET a rule by ID
fn get_rule_by_id(id: &str, district: &str) -> u16 {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/rules/{}", id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    response.status()
}

#[spin_test]
fn test_delete_rule_success() {
    let _store = key_value::Store::open("district12");

    let created = create_test_rule("district12");
    let rule_id = created["id"].as_str().unwrap();

    let (status, response) = delete_rule(rule_id, "district12");

    assert_eq!(status, 200, "Delete rule should return 200, got {}", status);
    assert_eq!(response["deleted"], true, "Response should indicate deletion");
}

#[spin_test]
fn test_delete_rule_not_found() {
    let _store = key_value::Store::open("district12");

    let random_uuid = "00000000-0000-0000-0000-000000000000";
    let (status, response) = delete_rule(random_uuid, "district12");

    assert_eq!(status, 200, "Delete non-existent rule should return 200, got {}", status);
    assert_eq!(response["deleted"], false, "Response should indicate no deletion");
}

#[spin_test]
fn test_delete_and_verify_gone() {
    let _store = key_value::Store::open("district12");

    let created = create_test_rule("district12");
    let rule_id = created["id"].as_str().unwrap();

    // Verify rule exists
    let get_status = get_rule_by_id(rule_id, "district12");
    assert_eq!(get_status, 200, "Rule should exist before deletion");

    // Delete the rule
    let (delete_status, _) = delete_rule(rule_id, "district12");
    assert_eq!(delete_status, 200, "Delete should succeed");

    // Verify rule is gone
    let get_after_status = get_rule_by_id(rule_id, "district12");
    assert_eq!(get_after_status, 404, "Rule should be gone after deletion, got {}", get_after_status);
}
