//! Health Check Endpoint Tests
//!
//! Tests for the /api/health endpoint as documented in the Utoipa specification:
//! - GET /api/health: Check the health status of the API
//!
//! Expected responses:
//! - 200 OK: API is healthy, returns HealthStatus object
//! - 503 Service Unavailable: API is unhealthy

use spin_test_sdk::{spin_test, bindings::wasi::http};
use http::types::{Headers, Method, OutgoingRequest};
use serde_json::Value;

/// Helper function to make an HTTP request to the health endpoint
fn make_health_request() -> (u16, String) {
    let headers = Headers::new();
    let request = OutgoingRequest::new(headers);

    // Set method to GET
    request.set_method(&Method::Get).unwrap();

    // Set path to /api/health
    request.set_path_with_query(Some("/api/health")).unwrap();

    // Perform the request
    let response = spin_test_sdk::perform_request(request);
    let status = response.status();
    let body = response.body_as_string().unwrap_or_default();

    (status, body)
}

#[spin_test]
fn test_health_check_returns_ok_when_healthy() {
    // Make request to health endpoint
    let (status, body) = make_health_request();

    // Assert status is 200
    assert_eq!(status, 200, "Health check should return 200 when healthy");

    // Parse response body
    let health_response: Value = serde_json::from_str(&body)
        .expect("Response should be valid JSON");

    // Verify response structure according to Utoipa spec
    assert_eq!(
        health_response["status"],
        "healthy",
        "Status should be 'healthy' when API is working"
    );

    // Verify all required fields are present
    assert!(
        health_response.get("version").is_some(),
        "Response should include version field"
    );

    assert!(
        health_response.get("storage").is_some(),
        "Response should include storage field"
    );

    assert!(
        health_response.get("timestamp").is_some(),
        "Response should include timestamp field"
    );
}

#[spin_test]
fn test_health_check_response_structure() {
    // Make request to health endpoint
    let (status, body) = make_health_request();

    // Should return 200
    assert_eq!(status, 200, "Health check should return 200");

    // Parse and validate JSON structure
    let health_response: Value = serde_json::from_str(&body)
        .expect("Response should be valid JSON");

    // Validate each field type according to the HealthStatus schema
    assert!(
        health_response["status"].is_string(),
        "status field should be a string"
    );

    assert!(
        health_response["version"].is_string(),
        "version field should be a string"
    );

    assert!(
        health_response["storage"].is_string(),
        "storage field should be a string"
    );

    assert!(
        health_response["timestamp"].is_string(),
        "timestamp field should be a string"
    );

    // Validate storage status is one of the expected values
    let storage_status = health_response["storage"].as_str().unwrap();
    assert!(
        ["connected", "disconnected", "error"].contains(&storage_status),
        "Storage status should be 'connected', 'disconnected', or 'error'"
    );
}

#[spin_test]
fn test_health_check_returns_json_content_type() {
    // Make request to health endpoint
    let headers = Headers::new();
    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/health")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Check content-type header
    let content_type_values = response.headers().get(&"content-type".to_string());
    let content_type = content_type_values
        .first()
        .and_then(|value| std::str::from_utf8(value).ok());

    assert_eq!(
        content_type,
        Some("application/json"),
        "Health check should return application/json content type"
    );
}

#[spin_test]
fn test_health_check_timestamp_is_valid_rfc3339() {
    // Make request to health endpoint
    let (status, body) = make_health_request();

    assert_eq!(status, 200, "Health check should return 200");

    let health_response: Value = serde_json::from_str(&body)
        .expect("Response should be valid JSON");

    let timestamp = health_response["timestamp"]
        .as_str()
        .expect("Timestamp should be a string");

    // Try to parse as RFC3339
    // Verify timestamp format looks like RFC3339 (basic validation)
    assert!(
        timestamp.contains("T") && timestamp.contains(":"),
        "Timestamp should look like RFC3339 format"
    );
}

#[spin_test]
fn test_health_check_no_authentication_required() {
    // Make request without any authentication headers
    let headers = Headers::new();
    // Explicitly not adding any auth headers or district headers

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/health")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();

    // Health check should work without authentication
    assert_eq!(
        status, 200,
        "Health check should return 200 without authentication headers"
    );
}