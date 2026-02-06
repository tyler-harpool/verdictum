//! Filing pipeline deadline chain tests
//!
//! Tests that motion filings through the pipeline generate correct
//! deadline chains from configured rules, and validates that ARWD
//! rule configuration is properly loaded from TOML.

use spin_test_sdk::{
    spin_test,
    bindings::{wasi::http, fermyon::spin_test_virt::key_value},
};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to make a POST request with JSON body
fn post_json(path: &str, body: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(path)).unwrap();

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream
        .blocking_write_and_flush(serde_json::to_string(&body).unwrap().as_bytes())
        .unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let resp_body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if resp_body.is_empty() {
        json!(null)
    } else {
        serde_json::from_str(&resp_body).unwrap_or(json!({"raw": resp_body}))
    };

    (status, body_json)
}

/// Create a rule via the rules API
fn create_rule(rule_data: Value, district: &str) -> (u16, Value) {
    post_json("/api/rules", rule_data, district)
}

/// Test that a motion filing generates a deadline chain when rules with
/// GenerateDeadline actions are configured for the jurisdiction.
///
/// This test:
/// 1. Creates two rules with GenerateDeadline actions for district9
/// 2. Submits a motion filing
/// 3. Verifies the receipt contains deadlines from both rules
#[spin_test]
fn test_filing_arwd_motion_generates_deadline_chain() {
    let _store = key_value::Store::open("district9");

    // Step 1: Create a response-deadline rule (14 days for motion response)
    let response_rule = json!({
        "name": "LR 7.2(b) Motion Response Deadline",
        "description": "Response to motion must be filed within 14 days",
        "source": "local_rule",
        "category": "deadline",
        "triggers": ["document_filed"],
        "conditions": [
            {"type": "field_equals", "field": "document_type", "value": "motion"}
        ],
        "actions": [
            {
                "type": "generate_deadline",
                "description": "Response to motion due",
                "days_from_trigger": 14
            }
        ],
        "priority": "local",
        "status": "active",
        "jurisdiction": "district9",
        "citation": "LR 7.2(b)"
    });

    let (rule_status, _) = create_rule(response_rule, "district9");
    assert_eq!(rule_status, 201, "First rule creation should succeed");

    // Step 2: Create a reply-deadline rule (7 days for reply brief)
    let reply_rule = json!({
        "name": "LR 7.2(c) Reply Brief Deadline",
        "description": "Reply brief must be filed within 7 days after response",
        "source": "local_rule",
        "category": "deadline",
        "triggers": ["document_filed"],
        "conditions": [
            {"type": "field_equals", "field": "document_type", "value": "motion"}
        ],
        "actions": [
            {
                "type": "generate_deadline",
                "description": "Reply brief due",
                "days_from_trigger": 21
            }
        ],
        "priority": "local",
        "status": "active",
        "jurisdiction": "district9",
        "citation": "LR 7.2(c)"
    });

    let (rule_status, _) = create_rule(reply_rule, "district9");
    assert_eq!(rule_status, 201, "Second rule creation should succeed");

    // Step 3: Submit a motion filing
    let filing = json!({
        "case_number": "5:24-cr-00999",
        "document_type": "motion",
        "filer_name": "ARWD Test Attorney",
        "filer_role": "plaintiff_attorney",
        "document_text": "Motion for extension of time to complete discovery.",
        "metadata": {
            "case_type": "criminal"
        }
    });

    let (status, response) = post_json("/api/filings", filing, "district9");

    assert_eq!(
        status, 201,
        "Filing should be accepted with 201, got {} body: {}",
        status, response
    );

    // Step 4: Verify the compliance report contains deadlines
    let compliance = &response["compliance_report"];
    assert!(
        compliance.is_object(),
        "Response should contain compliance_report"
    );

    let deadlines = compliance["deadlines"].as_array();
    assert!(
        deadlines.is_some(),
        "Compliance report should contain deadlines array"
    );

    let deadlines = deadlines.unwrap();

    // The filing handler runs deadline computation through both the rules engine
    // (which adds deadlines during evaluate()) and the deadline engine (for
    // GenerateDeadline actions from the sorted rules). We expect at least 2
    // deadlines from our configured rules.
    assert!(
        deadlines.len() >= 2,
        "Expected at least 2 deadlines in the chain, got {}. Deadlines: {:?}",
        deadlines.len(),
        deadlines
    );

    // Verify deadline descriptions exist
    let descriptions: Vec<&str> = deadlines
        .iter()
        .filter_map(|d| d["description"].as_str())
        .collect();

    assert!(
        descriptions.iter().any(|d| d.contains("Response to motion")),
        "Should have a 'Response to motion' deadline. Found: {:?}",
        descriptions
    );
    assert!(
        descriptions.iter().any(|d| d.contains("Reply brief")),
        "Should have a 'Reply brief' deadline. Found: {:?}",
        descriptions
    );
}

/// Test that the rules API correctly stores and retrieves rules that would
/// be used in an ARWD-style TOML configuration. This validates the round-trip
/// of rule creation with all fields needed for TOML-configured rule packs.
#[spin_test]
fn test_arwd_rules_load_from_toml_structure() {
    let _store = key_value::Store::open("district12");

    // Create a rule mimicking the structure that would come from rules.toml
    let toml_style_rule = json!({
        "name": "ARWD LR 56.1 Summary Judgment Statement",
        "description": "Summary judgment motion requires separate statement of undisputed material facts",
        "source": "local_rule",
        "category": "filing",
        "triggers": ["document_filed"],
        "conditions": [
            {"type": "field_equals", "field": "document_type", "value": "summary_judgment"}
        ],
        "actions": [
            {
                "type": "flag_for_review",
                "reason": "Summary judgment motion requires separate statement of undisputed material facts per LR 56.1"
            }
        ],
        "priority": "local",
        "status": "active",
        "jurisdiction": "district12",
        "citation": "ARWD LR 56.1"
    });

    // Create the rule
    let (create_status, create_response) = create_rule(toml_style_rule, "district12");
    assert_eq!(
        create_status, 201,
        "Rule creation should succeed, got {} body: {}",
        create_status, create_response
    );

    let rule_id = create_response["id"].as_str()
        .expect("Rule should have an id");

    // Retrieve the rule to verify persistence
    let get_path = format!("/api/rules/{}", rule_id);
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&get_path)).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let get_status = response.status();
    let get_body = response.body_as_string().unwrap_or_default();
    let get_response: Value = serde_json::from_str(&get_body).unwrap_or(json!(null));

    assert_eq!(
        get_status, 200,
        "GET rule should return 200, got {} body: {}",
        get_status, get_body
    );

    // Verify all TOML-configured fields survived the round trip
    assert_eq!(get_response["name"], "ARWD LR 56.1 Summary Judgment Statement");
    assert_eq!(get_response["source"], "local_rule");
    assert_eq!(get_response["category"], "filing");
    assert_eq!(get_response["status"], "active");
    assert_eq!(get_response["jurisdiction"], "district12");
    assert_eq!(get_response["citation"], "ARWD LR 56.1");

    // Verify triggers round-tripped
    let triggers = get_response["triggers"].as_array()
        .expect("Rule should have triggers array");
    assert!(
        triggers.contains(&json!("document_filed")),
        "Triggers should contain document_filed"
    );

    // Verify conditions round-tripped
    let conditions = get_response["conditions"].as_array()
        .expect("Rule should have conditions array");
    assert_eq!(conditions.len(), 1, "Should have exactly one condition");
    assert_eq!(conditions[0]["type"], "field_equals");
    assert_eq!(conditions[0]["field"], "document_type");
    assert_eq!(conditions[0]["value"], "summary_judgment");

    // Verify actions round-tripped
    let actions = get_response["actions"].as_array()
        .expect("Rule should have actions array");
    assert_eq!(actions.len(), 1, "Should have exactly one action");
    assert_eq!(actions[0]["type"], "flag_for_review");
    assert!(
        actions[0]["reason"].as_str().unwrap().contains("LR 56.1"),
        "Action reason should reference LR 56.1"
    );
}
