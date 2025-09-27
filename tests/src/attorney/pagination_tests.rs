//! Pagination tests for attorney endpoints
//!
//! Tests for pagination functionality on list and search endpoints

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create multiple test attorneys
fn create_test_attorneys(district: &str, count: usize) -> Vec<String> {
    let mut ids = Vec::new();

    for i in 0..count {
        let headers = Headers::new();
        headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
        headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

        let request = OutgoingRequest::new(headers);
        request.set_method(&Method::Post).unwrap();
        request.set_path_with_query(Some("/api/attorneys")).unwrap();

        let unique_bar = format!("PAGE{}{}", i, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());

        let attorney_data = json!({
            "bar_number": unique_bar,
            "first_name": format!("Attorney{}", i),
            "last_name": "Paginated",
            "email": format!("attorney{}@law.com", i),
            "phone": format!("555-{:04}", i),
            "address": {
                "street1": format!("{} Page St", i),
                "city": "Page City",
                "state": "PC",
                "zip_code": "12345",
                "country": "USA"
            }
        });

        let request_body = request.body().unwrap();
        let stream = request_body.write().unwrap();
        stream.blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes()).unwrap();
        drop(stream);
        http::types::OutgoingBody::finish(request_body, None).unwrap();

        let response = spin_test_sdk::perform_request(request);
        let body = response.body_as_string().unwrap();
        let body_json: Value = serde_json::from_str(&body).unwrap();

        ids.push(body_json["id"].as_str().unwrap().to_string());
    }

    ids
}

/// Helper to delete multiple attorneys
fn delete_attorneys(ids: &[String], district: &str) {
    for id in ids {
        let headers = Headers::new();
        headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

        let request = OutgoingRequest::new(headers);
        request.set_method(&Method::Delete).unwrap();
        request.set_path_with_query(Some(&format!("/api/attorneys/{}", id))).unwrap();

        spin_test_sdk::perform_request(request);
    }
}

/// Helper to list attorneys with pagination
fn list_attorneys_paginated(district: &str, page: usize, limit: usize) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys?page={}&limit={}", page, limit))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!({})
    } else {
        serde_json::from_str(&body).unwrap_or(json!({}))
    };

    (status, body_json)
}

/// Helper to search attorneys with pagination
fn search_attorneys_paginated(district: &str, query: &str, page: usize, limit: usize) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/attorneys/search?q={}&page={}&limit={}", query, page, limit))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!({})
    } else {
        serde_json::from_str(&body).unwrap_or(json!({}))
    };

    (status, body_json)
}

#[spin_test]
fn test_list_attorneys_first_page() {
    let _store = key_value::Store::open("district9");

    // Create 5 attorneys
    let ids = create_test_attorneys("district9", 5);

    // Get first page with limit 3
    let (status, response) = list_attorneys_paginated("district9", 1, 3);

    assert_eq!(status, 200, "Should return 200");

    // Check response structure
    assert!(response["data"].is_array(), "Should have data array");
    assert!(response["meta"].is_object(), "Should have meta object");

    let data = response["data"].as_array().unwrap();
    assert!(data.len() <= 3, "Should return at most 3 items");

    let meta = &response["meta"];
    assert_eq!(meta["page"], 1, "Should be page 1");
    assert_eq!(meta["limit"], 3, "Should have limit 3");
    assert!(meta["total"].as_u64().unwrap() >= 5, "Should have at least 5 total");
    assert!(meta["has_next"].as_bool().unwrap(), "Should have next page");
    assert!(!meta["has_prev"].as_bool().unwrap(), "Should not have previous page");

    // Clean up
    delete_attorneys(&ids, "district9");
}

#[spin_test]
fn test_list_attorneys_second_page() {
    let _store = key_value::Store::open("district12");

    // Create 5 attorneys
    let ids = create_test_attorneys("district12", 5);

    // Get second page with limit 2
    let (status, response) = list_attorneys_paginated("district12", 2, 2);

    assert_eq!(status, 200, "Should return 200");

    let data = response["data"].as_array().unwrap();
    assert!(data.len() <= 2, "Should return at most 2 items");

    let meta = &response["meta"];
    assert_eq!(meta["page"], 2, "Should be page 2");
    assert!(meta["has_prev"].as_bool().unwrap(), "Should have previous page");

    // Clean up
    delete_attorneys(&ids, "district12");
}

#[spin_test]
fn test_list_attorneys_empty_page() {
    let _store = key_value::Store::open("district9");

    // Request a page that doesn't exist (very high page number)
    let (status, response) = list_attorneys_paginated("district9", 1000, 20);

    assert_eq!(status, 200, "Should still return 200");

    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 0, "Should return empty array for non-existent page");

    let meta = &response["meta"];
    assert_eq!(meta["page"], 1000, "Should be page 1000");
}

#[spin_test]
fn test_list_attorneys_default_pagination() {
    let _store = key_value::Store::open("district12");

    // Create 25 attorneys
    let ids = create_test_attorneys("district12", 25);

    // Request without page/limit params (should use defaults)
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), b"district12").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body_json: Value = serde_json::from_str(&response.body_as_string().unwrap()).unwrap();

    let data = body_json["data"].as_array().unwrap();
    assert_eq!(data.len(), 20, "Default limit should be 20");

    let meta = &body_json["meta"];
    assert_eq!(meta["page"], 1, "Default page should be 1");
    assert_eq!(meta["limit"], 20, "Default limit should be 20");

    // Clean up
    delete_attorneys(&ids, "district12");
}

#[spin_test]
fn test_search_attorneys_pagination() {
    let _store = key_value::Store::open("district9");

    // Create attorneys with specific search term
    let mut ids = Vec::new();
    for i in 0..10 {
        let headers = Headers::new();
        headers.append(&"X-Court-District".to_string(), b"district9").unwrap();
        headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

        let request = OutgoingRequest::new(headers);
        request.set_method(&Method::Post).unwrap();
        request.set_path_with_query(Some("/api/attorneys")).unwrap();

        let attorney_data = json!({
            "bar_number": format!("SEARCH{}{}", i, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            "first_name": "Searchable",
            "last_name": format!("Attorney{}", i),
            "email": format!("search{}@law.com", i),
            "phone": "555-0100",
            "address": {
                "street1": "123 Search St",
                "city": "Search City",
                "state": "SC",
                "zip_code": "12345",
                "country": "USA"
            }
        });

        let request_body = request.body().unwrap();
        let stream = request_body.write().unwrap();
        stream.blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes()).unwrap();
        drop(stream);
        http::types::OutgoingBody::finish(request_body, None).unwrap();

        let response = spin_test_sdk::perform_request(request);
        let body_json: Value = serde_json::from_str(&response.body_as_string().unwrap()).unwrap();
        ids.push(body_json["id"].as_str().unwrap().to_string());
    }

    // Search with pagination
    let (status, response) = search_attorneys_paginated("district9", "Searchable", 1, 5);

    assert_eq!(status, 200, "Should return 200");

    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 5, "Should return 5 items");

    let meta = &response["meta"];
    assert_eq!(meta["page"], 1, "Should be page 1");
    assert_eq!(meta["limit"], 5, "Should have limit 5");
    assert_eq!(meta["total"], 10, "Should have 10 total matches");
    assert!(meta["has_next"].as_bool().unwrap(), "Should have next page");

    // Clean up
    delete_attorneys(&ids, "district9");
}

#[spin_test]
fn test_pagination_limit_max() {
    let _store = key_value::Store::open("district12");

    // Try to request more than max limit (100)
    let (status, response) = list_attorneys_paginated("district12", 1, 200);

    assert_eq!(status, 200, "Should return 200");

    let meta = &response["meta"];
    assert_eq!(meta["limit"], 100, "Limit should be capped at 100");
}

#[spin_test]
fn test_pagination_invalid_params() {
    let _store = key_value::Store::open("district9");

    // Test with invalid page (0 should default to 1)
    let (status, response) = list_attorneys_paginated("district9", 0, 10);

    assert_eq!(status, 200, "Should return 200");
    assert_eq!(response["meta"]["page"], 1, "Page 0 should default to 1");

    // Test with invalid limit (0 should default to 20)
    let (status2, response2) = list_attorneys_paginated("district9", 1, 0);

    assert_eq!(status2, 200, "Should return 200");
    assert_eq!(response2["meta"]["limit"], 20, "Limit 0 should default to 20");
}

#[spin_test]
fn test_pagination_metadata_accuracy() {
    let _store = key_value::Store::open("district12");

    // Create exactly 7 attorneys
    let ids = create_test_attorneys("district12", 7);

    // Get page 1 with limit 3
    let (_, response) = list_attorneys_paginated("district12", 1, 3);

    let meta = &response["meta"];
    assert!(meta["total"].as_u64().unwrap() >= 7, "Should have at least 7 total");
    assert!(meta["total_pages"].as_u64().unwrap() >= 3, "Should have at least 3 pages");
    assert!(meta["has_next"].as_bool().unwrap(), "Page 1 should have next");
    assert!(!meta["has_prev"].as_bool().unwrap(), "Page 1 should not have prev");

    // Get page 2 with limit 3
    let (_, response2) = list_attorneys_paginated("district12", 2, 3);

    let meta2 = &response2["meta"];
    assert!(meta2["has_next"].as_bool().unwrap() || meta2["total_pages"].as_u64().unwrap() == 2, "Page 2 might have next");
    assert!(meta2["has_prev"].as_bool().unwrap(), "Page 2 should have prev");

    // Clean up
    delete_attorneys(&ids, "district12");
}