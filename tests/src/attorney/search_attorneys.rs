//! Attorney SEARCH endpoint tests
//!
//! Tests for GET /api/attorneys/search endpoint

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create an attorney with specific attributes
fn create_attorney_with_details(
    district: &str,
    bar_number: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    firm_name: Option<&str>,
) -> String {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let mut attorney_data = json!({
        "bar_number": bar_number,
        "first_name": first_name,
        "last_name": last_name,
        "email": email,
        "phone": "555-0100",
        "address": {
            "street1": "123 Search St",
            "city": "Search City",
            "state": "SC",
            "zip_code": "12345",
            "country": "USA"
        }
    });

    if let Some(firm) = firm_name {
        attorney_data["firm_name"] = json!(firm);
    }

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    body_json["id"].as_str().unwrap().to_string()
}

/// Helper to search attorneys
fn search_attorneys(query: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/search?q={}", query))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!({"data": [], "meta": {}})
    } else {
        serde_json::from_str(&body).unwrap_or(json!({"data": [], "meta": {}}))
    };

    (status, body_json)
}

/// Helper to delete an attorney
fn delete_attorney(id: &str, district: &str) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Delete).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/{}", id))).unwrap();

    spin_test_sdk::perform_request(request);
}

#[spin_test]
fn test_search_attorneys_by_first_name() {
    let _store = key_value::Store::open("district9");

    // Create attorneys with unique names
    let id1 = create_attorney_with_details(
        "district9",
        &format!("SEARCH1{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Alexander",
        "Hamilton",
        "alex@law.com",
        Some("Founding Law"),
    );

    let id2 = create_attorney_with_details(
        "district9",
        &format!("SEARCH2{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Alexandra",
        "Madison",
        "alexa@law.com",
        Some("Madison & Associates"),
    );

    let id3 = create_attorney_with_details(
        "district9",
        &format!("SEARCH3{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Benjamin",
        "Franklin",
        "ben@law.com",
        None,
    );

    // Search for "Alex" - should match Alexander and Alexandra
    let (status, results) = search_attorneys("Alex", "district9");

    assert_eq!(status, 200, "Search should return 200");
    assert!(results["data"].is_array(), "Results should have data array");

    let results_array = results["data"].as_array().unwrap();
    assert_eq!(results_array.len(), 2, "Should find 2 attorneys with 'Alex' in name");

    // Verify the correct attorneys were found
    let ids: Vec<String> = results_array
        .iter()
        .filter_map(|a| a["id"].as_str().map(String::from))
        .collect();

    assert!(ids.contains(&id1), "Should find Alexander");
    assert!(ids.contains(&id2), "Should find Alexandra");
    assert!(!ids.contains(&id3), "Should not find Benjamin");

    // Clean up
    delete_attorney(&id1, "district9");
    delete_attorney(&id2, "district9");
    delete_attorney(&id3, "district9");
}

#[spin_test]
fn test_search_attorneys_by_last_name() {
    let _store = key_value::Store::open("district9");

    // Create attorneys
    let id1 = create_attorney_with_details(
        "district9",
        &format!("SEARCH4{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "John",
        "Jefferson",
        "john@law.com",
        None,
    );

    let id2 = create_attorney_with_details(
        "district9",
        &format!("SEARCH5{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Jane",
        "Jefferson",
        "jane@law.com",
        None,
    );

    // Search for "Jefferson"
    let (status, results) = search_attorneys("Jefferson", "district9");

    assert_eq!(status, 200);
    let results_array = results["data"].as_array().unwrap();
    assert_eq!(results_array.len(), 2, "Should find both Jeffersons");

    // Clean up
    delete_attorney(&id1, "district9");
    delete_attorney(&id2, "district9");
}

#[spin_test]
fn test_search_attorneys_by_bar_number() {
    let _store = key_value::Store::open("district12");

    let unique_bar = format!("NY{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());

    let id = create_attorney_with_details(
        "district12",
        &unique_bar,
        "Test",
        "Attorney",
        "test@law.com",
        None,
    );

    // Search by partial bar number
    let (status, results) = search_attorneys("NY", "district12");

    assert_eq!(status, 200);
    let results_array = results["data"].as_array().unwrap();

    // Should find at least our attorney (might find others from other tests)
    let found = results_array.iter().any(|a| a["id"].as_str() == Some(&id));
    assert!(found, "Should find attorney by bar number prefix");

    // Clean up
    delete_attorney(&id, "district12");
}

#[spin_test]
fn test_search_attorneys_by_email() {
    let _store = key_value::Store::open("district12");

    let id = create_attorney_with_details(
        "district12",
        &format!("EMAIL{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Email",
        "Test",
        "unique@example.com",
        None,
    );

    // Search by email domain
    let (status, results) = search_attorneys("@example.com", "district12");

    assert_eq!(status, 200);
    let results_array = results["data"].as_array().unwrap();

    let found = results_array.iter().any(|a| a["id"].as_str() == Some(&id));
    assert!(found, "Should find attorney by email");

    // Clean up
    delete_attorney(&id, "district12");
}

#[spin_test]
fn test_search_attorneys_by_firm_name() {
    let _store = key_value::Store::open("district9");

    let id1 = create_attorney_with_details(
        "district9",
        &format!("FIRM1{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Partner",
        "One",
        "p1@law.com",
        Some("SuperUnique Law Firm"),
    );

    let id2 = create_attorney_with_details(
        "district9",
        &format!("FIRM2{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Partner",
        "Two",
        "p2@law.com",
        Some("SuperUnique Law Firm"),
    );

    // Search by firm name
    let (status, results) = search_attorneys("SuperUnique", "district9");

    assert_eq!(status, 200);
    let results_array = results["data"].as_array().unwrap();
    assert_eq!(results_array.len(), 2, "Should find both partners from the firm");

    // Clean up
    delete_attorney(&id1, "district9");
    delete_attorney(&id2, "district9");
}

#[spin_test]
fn test_search_attorneys_case_insensitive() {
    let _store = key_value::Store::open("district12");

    let id = create_attorney_with_details(
        "district12",
        &format!("CASE{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "CaseSensitive",
        "TestName",
        "case@test.com",
        None,
    );

    // Search with different cases
    let test_cases = vec!["casesensitive", "CASESENSITIVE", "CaSeSeNsItIvE"];

    for query in test_cases {
        let (status, results) = search_attorneys(query, "district12");
        assert_eq!(status, 200);

        let results_array = results["data"].as_array().unwrap();
        let found = results_array.iter().any(|a| a["id"].as_str() == Some(&id));
        assert!(found, "Search should be case-insensitive for query: {}", query);
    }

    // Clean up
    delete_attorney(&id, "district12");
}

#[spin_test]
fn test_search_attorneys_empty_query() {
    let _store = key_value::Store::open("district9");

    // Create an attorney
    let id = create_attorney_with_details(
        "district9",
        &format!("EMPTY{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Test",
        "Empty",
        "empty@test.com",
        None,
    );

    // Search with empty query - should return all attorneys
    let (status, results) = search_attorneys("", "district9");

    assert_eq!(status, 200);
    assert!(results["data"].is_array());

    // Should return at least the attorney we created
    let results_array = results["data"].as_array().unwrap();
    assert!(results_array.len() >= 1, "Empty query should return all attorneys");

    // Clean up
    delete_attorney(&id, "district9");
}

#[spin_test]
fn test_search_attorneys_no_matches() {
    let _store = key_value::Store::open("district12");

    // Search for something that won't match
    let (status, results) = search_attorneys("NonExistentUniqueQueryXYZ123", "district12");

    assert_eq!(status, 200, "Should still return 200 even with no matches");
    assert!(results["data"].is_array());

    let results_array = results["data"].as_array().unwrap();
    assert_eq!(results_array.len(), 0, "Should return empty array when no matches");
}

#[spin_test]
fn test_search_attorneys_requires_district_header() {
    // Try to search without district header
    let headers = Headers::new();
    // Intentionally NOT adding X-Court-District header

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/attorneys/search?q=test")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(
        response.status(), 400,
        "Should return 400 when district header is missing"
    );
}

#[spin_test]
fn test_search_attorneys_partial_matches() {
    let _store = key_value::Store::open("district9");

    let id = create_attorney_with_details(
        "district9",
        &format!("PARTIAL{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "Christopher",
        "Washington",
        "cwash@law.com",
        Some("Washington Legal Group"),
    );

    // Test various partial matches
    let partial_queries = vec!["Chris", "opher", "Wash", "ington", "Legal"];

    for query in partial_queries {
        let (status, results) = search_attorneys(query, "district9");
        assert_eq!(status, 200);

        let results_array = results["data"].as_array().unwrap();
        let found = results_array.iter().any(|a| a["id"].as_str() == Some(&id));
        assert!(found, "Should find attorney with partial match: {}", query);
    }

    // Clean up
    delete_attorney(&id, "district9");
}