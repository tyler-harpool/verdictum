//! Criminal case CREATE endpoint tests
//!
//! Tests for POST /api/cases endpoint as documented in Utoipa

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to make POST request to create case
fn create_case_request(case_data: Value, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/cases")).unwrap();

    // Set body
    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    // Perform request
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
fn test_create_case_success() {
    // Open the test district store
    let _store = key_value::Store::open("district9");

    let case_data = json!({
        "title": "Bank Fraud Investigation",
        "description": "Investigation into fraudulent banking activities involving multiple suspects",
        "crimeType": "financial_fraud",
        "assignedJudge": "Judge Martinez",
        "location": "San Francisco, CA"
    });

    let (status, response) = create_case_request(case_data, "district9");

    // Should return 201 for successful creation
    assert_eq!(status, 201, "Create case should return 201, got {}", status);

    // Response should contain case ID and location header
    assert!(response.get("id").is_some(), "Response should contain case ID");
    assert!(response.get("case_number").is_some(), "Response should contain case number");

    // Verify required fields are present
    assert_eq!(response["title"], "Bank Fraud Investigation");
    assert_eq!(response["crime_type"], "financial_fraud");
    assert_eq!(response["assigned_judge"], "Judge Martinez");
    assert_eq!(response["location"], "San Francisco, CA");
    assert_eq!(response["status"], "open");
    assert_eq!(response["priority"], "medium");
}

#[spin_test]
fn test_create_case_all_crime_types() {
    let _store = key_value::Store::open("district9");

    let crime_types = vec![
        "violent_crime",
        "property_crime",
        "white_collar_crime",
        "drug_crime",
        "cybercrime",
        "financial_fraud",
        "organized_crime",
        "terrorism",
        "public_order",
        "other"
    ];

    for crime_type in crime_types {
        let case_data = json!({
            "title": format!("Test {} Case", crime_type),
            "description": format!("Testing case creation for {}", crime_type),
            "crimeType": crime_type,
            "assignedJudge": "Judge Test",
            "location": "Test Location"
        });

        let (status, response) = create_case_request(case_data, "district9");

        assert_eq!(status, 201, "Should create case for crime type: {}", crime_type);
        assert_eq!(response["crime_type"], crime_type);
    }
}

#[spin_test]
fn test_create_case_missing_required_field_title() {
    let _store = key_value::Store::open("district9");

    let case_data = json!({
        // Missing title
        "description": "Case without title",
        "crimeType": "other",
        "assignedJudge": "Judge Test",
        "location": "Test Location"
    });

    let (status, response) = create_case_request(case_data, "district9");

    assert_eq!(status, 400, "Should return 400 for missing title");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("title") || body_str.contains("required"),
        "Error should mention missing title field"
    );
}

#[spin_test]
fn test_create_case_empty_title() {
    let _store = key_value::Store::open("district12");

    let case_data = json!({
        "title": "",  // Empty title
        "description": "Case with empty title",
        "crimeType": "other",
        "assignedJudge": "Judge Test",
        "location": "Test Location"
    });

    let (status, response) = create_case_request(case_data, "district12");

    assert_eq!(status, 400, "Should return 400 for empty title");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("title") || body_str.contains("empty"),
        "Error should mention empty title"
    );
}

#[spin_test]
fn test_create_case_whitespace_only_title() {
    let _store = key_value::Store::open("district12");

    let case_data = json!({
        "title": "   ",  // Whitespace only title
        "description": "Case with whitespace-only title",
        "crimeType": "other",
        "assignedJudge": "Judge Test",
        "location": "Test Location"
    });

    let (status, response) = create_case_request(case_data, "district12");

    assert_eq!(status, 400, "Should return 400 for whitespace-only title");

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("title") || body_str.contains("empty"),
        "Error should mention empty title"
    );
}

#[spin_test]
fn test_create_case_missing_description() {
    let _store = key_value::Store::open("district9");

    let case_data = json!({
        "title": "Case Without Description",
        // Missing description
        "crimeType": "other",
        "assignedJudge": "Judge Test",
        "location": "Test Location"
    });

    let (status, response) = create_case_request(case_data, "district9");

    assert_eq!(status, 400, "Should return 400 for missing description");
}

#[spin_test]
fn test_create_case_invalid_crime_type() {
    let _store = key_value::Store::open("district12");

    let case_data = json!({
        "title": "Invalid Crime Type Case",
        "description": "Testing invalid crime type",
        "crimeType": "invalid_crime_type",  // Invalid crime type
        "assignedJudge": "Judge Test",
        "location": "Test Location"
    });

    let (status, response) = create_case_request(case_data, "district12");

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid crime type, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("crime") || body_str.contains("invalid"),
        "Error should mention invalid crime type"
    );
}

#[spin_test]
fn test_create_case_requires_district_header() {
    // Create request WITHOUT district header
    let headers = Headers::new();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/cases")).unwrap();

    let case_data = json!({
        "title": "No District Case",
        "description": "Case without district header",
        "crimeType": "other",
        "assignedJudge": "Judge Test",
        "location": "Test Location"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_create_case_district9_vs_district12_isolation() {
    // Create stores for both districts
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    let case_data = json!({
        "title": "District Isolation Test",
        "description": "Testing district data isolation",
        "crimeType": "other",
        "assignedJudge": "Judge Test",
        "location": "Test Location"
    });

    // Create same case in both districts
    let (status9, response9) = create_case_request(case_data.clone(), "district9");
    let (status12, response12) = create_case_request(case_data, "district12");

    // Both should succeed
    assert_eq!(status9, 201, "Should create case in district9");
    assert_eq!(status12, 201, "Should create case in district12");

    // Cases should have different IDs (isolated)
    assert_ne!(
        response9["id"], response12["id"],
        "Cases in different districts should have different IDs"
    );
}

#[spin_test]
fn test_create_case_with_complex_description() {
    let _store = key_value::Store::open("district9");

    let case_data = json!({
        "title": "Complex Multi-Defendant Case",
        "description": "This is a complex case involving multiple defendants, witnesses, and evidence. The case includes financial records, digital evidence, and witness testimony spanning several months of investigation.",
        "crimeType": "organized_crime",
        "assignedJudge": "Judge Rodriguez",
        "location": "Los Angeles, CA"
    });

    let (status, response) = create_case_request(case_data, "district9");

    assert_eq!(status, 201, "Should handle complex description");
    assert!(
        response["description"].as_str().unwrap().len() > 100,
        "Should preserve long description"
    );
}

#[spin_test]
fn test_create_case_with_special_characters() {
    let _store = key_value::Store::open("district12");

    let case_data = json!({
        "title": "Case with Special Characters: @#$%^&*()",
        "description": "Testing case with unicode: 测试, émojis: 🏛️⚖️, and symbols: §±≠",
        "crimeType": "cybercrime",
        "assignedJudge": "Judge O'Connor-Smith",
        "location": "San José, CA"
    });

    let (status, response) = create_case_request(case_data, "district12");

    assert_eq!(status, 201, "Should handle special characters");
    assert!(
        response["title"].as_str().unwrap().contains("@#$%^&*()"),
        "Should preserve special characters in title"
    );
    assert!(
        response["location"].as_str().unwrap().contains("José"),
        "Should preserve unicode characters"
    );
}

#[spin_test]
fn test_create_case_response_contains_timestamps() {
    let _store = key_value::Store::open("district9");

    let case_data = json!({
        "title": "Timestamp Test Case",
        "description": "Testing timestamp fields in response",
        "crimeType": "other",
        "assignedJudge": "Judge Time",
        "location": "Time City"
    });

    let (status, response) = create_case_request(case_data, "district9");

    assert_eq!(status, 201);

    // Should have timestamps
    assert!(response.get("opened_at").is_some(), "Should have opened_at timestamp");
    assert!(response.get("updated_at").is_some(), "Should have updated_at timestamp");
    assert!(response["closed_at"].is_null(), "Should not have closed_at for new case");

    // Should have empty arrays for new case
    assert!(response["defendants"].is_array(), "Should have defendants array");
    assert!(response["evidence"].is_array(), "Should have evidence array");
    assert_eq!(response["defendants"].as_array().unwrap().len(), 0);
    assert_eq!(response["evidence"].as_array().unwrap().len(), 0);
    assert_eq!(response["notes_count"], 0);
}

#[spin_test]
fn test_create_case_malformed_json() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/cases")).unwrap();

    // Malformed JSON
    let malformed_json = r#"{"title": "Test", "description": "Test", "crimeType": "other", "assignedJudge": "Judge", "location": "Test""#;

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(malformed_json.as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 400, "Should return 400 for malformed JSON");
}