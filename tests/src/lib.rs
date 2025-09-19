use spin_test_sdk::{
    bindings::{fermyon::spin_test_virt, wasi::http},
    spin_test,
};

// Include migration tests module
mod migration_tests;

/// Test that health check endpoint works
#[spin_test]
fn test_health_check() {
    use http::types::{Headers, Method, Scheme};

    let headers = Headers::new();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/health")).unwrap();
    request.set_scheme(Some(&Scheme::Http)).unwrap();
    request.set_authority(Some("127.0.0.1:3000")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200, "Health check should return 200 OK");
}

/// Test that missing district header returns error
#[spin_test]
fn test_missing_district_header_returns_error() {
    use http::types::{Headers, Method, Scheme};

    let headers = Headers::new();
    // No X-Court-District header

    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/config")).unwrap();
    request.set_scheme(Some(&Scheme::Http)).unwrap();
    request.set_authority(Some("127.0.0.1:3000")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should return 400 Bad Request when district header is missing
    assert_eq!(
        response.status(),
        400,
        "Should return 400 when X-Court-District header is missing"
    );
}

/// Test that config endpoint accepts district header
#[spin_test]
fn test_config_accepts_district_header() {
    use http::types::{Headers, Method, Scheme};

    // Mock the KV store that will be used
    let _store = spin_test_virt::key_value::Store::open("default");

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), &"TEST".as_bytes().to_vec()).unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/config")).unwrap();
    request.set_scheme(Some(&Scheme::Http)).unwrap();
    request.set_authority(Some("127.0.0.1:3000")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // We expect either 200 (success) or 500 (if TOML loading fails in test env)
    // but NOT 400 (bad request) since we provided the header
    assert_ne!(
        response.status(),
        400,
        "Should not return 400 when X-Court-District header is provided"
    );
}

/// Test that the header-based config endpoint returns valid configuration data
#[spin_test]
fn test_config_endpoint_returns_valid_response() {
    use http::types::{Headers, Method, Scheme};

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), &"SDNY".as_bytes().to_vec()).unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/config")).unwrap();
    request.set_scheme(Some(&Scheme::Http)).unwrap();
    request.set_authority(Some("127.0.0.1:3000")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should return 200 OK
    assert_eq!(response.status(), 200, "Config endpoint should return 200 OK");

    // Read response body
    let incoming_body = response.consume().unwrap();
    let stream = incoming_body.stream().unwrap();
    let mut body = Vec::new();

    loop {
        match stream.blocking_read(1024 * 1024) {
            Ok(chunk) if !chunk.is_empty() => body.extend_from_slice(&chunk),
            _ => break,
        }
    }

    // Parse as JSON to verify it's valid configuration
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("Response should be valid JSON");

    // Verify expected configuration structure
    assert!(json.get("features").is_some(), "Config should have features");
    assert!(json.get("system").is_some(), "Config should have system info");
    assert!(json.get("_metadata").is_some(), "Config should have metadata");

    // Verify district metadata matches request (could be uppercase or lowercase)
    let metadata = json.get("_metadata").unwrap();
    let district = metadata.get("district").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        district.eq_ignore_ascii_case("sdny"),
        "Metadata district '{}' should match 'SDNY' (case-insensitive)", district
    );
}

/// Test that the URL-based config endpoint returns valid configuration data
#[spin_test]
fn test_url_based_config_endpoint() {
    use http::types::{Headers, Method, Scheme};

    let headers = Headers::new();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/courts/edny/config")).unwrap();
    request.set_scheme(Some(&Scheme::Http)).unwrap();
    request.set_authority(Some("127.0.0.1:3000")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should return 200 OK
    assert_eq!(response.status(), 200, "URL-based config endpoint should return 200 OK");

    // Read response body
    let incoming_body = response.consume().unwrap();
    let stream = incoming_body.stream().unwrap();
    let mut body = Vec::new();

    loop {
        match stream.blocking_read(1024 * 1024) {
            Ok(chunk) if !chunk.is_empty() => body.extend_from_slice(&chunk),
            _ => break,
        }
    }

    // Parse as JSON to verify it's valid configuration
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("Response should be valid JSON");

    // Verify expected configuration structure
    assert!(json.get("features").is_some(), "Config should have features");
    assert!(json.get("system").is_some(), "Config should have system info");
    assert!(json.get("_metadata").is_some(), "Config should have metadata");

    // Verify district metadata matches the URL path (could be uppercase or lowercase)
    let metadata = json.get("_metadata").unwrap();
    let district = metadata.get("district").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        district.eq_ignore_ascii_case("edny"),
        "Metadata district '{}' should match 'EDNY' from URL (case-insensitive)", district
    );

    // Verify it's a district court configuration
    let features = json.get("features").unwrap();
    assert_eq!(
        features.get("court_type").and_then(|v| v.as_str()),
        Some("district"),
        "EDNY should be a district court"
    );
}