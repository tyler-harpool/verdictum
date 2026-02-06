//! Rules engine query and filter tests
//!
//! Tests for list, filter, and search endpoints

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a rule
fn create_rule(rule_data: Value, district: &str) -> Value {
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
    let body = response.body_as_string().unwrap_or_default();
    serde_json::from_str(&body).unwrap()
}

/// Helper to GET with path and district
fn get_request(path: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(path)).unwrap();

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
fn test_list_rules_empty() {
    let _store = key_value::Store::open("district9");

    let (status, response) = get_request("/api/rules", "district9");

    assert_eq!(status, 200, "List rules should return 200, got {}", status);
    assert_eq!(response["total"], 0, "Empty store should have total 0");

    let rules = response["rules"].as_array().unwrap();
    assert!(rules.is_empty(), "Empty store should return no rules");
}

#[spin_test]
fn test_list_rules_by_category_filter() {
    let _store = key_value::Store::open("district9");

    // Create a filing rule
    create_rule(json!({
        "name": "Filing Deadline Rule",
        "description": "Filing deadline rule",
        "source": "frcp",
        "category": "filing"
    }), "district9");

    // Create a privacy rule
    create_rule(json!({
        "name": "Privacy Redaction Rule",
        "description": "Privacy redaction",
        "source": "local_rule",
        "category": "privacy"
    }), "district9");

    // Filter by category=filing
    let (status, response) = get_request("/api/rules/category/filing", "district9");

    assert_eq!(status, 200, "Get rules by category should return 200, got {}", status);

    let rules = response.as_array().unwrap();
    assert!(rules.len() >= 1, "Should have at least one filing rule");

    for rule in rules {
        assert_eq!(rule["category"], "filing", "All returned rules should be filing category");
    }
}

#[spin_test]
fn test_rules_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    // Create rule in district9
    create_rule(json!({
        "name": "District 9 Only Rule",
        "description": "Should only be visible in district9",
        "source": "standing_order",
        "category": "procedural"
    }), "district9");

    // List rules from district12 - should not see district9's rule
    let (status, response) = get_request("/api/rules", "district12");

    assert_eq!(status, 200, "List rules should return 200, got {}", status);

    let rules = response["rules"].as_array().unwrap();
    for rule in rules {
        assert_ne!(
            rule["name"], "District 9 Only Rule",
            "District 12 should not see district 9 rules"
        );
    }
}

#[spin_test]
fn test_get_active_rules_for_jurisdiction() {
    let _store = key_value::Store::open("district12");

    // Create an active rule with jurisdiction
    create_rule(json!({
        "name": "SDNY Active Rule",
        "description": "Active SDNY rule",
        "source": "local_rule",
        "category": "procedural",
        "status": "active",
        "jurisdiction": "SDNY"
    }), "district12");

    // Create a draft rule (should not appear)
    create_rule(json!({
        "name": "SDNY Draft Rule",
        "description": "Draft SDNY rule",
        "source": "local_rule",
        "category": "procedural",
        "status": "draft",
        "jurisdiction": "SDNY"
    }), "district12");

    let (status, response) = get_request("/api/rules/jurisdiction/SDNY", "district12");

    assert_eq!(status, 200, "Get active rules for jurisdiction should return 200, got {}", status);

    let rules = response.as_array().unwrap();
    for rule in rules {
        assert_eq!(rule["status"], "active", "Only active rules should be returned");
    }
}

#[spin_test]
fn test_get_rules_by_trigger() {
    let _store = key_value::Store::open("district9");

    // Create a rule with a specific trigger
    create_rule(json!({
        "name": "Motion Filed Trigger Rule",
        "description": "Rule triggered when a motion is filed",
        "source": "frcp",
        "category": "procedural",
        "triggers": ["motion_filed"],
        "status": "active"
    }), "district9");

    // Create another rule with a different trigger to verify filtering
    create_rule(json!({
        "name": "Case Filed Trigger Rule",
        "description": "Rule triggered when a case is filed",
        "source": "frcp",
        "category": "filing",
        "triggers": ["case_filed"],
        "status": "active"
    }), "district9");

    let (status, response) = get_request("/api/rules/trigger/motion_filed", "district9");

    assert_eq!(status, 200, "Get rules by trigger should return 200, got {}", status);

    let rules = response.as_array().unwrap();
    assert!(rules.len() >= 1, "Should have at least one rule with motion_filed trigger");

    for rule in rules {
        let triggers = rule["triggers"].as_array().unwrap();
        let has_motion_filed = triggers.iter().any(|t| t.as_str() == Some("motion_filed"));
        assert!(has_motion_filed, "All returned rules should have motion_filed trigger");
    }
}

#[spin_test]
fn test_evaluate_rules_placeholder() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/rules/evaluate")).unwrap();

    let evaluate_body = json!({
        "trigger": "motion_filed",
        "context": {
            "case_type": "criminal"
        }
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&evaluate_body).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(status, 200, "Evaluate rules should return 200, got {}", status);
    assert!(body_json.get("message").is_some(), "Response should contain a message field");
    let message = body_json["message"].as_str().unwrap();
    assert!(message.contains("Phase 2"), "Message should reference Phase 2, got: {}", message);
    assert_eq!(body_json["evaluated_count"], 0, "Placeholder should have evaluated_count of 0");
}
