//! Rules engine GET endpoint tests
//!
//! Tests for GET /api/rules/:id endpoint

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a rule and return its ID
fn create_test_rule(district: &str) -> Value {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/rules")).unwrap();

    let rule_data = json!({
        "name": "Test Filing Rule",
        "description": "Test rule for GET endpoint testing",
        "source": "frcp",
        "category": "filing",
        "status": "active"
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

/// Helper to GET a rule by ID
fn get_rule_by_id(id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
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

#[spin_test]
fn test_get_rule_by_id() {
    let _store = key_value::Store::open("district9");

    let created_rule = create_test_rule("district9");
    let rule_id = created_rule["id"].as_str().unwrap();

    let (status, response) = get_rule_by_id(rule_id, "district9");

    assert_eq!(status, 200, "Get rule by ID should return 200, got {}", status);
    assert_eq!(response["id"], rule_id);
    assert_eq!(response["name"], "Test Filing Rule");
}

#[spin_test]
fn test_get_rule_not_found() {
    let _store = key_value::Store::open("district9");

    let random_uuid = "00000000-0000-0000-0000-000000000000";
    let (status, _response) = get_rule_by_id(random_uuid, "district9");

    assert_eq!(status, 404, "Non-existent rule should return 404, got {}", status);
}

#[spin_test]
fn test_get_rule_invalid_id() {
    let _store = key_value::Store::open("district9");

    let (status, _response) = get_rule_by_id("not-a-uuid", "district9");

    assert_eq!(status, 400, "Invalid UUID should return 400, got {}", status);
}
