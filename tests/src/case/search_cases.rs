//! Criminal case search and query tests
//!
//! Tests for GET /api/cases, GET /api/cases/statistics, GET /api/cases/by-judge/{judge},
//! and GET /api/cases/count-by-status/{status} endpoints

use spin_test_sdk::{spin_test, bindings::{wasi::http, fermyon::spin_test_virt::key_value}};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};

/// Helper to create a test case with specific properties
fn create_test_case_with_properties(title: &str, crime_type: &str, assigned_judge: &str, district: &str) -> String {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();
    headers.append(&"Content-Type".to_string(), b"application/json").unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/cases")).unwrap();

    let case_data = json!({
        "title": title,
        "description": format!("Test case for search testing: {}", title),
        "crimeType": crime_type,
        "assignedJudge": assigned_judge,
        "location": "Search Test City, STC"
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream.blocking_write_and_flush(serde_json::to_string(&case_data).unwrap().as_bytes()).unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    body_json["id"].as_str().unwrap().to_string()
}

/// Helper to search cases with query parameters
fn search_cases_request(query_params: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();

    let path = if query_params.is_empty() {
        "/api/cases".to_string()
    } else {
        format!("/api/cases?{}", query_params)
    };
    request.set_path_with_query(Some(&path)).unwrap();

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

/// Helper to get case statistics
fn get_statistics_request(district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/cases/statistics")).unwrap();

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

/// Helper to get cases by judge
fn get_cases_by_judge_request(judge: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/by-judge/{}", judge))).unwrap();

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

/// Helper to count cases by status
fn count_by_status_request(status: &str, district: &str) -> (u16, Value) {
    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!("/api/cases/count-by-status/{}", status))).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status_code = response.status();
    let body = response.body_as_string().unwrap_or_default();

    let body_json: Value = if body.is_empty() {
        json!(null)
    } else {
        serde_json::from_str(&body).unwrap_or(json!({"raw": body}))
    };

    (status_code, body_json)
}

#[spin_test]
fn test_search_cases_no_filters() {
    let _store = key_value::Store::open("district9");

    // Create a few test cases
    create_test_case_with_properties("Test Case 1", "financial_fraud", "Judge Smith", "district9");
    create_test_case_with_properties("Test Case 2", "cybercrime", "Judge Jones", "district9");

    let (status, response) = search_cases_request("", "district9");

    assert_eq!(status, 200, "Should return 200 for case search");
    assert!(response.get("cases").is_some(), "Should have cases array");
    assert!(response.get("total").is_some(), "Should have total count");
    assert!(response["cases"].is_array(), "Cases should be an array");
    assert!(response["total"].is_number(), "Total should be a number");
}

#[spin_test]
fn test_search_cases_by_status() {
    let _store = key_value::Store::open("district12");

    // Create test cases
    create_test_case_with_properties("Open Case", "financial_fraud", "Judge Smith", "district12");

    let (status, response) = search_cases_request("status=open", "district12");

    assert_eq!(status, 200, "Should return 200 for status filter");
    assert!(response["cases"].is_array(), "Should return cases array");

    // All returned cases should have open status
    if let Some(cases) = response["cases"].as_array() {
        for case in cases {
            assert_eq!(case["status"], "open", "All cases should have open status");
        }
    }
}

#[spin_test]
fn test_search_cases_by_priority() {
    let _store = key_value::Store::open("district9");

    // Create test cases
    create_test_case_with_properties("Medium Priority Case", "other", "Judge Test", "district9");

    let (status, response) = search_cases_request("priority=medium", "district9");

    assert_eq!(status, 200, "Should return 200 for priority filter");
    assert!(response["cases"].is_array(), "Should return cases array");

    // All returned cases should have medium priority
    if let Some(cases) = response["cases"].as_array() {
        for case in cases {
            assert_eq!(case["priority"], "medium", "All cases should have medium priority");
        }
    }
}

#[spin_test]
fn test_search_cases_by_judge() {
    let _store = key_value::Store::open("district12");

    // Create test cases with different judges
    create_test_case_with_properties("Judge Smith Case", "financial_fraud", "Judge Smith", "district12");
    create_test_case_with_properties("Judge Jones Case", "cybercrime", "Judge Jones", "district12");

    let (status, response) = search_cases_request("judge=Judge Smith", "district12");

    assert_eq!(status, 200, "Should return 200 for judge filter");
    assert!(response["cases"].is_array(), "Should return cases array");

    // All returned cases should be assigned to Judge Smith
    if let Some(cases) = response["cases"].as_array() {
        for case in cases {
            assert_eq!(case["assigned_judge"], "Judge Smith", "All cases should be assigned to Judge Smith");
        }
    }
}

#[spin_test]
fn test_search_cases_with_pagination() {
    let _store = key_value::Store::open("district9");

    // Create multiple test cases
    for i in 1..=5 {
        create_test_case_with_properties(
            &format!("Pagination Test Case {}", i),
            "other",
            "Judge Pagination",
            "district9"
        );
    }

    // Test first page with limit
    let (status, response) = search_cases_request("page=1&limit=2", "district9");

    assert_eq!(status, 200, "Should return 200 for paginated search");
    assert!(response["cases"].is_array(), "Should return cases array");
    assert!(response["total"].is_number(), "Should have total count");

    let cases = response["cases"].as_array().unwrap();
    assert!(cases.len() <= 2, "Should return at most 2 cases for limit=2");
}

#[spin_test]
fn test_search_cases_multiple_filters() {
    let _store = key_value::Store::open("district12");

    // Create test cases
    create_test_case_with_properties("Multi Filter Case", "financial_fraud", "Judge Multi", "district12");

    let (status, response) = search_cases_request("status=open&priority=medium&judge=Judge Multi", "district12");

    assert_eq!(status, 200, "Should return 200 for multiple filters");
    assert!(response["cases"].is_array(), "Should return cases array");

    // Verify all filters are applied
    if let Some(cases) = response["cases"].as_array() {
        for case in cases {
            assert_eq!(case["status"], "open");
            assert_eq!(case["priority"], "medium");
            assert_eq!(case["assigned_judge"], "Judge Multi");
        }
    }
}

#[spin_test]
fn test_search_cases_invalid_status() {
    let _store = key_value::Store::open("district9");

    let (status, response) = search_cases_request("status=invalid_status", "district9");

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid status, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("status") || body_str.contains("invalid"),
        "Error should mention invalid status"
    );
}

#[spin_test]
fn test_search_cases_invalid_pagination() {
    let _store = key_value::Store::open("district12");

    // Test invalid page number
    let (status, response) = search_cases_request("page=0", "district12");

    assert!(
        status == 400 || status == 200,
        "Should handle page=0 appropriately"
    );

    // Test invalid limit
    let (status2, response2) = search_cases_request("limit=0", "district12");

    assert!(
        status2 == 400 || status2 == 200,
        "Should handle limit=0 appropriately"
    );
}

#[spin_test]
fn test_get_case_statistics() {
    let _store = key_value::Store::open("district9");

    // Create some test cases
    create_test_case_with_properties("Stats Case 1", "financial_fraud", "Judge Stats", "district9");
    create_test_case_with_properties("Stats Case 2", "cybercrime", "Judge Stats", "district9");

    let (status, response) = get_statistics_request("district9");

    assert_eq!(status, 200, "Should return 200 for statistics");

    // Verify statistics structure (actual fields depend on implementation)
    assert!(response.is_object(), "Statistics should be an object");
}

#[spin_test]
fn test_get_cases_by_judge_endpoint() {
    let _store = key_value::Store::open("district12");

    // Create test cases for specific judge
    create_test_case_with_properties("Judge Specific Case 1", "financial_fraud", "Judge Specific", "district12");
    create_test_case_with_properties("Judge Specific Case 2", "cybercrime", "Judge Specific", "district12");

    let (status, response) = get_cases_by_judge_request("Judge Specific", "district12");

    assert_eq!(status, 200, "Should return 200 for cases by judge");
    assert!(response.get("cases").is_some(), "Should have cases array");
    assert!(response.get("total").is_some(), "Should have total count");

    // All cases should be assigned to the specified judge
    if let Some(cases) = response["cases"].as_array() {
        for case in cases {
            assert_eq!(case["assigned_judge"], "Judge Specific");
        }
    }
}

#[spin_test]
fn test_get_cases_by_nonexistent_judge() {
    let _store = key_value::Store::open("district9");

    let (status, response) = get_cases_by_judge_request("Judge Nonexistent", "district9");

    assert_eq!(status, 200, "Should return 200 even for non-existent judge");
    assert!(response["cases"].is_array(), "Should return empty cases array");
    assert_eq!(response["cases"].as_array().unwrap().len(), 0, "Should return no cases");
}

#[spin_test]
fn test_count_by_status() {
    let _store = key_value::Store::open("district12");

    // Create test cases
    create_test_case_with_properties("Count Case 1", "financial_fraud", "Judge Count", "district12");
    create_test_case_with_properties("Count Case 2", "cybercrime", "Judge Count", "district12");

    let (status, response) = count_by_status_request("open", "district12");

    assert_eq!(status, 200, "Should return 200 for count by status");
    assert!(response.get("count").is_some(), "Should have count field");
    assert!(response["count"].is_number(), "Count should be a number");
    assert!(response["count"].as_u64().unwrap() >= 0, "Count should be non-negative");
}

#[spin_test]
fn test_count_by_invalid_status() {
    let _store = key_value::Store::open("district9");

    let (status, response) = count_by_status_request("invalid_status", "district9");

    assert!(
        status == 400 || status == 422,
        "Should return 400 or 422 for invalid status, got {}", status
    );

    let body_str = serde_json::to_string(&response).unwrap();
    assert!(
        body_str.contains("status") || body_str.contains("invalid"),
        "Error should mention invalid status"
    );
}

#[spin_test]
fn test_search_endpoints_require_district_header() {
    // Test search cases without district header
    let headers = Headers::new();
    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/cases")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 400, "Search should require district header");

    // Test statistics without district header
    let headers2 = Headers::new();
    let request2 = OutgoingRequest::new(headers2);
    request2.set_method(&Method::Get).unwrap();
    request2.set_path_with_query(Some("/api/cases/statistics")).unwrap();

    let response2 = spin_test_sdk::perform_request(request2);
    assert_eq!(response2.status(), 400, "Statistics should require district header");

    // Test by-judge without district header
    let headers3 = Headers::new();
    let request3 = OutgoingRequest::new(headers3);
    request3.set_method(&Method::Get).unwrap();
    request3.set_path_with_query(Some("/api/cases/by-judge/TestJudge")).unwrap();

    let response3 = spin_test_sdk::perform_request(request3);
    assert_eq!(response3.status(), 400, "By-judge should require district header");
}

#[spin_test]
fn test_search_district_isolation() {
    // Create stores for both districts
    let _store9 = key_value::Store::open("district9");
    let _store12 = key_value::Store::open("district12");

    // Create cases in different districts
    create_test_case_with_properties("District 9 Case", "financial_fraud", "Judge Nine", "district9");
    create_test_case_with_properties("District 12 Case", "cybercrime", "Judge Twelve", "district12");

    // Search in district9
    let (status9, response9) = search_cases_request("", "district9");
    assert_eq!(status9, 200);

    // Search in district12
    let (status12, response12) = search_cases_request("", "district12");
    assert_eq!(status12, 200);

    // Verify cases are isolated by district
    let cases9 = response9["cases"].as_array().unwrap();
    let cases12 = response12["cases"].as_array().unwrap();

    // Check that district9 cases don't appear in district12 results and vice versa
    for case in cases9 {
        assert_ne!(case["title"], "District 12 Case", "District 9 search should not return District 12 cases");
    }

    for case in cases12 {
        assert_ne!(case["title"], "District 9 Case", "District 12 search should not return District 9 cases");
    }
}

#[spin_test]
fn test_search_with_url_encoded_parameters() {
    let _store = key_value::Store::open("district9");

    // Create test case with judge name containing spaces
    create_test_case_with_properties("URL Encoded Test", "other", "Judge John Smith", "district9");

    // Search with URL-encoded judge name
    let (status, response) = search_cases_request("judge=Judge%20John%20Smith", "district9");

    assert_eq!(status, 200, "Should handle URL-encoded parameters");
    assert!(response["cases"].is_array(), "Should return cases array");

    if let Some(cases) = response["cases"].as_array() {
        for case in cases {
            assert_eq!(case["assigned_judge"], "Judge John Smith");
        }
    }
}

#[spin_test]
fn test_search_response_format() {
    let _store = key_value::Store::open("district12");

    create_test_case_with_properties("Format Test Case", "financial_fraud", "Judge Format", "district12");

    let (status, response) = search_cases_request("", "district12");

    assert_eq!(status, 200);

    // Verify response structure
    assert!(response.get("cases").is_some(), "Should have cases field");
    assert!(response.get("total").is_some(), "Should have total field");
    assert!(response["cases"].is_array(), "Cases should be array");
    assert!(response["total"].is_number(), "Total should be number");

    // If there are cases, verify their structure
    if let Some(cases) = response["cases"].as_array() {
        if !cases.is_empty() {
            let case = &cases[0];
            assert!(case.get("id").is_some(), "Case should have id");
            assert!(case.get("case_number").is_some(), "Case should have case_number");
            assert!(case.get("title").is_some(), "Case should have title");
            assert!(case.get("status").is_some(), "Case should have status");
            assert!(case.get("priority").is_some(), "Case should have priority");
        }
    }
}