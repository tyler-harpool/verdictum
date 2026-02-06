//! Rules engine UPDATE endpoint tests
//!
//! Tests for PUT /api/rules/:id endpoint

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
        "name": "Original Rule Name",
        "description": "Original description",
        "source": "frcp",
        "category": "filing",
        "status": "draft"
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

/// Helper to PUT update a rule
fn update_rule(id: &str, update_data: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Put).unwrap();
    request.set_path_with_query(Some(&format!("/api/rules/{}", id))).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&update_data).unwrap().as_bytes()).unwrap();
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
fn test_update_rule_name() {
    let _store = key_value::Store::open("district9");

    let created = create_test_rule("district9");
    let rule_id = created["id"].as_str().unwrap();

    let update_data = json!({
        "name": "Updated Rule Name"
    });

    let (status, response) = update_rule(rule_id, update_data, "district9");

    assert_eq!(status, 200, "Update rule should return 200, got {}", status);
    assert_eq!(response["name"], "Updated Rule Name");
    assert_eq!(response["description"], "Original description", "Unchanged fields should persist");
}

#[spin_test]
fn test_update_rule_status_to_active() {
    let _store = key_value::Store::open("district9");

    let created = create_test_rule("district9");
    let rule_id = created["id"].as_str().unwrap();

    assert_eq!(created["status"], "draft");

    let update_data = json!({
        "status": "active"
    });

    let (status, response) = update_rule(rule_id, update_data, "district9");

    assert_eq!(status, 200, "Update rule status should return 200, got {}", status);
    assert_eq!(response["status"], "active");
}

#[spin_test]
fn test_update_rule_not_found() {
    let _store = key_value::Store::open("district9");

    let random_uuid = "00000000-0000-0000-0000-000000000000";
    let update_data = json!({
        "name": "This should fail"
    });

    let (status, _response) = update_rule(random_uuid, update_data, "district9");

    assert_eq!(status, 404, "Updating non-existent rule should return 404, got {}", status);
}
