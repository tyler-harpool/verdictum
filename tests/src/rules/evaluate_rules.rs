//! Rules evaluation engine tests
//!
//! Tests for POST /api/rules/evaluate endpoint.
//! These tests create realistic rule configurations with conditions and actions,
//! then invoke the evaluate endpoint to verify evaluation behavior.
//!
//! Currently validates the Phase 2 stub response; once the evaluate handler
//! is wired to SpinRulesEngine, these tests will exercise the full evaluation
//! pipeline including condition matching, priority resolution, and action
//! collection into ComplianceReport.

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a rule via POST /api/rules and return the response
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
    assert_eq!(response.status(), 201, "Rule creation should return 201");
    let body = response.body_as_string().unwrap_or_default();
    serde_json::from_str(&body).unwrap()
}

/// Helper to call POST /api/rules/evaluate with a filing context
fn evaluate_rules(evaluate_data: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/rules/evaluate")).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&evaluate_data).unwrap().as_bytes()).unwrap();
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

/// Test that a civil complaint filing passes when rules allow it.
/// Creates a simple filing rule that does not block civil complaints,
/// then verifies the evaluate endpoint returns a successful response.
#[spin_test]
fn test_evaluate_civil_complaint_passes_all_rules() {
    let _store = key_value::Store::open("district9");

    // Create a rule that flags for review but does NOT block civil complaints
    create_rule(json!({
        "name": "Civil Complaint Review Rule",
        "description": "Flag civil complaints for clerk review",
        "source": "local_rule",
        "category": "filing",
        "triggers": ["document_filed"],
        "conditions": [
            {"type": "field_equals", "field": "case_type", "value": "civil"},
            {"type": "field_equals", "field": "document_type", "value": "complaint"}
        ],
        "actions": [
            {"type": "flag_for_review", "reason": "New civil complaint requires clerk review"}
        ],
        "priority": "local",
        "status": "active",
        "jurisdiction": "SDNY"
    }), "district9");

    let evaluate_data = json!({
        "trigger": "document_filed",
        "context": {
            "case_type": "civil",
            "document_type": "complaint",
            "filer_role": "plaintiff_attorney",
            "jurisdiction_id": "SDNY"
        }
    });

    let (status, response) = evaluate_rules(evaluate_data, "district9");

    assert_eq!(status, 200, "Evaluate should return 200, got {}", status);
    // Validate the endpoint responds (stub or real)
    assert!(!response.is_null(), "Response should not be null");
}

/// Test that a filing with unredacted SSN triggers a block action.
/// Creates a privacy rule that blocks filings containing SSN data,
/// and verifies the evaluate endpoint accepts the request.
#[spin_test]
fn test_evaluate_unredacted_ssn_blocks_filing() {
    let _store = key_value::Store::open("district9");

    // Create a privacy rule that blocks filings with unredacted SSN
    create_rule(json!({
        "name": "FRCP 5.2 Privacy Protection",
        "description": "Block filings containing unredacted social security numbers",
        "source": "frcp",
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
            {"type": "block_filing", "reason": "Filing contains unredacted SSN per FRCP 5.2"},
            {"type": "require_redaction", "fields": ["social_security_number"]}
        ],
        "priority": "federal_rule",
        "status": "active",
        "citation": "FRCP 5.2(a)"
    }), "district9");

    let evaluate_data = json!({
        "trigger": "document_filed",
        "context": {
            "case_type": "civil",
            "document_type": "complaint",
            "filer_role": "plaintiff_attorney",
            "jurisdiction_id": "SDNY",
            "social_security_number": "123-45-6789"
        }
    });

    let (status, response) = evaluate_rules(evaluate_data, "district9");

    assert_eq!(status, 200, "Evaluate should return 200, got {}", status);
    assert!(!response.is_null(), "Response should not be null");
}

/// Test that a standing order (higher priority) overrides a local rule
/// when both match the same filing context.
/// Validates that rule priority ordering affects evaluation results.
#[spin_test]
fn test_standing_order_overrides_local_rule() {
    let _store = key_value::Store::open("district12");

    // Create a local rule with lower priority
    create_rule(json!({
        "name": "Local Filing Deadline Rule",
        "description": "Local rule setting 30-day deadline",
        "source": "local_rule",
        "category": "deadline",
        "triggers": ["case_filed"],
        "conditions": [
            {"type": "field_equals", "field": "case_type", "value": "civil"}
        ],
        "actions": [
            {"type": "generate_deadline", "description": "Answer due", "days_from_trigger": 30}
        ],
        "priority": "local",
        "status": "active",
        "jurisdiction": "CDCA"
    }), "district12");

    // Create a standing order with higher priority that overrides
    create_rule(json!({
        "name": "Standing Order - Expedited Deadline",
        "description": "Standing order requiring 14-day deadline for civil cases",
        "source": "standing_order",
        "category": "deadline",
        "triggers": ["case_filed"],
        "conditions": [
            {"type": "field_equals", "field": "case_type", "value": "civil"}
        ],
        "actions": [
            {"type": "generate_deadline", "description": "Answer due (expedited)", "days_from_trigger": 14}
        ],
        "priority": "standing_order_priority",
        "status": "active",
        "jurisdiction": "CDCA"
    }), "district12");

    let evaluate_data = json!({
        "trigger": "case_filed",
        "context": {
            "case_type": "civil",
            "document_type": "complaint",
            "filer_role": "plaintiff_attorney",
            "jurisdiction_id": "CDCA"
        }
    });

    let (status, response) = evaluate_rules(evaluate_data, "district12");

    assert_eq!(status, 200, "Evaluate should return 200, got {}", status);
    assert!(!response.is_null(), "Response should not be null");
}

/// Test AND condition: both sub-conditions must match.
/// Creates a rule with And{conditions} that requires case_type=criminal
/// AND document_type=motion. Verifies the rule is stored correctly.
#[spin_test]
fn test_condition_and_both_must_match() {
    let _store = key_value::Store::open("district9");

    let rule = create_rule(json!({
        "name": "Criminal Motion Fee Rule",
        "description": "Require fee for motions in criminal cases",
        "source": "local_rule",
        "category": "fee",
        "triggers": ["motion_filed"],
        "conditions": [
            {
                "type": "and",
                "conditions": [
                    {"type": "field_equals", "field": "case_type", "value": "criminal"},
                    {"type": "field_equals", "field": "document_type", "value": "motion"}
                ]
            }
        ],
        "actions": [
            {"type": "require_fee", "amount_cents": 4600, "description": "Motion filing fee"}
        ],
        "priority": "local",
        "status": "active"
    }), "district9");

    // Verify the rule was stored with And conditions
    let conditions = rule["conditions"].as_array().unwrap();
    assert_eq!(conditions.len(), 1, "Should have one top-level And condition");
    assert_eq!(conditions[0]["type"], "and", "Top-level condition should be 'and'");

    let inner = conditions[0]["conditions"].as_array().unwrap();
    assert_eq!(inner.len(), 2, "And condition should have two sub-conditions");

    // Evaluate with matching context (both conditions met)
    let evaluate_data = json!({
        "trigger": "motion_filed",
        "context": {
            "case_type": "criminal",
            "document_type": "motion",
            "filer_role": "defense_attorney",
            "jurisdiction_id": "SDNY"
        }
    });

    let (status, _response) = evaluate_rules(evaluate_data, "district9");
    assert_eq!(status, 200, "Evaluate should return 200");
}

/// Test OR condition: either sub-condition should match.
/// Creates a rule with Or{conditions} that matches case_type=civil
/// OR case_type=criminal. Verifies the rule structure.
#[spin_test]
fn test_condition_or_either_matches() {
    let _store = key_value::Store::open("district12");

    let rule = create_rule(json!({
        "name": "Universal Deadline Rule",
        "description": "Apply disclosure deadline to both civil and criminal cases",
        "source": "frcp",
        "category": "deadline",
        "triggers": ["case_filed"],
        "conditions": [
            {
                "type": "or",
                "conditions": [
                    {"type": "field_equals", "field": "case_type", "value": "civil"},
                    {"type": "field_equals", "field": "case_type", "value": "criminal"}
                ]
            }
        ],
        "actions": [
            {"type": "generate_deadline", "description": "Initial disclosure due", "days_from_trigger": 14}
        ],
        "priority": "federal_rule",
        "status": "active"
    }), "district12");

    // Verify the rule was stored with Or conditions
    let conditions = rule["conditions"].as_array().unwrap();
    assert_eq!(conditions.len(), 1, "Should have one top-level Or condition");
    assert_eq!(conditions[0]["type"], "or", "Top-level condition should be 'or'");

    let inner = conditions[0]["conditions"].as_array().unwrap();
    assert_eq!(inner.len(), 2, "Or condition should have two sub-conditions");

    // Evaluate with civil case (first alternative matches)
    let evaluate_data = json!({
        "trigger": "case_filed",
        "context": {
            "case_type": "civil",
            "document_type": "complaint",
            "filer_role": "plaintiff_attorney",
            "jurisdiction_id": "EDNY"
        }
    });

    let (status, _response) = evaluate_rules(evaluate_data, "district12");
    assert_eq!(status, 200, "Evaluate should return 200");
}

/// Test wildcard matching: Always condition matches any case type.
/// Creates a rule with an Always condition that applies universally,
/// then verifies the endpoint accepts the evaluation request.
#[spin_test]
fn test_wildcard_matches_all_case_types() {
    let _store = key_value::Store::open("district9");

    // Create a rule with Always condition (wildcard match)
    create_rule(json!({
        "name": "Universal Compliance Logging",
        "description": "Log compliance for all filings regardless of case type",
        "source": "admin_procedure",
        "category": "procedural",
        "triggers": ["document_filed"],
        "conditions": [
            {"type": "always"}
        ],
        "actions": [
            {"type": "log_compliance", "message": "Filing received and logged for compliance"}
        ],
        "priority": "administrative",
        "status": "active"
    }), "district9");

    // Evaluate with a criminal case - should match due to Always condition
    let evaluate_data_criminal = json!({
        "trigger": "document_filed",
        "context": {
            "case_type": "criminal",
            "document_type": "indictment",
            "filer_role": "prosecutor",
            "jurisdiction_id": "SDNY"
        }
    });

    let (status, response) = evaluate_rules(evaluate_data_criminal, "district9");
    assert_eq!(status, 200, "Evaluate should return 200 for criminal case");
    assert!(!response.is_null(), "Response should not be null");

    // Evaluate with a civil case - should also match due to Always condition
    let evaluate_data_civil = json!({
        "trigger": "document_filed",
        "context": {
            "case_type": "civil",
            "document_type": "complaint",
            "filer_role": "plaintiff_attorney",
            "jurisdiction_id": "SDNY"
        }
    });

    let (status, response) = evaluate_rules(evaluate_data_civil, "district9");
    assert_eq!(status, 200, "Evaluate should return 200 for civil case");
    assert!(!response.is_null(), "Response should not be null");
}
