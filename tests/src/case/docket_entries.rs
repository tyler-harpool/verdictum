//! Docket entry tests
//!
//! Tests for adding and retrieving docket entries on criminal cases.

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
        "title": "Docket Test Case",
        "description": "Case for docket entry testing",
        "crimeType": "fraud",
        "districtCode": "SDNY",
        "location": "New York, NY"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap()).unwrap();
    body["id"].as_str().unwrap().to_string()
}

/// Helper to add a docket entry
fn add_docket_entry(case_id: &str, entry_type: &str, description: &str, filed_by: Option<&str>, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/docket", case_id))).unwrap();

    let mut data = json!({
        "entryType": entry_type,
        "description": description
    });
    if let Some(filer) = filed_by {
        data["filedBy"] = json!(filer);
    }

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap_or_default()).unwrap_or(json!(null));
    (status, body)
}

/// Helper to get docket entries
fn get_docket_entries(case_id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/docket", case_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap_or_default()).unwrap_or(json!(null));
    (status, body)
}

#[spin_test]
fn test_add_docket_entry_success() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, response) = add_docket_entry(&case_id, "motion", "Motion to Dismiss", Some("Defense Counsel"), "district9");

    assert_eq!(status, 200, "Should return 200 for successful docket entry");
    assert!(response["docketEntriesCount"].as_u64().unwrap() >= 1);
}

#[spin_test]
fn test_add_multiple_docket_entries_auto_increment() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    add_docket_entry(&case_id, "complaint", "Criminal Complaint", Some("AUSA"), "district12");
    add_docket_entry(&case_id, "indictment", "Grand Jury Indictment", Some("Grand Jury"), "district12");
    add_docket_entry(&case_id, "motion", "Motion for Discovery", Some("Defense"), "district12");

    let (status, entries) = get_docket_entries(&case_id, "district12");
    assert_eq!(status, 200);

    let arr = entries.as_array().unwrap();
    assert_eq!(arr.len(), 3);

    // Verify sequential numbering
    assert_eq!(arr[0]["entryNumber"], 1);
    assert_eq!(arr[1]["entryNumber"], 2);
    assert_eq!(arr[2]["entryNumber"], 3);
}

#[spin_test]
fn test_get_docket_entries_empty() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, entries) = get_docket_entries(&case_id, "district9");
    assert_eq!(status, 200);
    assert_eq!(entries.as_array().unwrap().len(), 0);
}

#[spin_test]
fn test_get_docket_entries_sorted() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    add_docket_entry(&case_id, "complaint", "First", None, "district12");
    add_docket_entry(&case_id, "order", "Second", None, "district12");

    let (status, entries) = get_docket_entries(&case_id, "district12");
    assert_eq!(status, 200);

    let arr = entries.as_array().unwrap();
    let num1 = arr[0]["entryNumber"].as_u64().unwrap();
    let num2 = arr[1]["entryNumber"].as_u64().unwrap();
    assert!(num1 < num2, "Entries should be sorted by entry number");
}

#[spin_test]
fn test_add_docket_entry_empty_description() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, _) = add_docket_entry(&case_id, "motion", "", None, "district9");
    assert_eq!(status, 400, "Should return 400 for empty description");
}

#[spin_test]
fn test_add_docket_entry_nonexistent_case() {
    let _store = key_value::Store::open("district12");
    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, _) = add_docket_entry(fake_id, "motion", "Test", None, "district12");
    assert_eq!(status, 404, "Should return 404 for nonexistent case");
}

#[spin_test]
fn test_docket_entry_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    let case_id = create_test_case("district9");

    let (status, _) = add_docket_entry(&case_id, "motion", "Cross-district test", None, "district12");
    assert_eq!(status, 404, "Should not find district9 case from district12");
}

#[spin_test]
fn test_case_response_includes_docket_entries_count() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (_, response) = add_docket_entry(&case_id, "motion", "Test entry", None, "district9");
    assert!(response.get("docketEntriesCount").is_some(), "CaseResponse should include docketEntriesCount");
    assert_eq!(response["docketEntriesCount"], 1);
}
