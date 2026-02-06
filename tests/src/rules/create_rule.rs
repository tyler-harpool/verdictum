//! Rules engine CREATE endpoint tests
//!
//! Tests for POST /api/rules endpoint

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to make POST request to create a rule
fn create_rule_request(rule_data: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/rules")).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&rule_data).unwrap().as_bytes()).unwrap();
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
fn test_create_rule_success() {
    let _store = key_value::Store::open("district9");

    let rule_data = json!({
        "name": "FRCP Rule 26 Initial Disclosure Deadline",
        "description": "Parties must make initial disclosures within 14 days of Rule 26(f) conference",
        "source": "frcp",
        "category": "filing"
    });

    let (status, response) = create_rule_request(rule_data, "district9");

    assert_eq!(status, 201, "Create rule should return 201, got {}", status);
    assert!(response.get("id").is_some(), "Response should contain rule ID");
    assert_eq!(response["name"], "FRCP Rule 26 Initial Disclosure Deadline");
    assert_eq!(response["source"], "frcp");
    assert_eq!(response["category"], "filing");
    assert_eq!(response["status"], "draft");
}

#[spin_test]
fn test_create_rule_all_sources() {
    let _store = key_value::Store::open("district9");

    let sources = vec![
        "frcp", "frcr_p", "fre", "frap", "local_rule",
        "admin_procedure", "standing_order", "statute", "general_order",
    ];

    for source in sources {
        let rule_data = json!({
            "name": format!("Test rule for source {}", source),
            "description": "Test rule",
            "source": source,
            "category": "procedural"
        });

        let (status, response) = create_rule_request(rule_data, "district9");
        assert_eq!(status, 201, "Create rule with source '{}' should return 201, got {}", source, status);
        assert_eq!(response["source"], source, "Source should match for '{}'", source);
    }
}

#[spin_test]
fn test_create_rule_with_conditions_and_actions() {
    let _store = key_value::Store::open("district9");

    let rule_data = json!({
        "name": "Privacy Redaction Rule",
        "description": "Automatically flag filings for redaction review",
        "source": "local_rule",
        "category": "privacy",
        "triggers": ["document_filed"],
        "conditions": [
            {
                "type": "and",
                "conditions": [
                    {"type": "field_equals", "field": "document_type", "value": "complaint"},
                    {"type": "field_exists", "field": "social_security_number"}
                ]
            }
        ],
        "actions": [
            {"type": "require_redaction", "fields": ["social_security_number", "date_of_birth"]},
            {"type": "flag_for_review", "reason": "PII detected in filing"}
        ],
        "priority": "local",
        "status": "active",
        "jurisdiction": "SDNY",
        "citation": "Local Rule 5.2(a)"
    });

    let (status, response) = create_rule_request(rule_data, "district9");

    assert_eq!(status, 201, "Create rule should return 201, got {}", status);
    assert_eq!(response["status"], "active");
    assert_eq!(response["priority"], "local");
    assert_eq!(response["jurisdiction"], "SDNY");

    let triggers = response["triggers"].as_array().unwrap();
    assert_eq!(triggers.len(), 1);

    let conditions = response["conditions"].as_array().unwrap();
    assert_eq!(conditions.len(), 1);

    let actions = response["actions"].as_array().unwrap();
    assert_eq!(actions.len(), 2);
}

#[spin_test]
fn test_create_rule_invalid_json() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/rules")).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(b"not valid json").unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 400, "Invalid JSON should return 400");
}

#[spin_test]
fn test_create_rule_requires_district_header() {
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/rules")).unwrap();

    let rule_data = json!({
        "name": "Test Rule",
        "description": "Test",
        "source": "frcp",
        "category": "filing"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&rule_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 400, "Missing district header should return 400");
}
