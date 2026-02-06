//! Filing pipeline submission endpoint tests
//!
//! Tests for POST /api/filings, POST /api/filings/validate,
//! and GET /api/filings/jurisdictions

use spin_test_sdk::{
    spin_test,
    bindings::{wasi::http, fermyon::spin_test_virt::key_value},
};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to make a POST request with a JSON body
fn post_request(path: &str, body: Value, district: Option<&str>) -> (u16, Value) {
    let headers = Headers::new();
    if let Some(d) = district {
        headers.append(&"X-Court-District".to_string(), d.as_bytes()).unwrap();
    }
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

/// Helper to make a GET request
fn get_request(path: &str) -> (u16, Value) {
    let headers = Headers::new();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(path)).unwrap();

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

#[spin_test]
fn test_filing_submission_clean_passes() {
    let _store = key_value::Store::open("district9");

    let filing = json!({
        "case_number": "5:24-cr-00123",
        "document_type": "motion",
        "filer_name": "Jane Attorney",
        "filer_role": "plaintiff_attorney",
        "document_text": "This motion requests summary judgment based on the evidence presented.",
        "metadata": {
            "case_type": "criminal"
        }
    });

    let (status, response) = post_request("/api/filings", filing, Some("district9"));

    assert_eq!(
        status, 201,
        "Clean filing should return 201, got {} with body: {}",
        status, response
    );
    assert!(
        response.get("filing_id").is_some(),
        "Response should contain filing_id"
    );
    assert_eq!(response["case_number"], "5:24-cr-00123");
    assert_eq!(response["document_type"], "motion");
    assert!(
        response.get("compliance_report").is_some(),
        "Response should contain compliance_report"
    );
    assert!(
        response.get("nef").is_some(),
        "Response should contain NEF record"
    );
}

#[spin_test]
fn test_filing_with_unredacted_ssn_rejected() {
    let _store = key_value::Store::open("district9");

    let filing = json!({
        "case_number": "5:24-cr-00456",
        "document_type": "motion",
        "filer_name": "John Lawyer",
        "filer_role": "defendant_attorney",
        "document_text": "The defendant's SSN is 123-45-6789 and was used for identification.",
        "metadata": {
            "case_type": "criminal"
        }
    });

    let (status, response) = post_request("/api/filings", filing, Some("district9"));

    assert_eq!(
        status, 422,
        "Filing with unredacted SSN should return 422, got {} with body: {}",
        status, response
    );
    // The response should be a PrivacyScanResult with violations
    assert_eq!(
        response["clean"], false,
        "Privacy scan should report document as not clean"
    );
    let violations = response["violations"].as_array();
    assert!(
        violations.is_some() && !violations.unwrap().is_empty(),
        "Should report PII violations"
    );
}

#[spin_test]
fn test_filing_requires_district() {
    let filing = json!({
        "case_number": "5:24-cr-00789",
        "document_type": "brief",
        "filer_name": "Jane Doe",
        "filer_role": "plaintiff_attorney",
        "metadata": {}
    });

    let (status, _response) = post_request("/api/filings", filing, None);

    assert_eq!(
        status, 400,
        "Filing without district header should return 400, got {}",
        status
    );
}

#[spin_test]
fn test_filing_validate_dry_run() {
    let _store = key_value::Store::open("district12");

    let filing = json!({
        "case_number": "3:25-cv-00001",
        "document_type": "complaint",
        "filer_name": "Test Attorney",
        "filer_role": "plaintiff_attorney",
        "document_text": "This is a clean complaint with no PII violations.",
        "metadata": {
            "case_type": "civil"
        }
    });

    let (status, response) = post_request("/api/filings/validate", filing, Some("district12"));

    assert_eq!(
        status, 200,
        "Validate endpoint should return 200 (not 201), got {} with body: {}",
        status, response
    );
    assert!(
        response.get("compliance_report").is_some(),
        "Should contain compliance_report"
    );
    assert!(
        response.get("privacy_clean").is_some(),
        "Should contain privacy_clean flag"
    );
    assert!(
        response.get("would_accept").is_some(),
        "Should contain would_accept flag"
    );
}

#[spin_test]
fn test_list_jurisdictions() {
    let (status, response) = get_request("/api/filings/jurisdictions");

    assert_eq!(
        status, 200,
        "List jurisdictions should return 200, got {} with body: {}",
        status, response
    );

    let jurisdictions = response["jurisdictions"].as_array();
    assert!(
        jurisdictions.is_some(),
        "Response should contain jurisdictions array"
    );

    let jurisdictions = jurisdictions.unwrap();
    assert!(
        !jurisdictions.is_empty(),
        "Jurisdictions list should not be empty"
    );
    assert!(
        jurisdictions.contains(&json!("arwd")),
        "Jurisdictions should include arwd"
    );
    assert!(
        jurisdictions.contains(&json!("sdny")),
        "Jurisdictions should include sdny"
    );
}
