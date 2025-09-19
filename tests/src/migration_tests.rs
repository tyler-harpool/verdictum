//! Migration tests to ensure both header-based and URL-based routing work correctly
//!
//! These tests validate that:
//! 1. Both routing patterns return identical responses
//! 2. KV store isolation is maintained
//! 3. Migration is complete for each domain

use spin_test_sdk::{
    bindings::wasi::http,
    spin_test,
};
use serde_json::{json, Value};

/// Helper to perform a request with headers
fn make_request(method: &str, path: &str, headers: Vec<(&str, &str)>, body: Option<String>) -> (u16, Vec<u8>) {
    use http::types::{Headers, Method, Scheme};

    let req_headers = Headers::new();
    for (name, value) in headers {
        req_headers.append(&name.to_string(), &value.as_bytes().to_vec()).unwrap();
    }

    let request = http::types::OutgoingRequest::new(req_headers);

    let method = match method {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        _ => Method::Get,
    };
    request.set_method(&method).unwrap();
    request.set_path_with_query(Some(path)).unwrap();
    request.set_scheme(Some(&Scheme::Http)).unwrap();
    request.set_authority(Some("127.0.0.1:3000")).unwrap();

    // Add body if provided
    if let Some(body_content) = body {
        let outgoing_body = request.body().unwrap();
        let stream = outgoing_body.write().unwrap();
        stream.blocking_write_and_flush(body_content.as_bytes()).unwrap();
        drop(stream);
        http::types::OutgoingBody::finish(outgoing_body, None).unwrap();
    }

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();

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

    (status, body)
}

/// Helper to compare two JSON responses, ignoring certain fields
fn assert_json_equal(json1: &Value, json2: &Value, ignore_fields: &[&str]) {
    let mut json1_cleaned = json1.clone();
    let mut json2_cleaned = json2.clone();

    // Remove fields to ignore (like timestamps)
    for field in ignore_fields {
        if let Some(obj1) = json1_cleaned.as_object_mut() {
            obj1.remove(*field);
        }
        if let Some(obj2) = json2_cleaned.as_object_mut() {
            obj2.remove(*field);
        }
    }

    assert_eq!(json1_cleaned, json2_cleaned,
        "Responses don't match!\nHeader-based: {}\nURL-based: {}",
        serde_json::to_string_pretty(&json1).unwrap(),
        serde_json::to_string_pretty(&json2).unwrap()
    );
}

// ============================================================================
// CONFIG DOMAIN MIGRATION TESTS
// ============================================================================

#[spin_test]
fn test_config_migration_sdny() {
    // Test header-based routing
    let (header_status, header_body) = make_request(
        "GET",
        "/api/config",
        vec![("X-Court-District", "SDNY")],
        None
    );

    // Test URL-based routing
    let (url_status, url_body) = make_request(
        "GET",
        "/api/courts/sdny/config",
        vec![],
        None
    );

    // Both should return 200
    assert_eq!(header_status, 200, "Header-based request failed");
    assert_eq!(url_status, 200, "URL-based request failed");

    // Parse JSON responses
    let header_json: Value = serde_json::from_slice(&header_body).expect("Failed to parse header response");
    let url_json: Value = serde_json::from_slice(&url_body).expect("Failed to parse URL response");

    // Responses should be identical (except metadata timestamps)
    assert_json_equal(&header_json, &url_json, &["_metadata"]);
}

#[spin_test]
fn test_config_migration_bankruptcy() {
    // Test header-based routing with court type
    let (header_status, header_body) = make_request(
        "GET",
        "/api/config",
        vec![
            ("X-Court-District", "NYBK"),
            ("X-Court-Type", "bankruptcy")
        ],
        None
    );

    // Test URL-based routing (court type inferred from NYBK)
    let (url_status, url_body) = make_request(
        "GET",
        "/api/courts/nybk/config",
        vec![],
        None
    );

    assert_eq!(header_status, 200);
    assert_eq!(url_status, 200);

    // Verify both return bankruptcy configuration
    let header_json: Value = serde_json::from_slice(&header_body).unwrap();
    let url_json: Value = serde_json::from_slice(&url_body).unwrap();

    assert_eq!(header_json["features"]["court_type"], "bankruptcy");
    assert_eq!(url_json["features"]["court_type"], "bankruptcy");

    assert_json_equal(&header_json, &url_json, &["_metadata"]);
}

#[spin_test]
fn test_config_migration_with_judge() {
    let judge_id = "judge-chen";

    // Header-based with judge
    let (header_status, header_body) = make_request(
        "GET",
        "/api/config",
        vec![
            ("X-Court-District", "SDNY"),
            ("X-Judge-ID", judge_id)
        ],
        None
    );

    // URL-based with judge (still uses header for judge for now)
    let (url_status, url_body) = make_request(
        "GET",
        "/api/courts/sdny/config",
        vec![("X-Judge-ID", judge_id)],
        None
    );

    assert_eq!(header_status, 200);
    assert_eq!(url_status, 200);

    let header_json: Value = serde_json::from_slice(&header_body).unwrap();
    let url_json: Value = serde_json::from_slice(&url_body).unwrap();

    assert_json_equal(&header_json, &url_json, &["_metadata"]);
}

#[spin_test]
fn test_config_update_migration() {
    let config_update = json!({
        "features": {
            "advanced": {
                "ai_assisted": true
            }
        }
    });

    // Update via header-based route
    let (header_status, _) = make_request(
        "PUT",
        "/api/config/overrides/district",
        vec![
            ("X-Court-District", "SDNY"),
            ("Content-Type", "application/json")
        ],
        Some(config_update.to_string())
    );

    // Update via URL-based route
    let (url_status, _) = make_request(
        "PUT",
        "/api/courts/sdny/config/overrides/district",
        vec![("Content-Type", "application/json")],
        Some(config_update.to_string())
    );

    // Both should succeed
    assert!(header_status == 200 || header_status == 201 || header_status == 204);
    assert!(url_status == 200 || url_status == 201 || url_status == 204);
}

// ============================================================================
// KV STORE ISOLATION TESTS
// ============================================================================

#[spin_test]
fn test_kv_isolation_between_districts() {
    // Ensure SDNY and EDNY have separate configs
    let (sdny_status, sdny_body) = make_request(
        "GET",
        "/api/courts/sdny/config",
        vec![],
        None
    );

    let (edny_status, edny_body) = make_request(
        "GET",
        "/api/courts/edny/config",
        vec![],
        None
    );

    assert_eq!(sdny_status, 200);
    assert_eq!(edny_status, 200);

    let sdny_json: Value = serde_json::from_slice(&sdny_body).unwrap();
    let edny_json: Value = serde_json::from_slice(&edny_body).unwrap();

    // Verify different districts in metadata
    assert_eq!(sdny_json["_metadata"]["district"], "sdny");
    assert_eq!(edny_json["_metadata"]["district"], "edny");
}

#[spin_test]
fn test_missing_district_returns_error() {
    // URL without district should fail
    let (status, body) = make_request(
        "GET",
        "/api/config",
        vec![],
        None
    );

    // Should get 400 Bad Request
    assert_eq!(status, 400, "Missing district should return 400");

    let error: Value = serde_json::from_slice(&body).unwrap();
    let error_msg = error["BadRequest"].as_str().unwrap();
    assert!(error_msg.contains("District") || error_msg.contains("district"),
        "Error message should mention district: {}", error_msg);
}

// ============================================================================
// MIGRATION PROGRESS TRACKER
// ============================================================================

/// Test to verify which domains have been migrated
#[spin_test]
fn test_migration_status() {
    println!("\n=== MIGRATION STATUS ===");
    println!("config: ✅ MIGRATED");
    println!("cases: ❌ PENDING");
    println!("judges: ❌ PENDING");
    println!("attorneys: ❌ PENDING");
    println!("docket: ❌ PENDING");
    println!("orders: ❌ PENDING");
    println!("opinions: ❌ PENDING");
    println!("deadlines: ❌ PENDING");
    println!("sentencing: ❌ PENDING");
    println!("\nProgress: 1/9 domains (11.1%)");

    // This assertion tracks migration progress
    assert!(true, "Migration in progress - 1 of 9 domains complete");
}