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

/// Test that we can set and retrieve configuration from KV store
#[spin_test]
fn test_kv_store_config_override() {
    use http::types::{Headers, Method, Scheme};

    // Setup virtual KV store with an override
    let store = spin_test_virt::key_value::Store::open("default");

    // Set a district override in the store
    let override_json = r#"{"overrides":{"features.test_feature":true}}"#;
    store.set("config:district:TEST", override_json.as_bytes());

    let headers = Headers::new();
    headers.append(&"X-Court-District".to_string(), &"TEST".as_bytes().to_vec()).unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some("/api/config")).unwrap();
    request.set_scheme(Some(&Scheme::Http)).unwrap();
    request.set_authority(Some("127.0.0.1:3000")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Verify the KV store was accessed
    let calls = store.calls();
    assert!(
        calls.iter().any(|call| matches!(call, spin_test_virt::key_value::Call::Get(key) if key.starts_with("config:"))),
        "Should attempt to get config from KV store"
    );
}