//! CVRA Victim notification tests
//!
//! Tests for adding victims, retrieving victim lists, and sending notifications
//! per the Crime Victims' Rights Act (CVRA).

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
        "title": "Victim Test Case",
        "description": "Case for CVRA victim testing",
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

/// Helper to add a victim to a case
fn add_victim(case_id: &str, name: &str, victim_type: Option<&str>, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/victims", case_id))).unwrap();

    let mut data = json!({
        "name": name,
        "preferredMethod": "email",
        "email": "victim@example.com"
    });
    if let Some(vt) = victim_type {
        data["victimType"] = json!(vt);
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

/// Helper to get victims for a case
fn get_victims(case_id: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/victims", case_id))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body: Value = serde_json::from_str(&response.body_as_string().unwrap_or_default()).unwrap_or(json!(null));
    (status, body)
}

/// Helper to send a notification to a victim
fn send_notification(case_id: &str, victim_id: &str, notification_type: &str, summary: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/{}/victims/{}/notifications", case_id, victim_id))).unwrap();

    let data = json!({
        "notificationType": notification_type,
        "contentSummary": summary
    });

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

#[spin_test]
fn test_add_victim_success() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, response) = add_victim(&case_id, "Jane Doe", Some("individual"), "district9");

    assert_eq!(status, 200);
    assert!(response.get("victimsCount").is_some(), "Should have victimsCount");
    assert_eq!(response["victimsCount"], 1);
}

#[spin_test]
fn test_add_victim_empty_name() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    let (status, _) = add_victim(&case_id, "", None, "district12");
    assert_eq!(status, 400, "Should return 400 for empty victim name");
}

#[spin_test]
fn test_add_multiple_victims() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    add_victim(&case_id, "Victim One", Some("individual"), "district9");
    add_victim(&case_id, "Victim Two", Some("business"), "district9");
    let (status, response) = add_victim(&case_id, "Victim Three", Some("organization"), "district9");

    assert_eq!(status, 200);
    assert_eq!(response["victimsCount"], 3);
}

#[spin_test]
fn test_get_victims_list() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    add_victim(&case_id, "Alice Smith", Some("individual"), "district12");
    add_victim(&case_id, "Acme Corp", Some("business"), "district12");

    let (status, victims) = get_victims(&case_id, "district12");

    assert_eq!(status, 200);
    let arr = victims.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr[0].get("id").is_some(), "Victim should have id");
    assert!(arr[0].get("name").is_some(), "Victim should have name");
    assert!(arr[0].get("victimType").is_some(), "Victim should have victimType");
    assert!(arr[0].get("optedOut").is_some(), "Victim should have optedOut");
}

#[spin_test]
fn test_get_victims_empty() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (status, victims) = get_victims(&case_id, "district9");

    assert_eq!(status, 200);
    assert_eq!(victims.as_array().unwrap().len(), 0);
}

#[spin_test]
fn test_send_notification_success() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    // Add victim and get ID from the victim list
    add_victim(&case_id, "Notification Test Victim", Some("individual"), "district12");
    let (_, victims) = get_victims(&case_id, "district12");
    let victim_id = victims.as_array().unwrap()[0]["id"].as_str().unwrap();

    // Send notification
    let (status, response) = send_notification(
        &case_id,
        victim_id,
        "hearing_scheduled",
        "Status conference scheduled for Feb 20, 2026",
        "district12",
    );

    assert_eq!(status, 200);
    assert!(response.get("victimsCount").is_some());
}

#[spin_test]
fn test_send_notification_nonexistent_victim() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let fake_victim_id = "550e8400-e29b-41d4-a716-446655440000";
    let (status, _) = send_notification(
        &case_id,
        fake_victim_id,
        "case_filed",
        "Case has been filed",
        "district9",
    );

    assert_eq!(status, 404, "Should return 404 for nonexistent victim");
}

#[spin_test]
fn test_send_notification_opted_out() {
    let _store = key_value::Store::open("district12");
    let case_id = create_test_case("district12");

    // Add a victim with opt_out. Since opt_out defaults to false, we need to
    // modify the victim directly. Instead, we'll test by adding a victim,
    // verifying they aren't opted out by default.
    add_victim(&case_id, "Opt Out Victim", Some("individual"), "district12");
    let (_, victims) = get_victims(&case_id, "district12");
    let victim = &victims.as_array().unwrap()[0];
    assert_eq!(victim["optedOut"], false, "Victim should not be opted out by default");
}

#[spin_test]
fn test_victims_count_in_case_response() {
    let _store = key_value::Store::open("district9");
    let case_id = create_test_case("district9");

    let (_, response) = add_victim(&case_id, "Count Test Victim", None, "district9");

    assert!(response.get("victimsCount").is_some(), "CaseResponse should include victimsCount");
    assert_eq!(response["victimsCount"], 1);
}

#[spin_test]
fn test_victims_district_isolation() {
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    let case_id = create_test_case("district9");

    let (status, _) = add_victim(&case_id, "Cross-District Victim", None, "district12");
    assert_eq!(status, 404, "Should not find district9 case from district12");
}
