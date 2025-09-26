# Spin Test SDK Patterns Documentation

## Overview

The Spin Test SDK provides a way to test WebAssembly components in Spin applications. This document outlines the correct patterns to avoid the common error:

```
thread '<unnamed>' panicked at crates/spin-test-virt/src/wasi/http.rs:370:9
```

## Core Principles

### 1. Use Proper Import Structure

**Correct:**
```rust
use spin_test_sdk::{
    spin_test,
    bindings::{fermyon::spin_test_virt::key_value, wasi::http}
};
use http::types::{Headers, Method, OutgoingRequest};
```

**Incorrect:**
```rust
// Don't mix spin_sdk with spin_test_sdk
use spin_sdk::http::{Request, Response};
use spin_test_sdk::spin_test;
```

### 2. Create Requests Using OutgoingRequest

**Correct Pattern:**
```rust
fn make_request(method: &str, path: &str, district: &str, body: Option<&str>) -> (u16, String) {
    let headers = Headers::new();

    // Add headers
    headers.append(&"X-Court-District".to_string(), district.as_bytes()).unwrap();

    if body.is_some() {
        headers.append(&"Content-Type".to_string(), b"application/json").unwrap();
    }

    // Create request using OutgoingRequest
    let request = OutgoingRequest::new(headers);

    // Set method using enum
    let method_enum = match method {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "PATCH" => Method::Patch,
        "DELETE" => Method::Delete,
        _ => Method::Get,
    };
    request.set_method(&method_enum).unwrap();
    request.set_path_with_query(Some(path)).unwrap();

    // Add body if provided
    if let Some(body_content) = body {
        let request_body = request.body().unwrap();
        let stream = request_body.write().unwrap();
        stream.blocking_write_and_flush(body_content.as_bytes()).unwrap();
        drop(stream);  // Important: drop stream before finishing
        http::types::OutgoingBody::finish(request_body, None).unwrap();
    }

    // Execute request
    let response = spin_test_sdk::perform_request(request);
    (response.status(), response.body_as_string().unwrap_or_default())
}
```

### 3. Mock KV Store Access

**Correct Pattern:**
```rust
#[spin_test]
fn test_with_kv_store() {
    // Open the store to enable mocking
    let store = key_value::Store::open("district9");

    // Pre-populate data
    let data = json!({"id": "123", "name": "Test"});
    store.set("key", serde_json::to_vec(&data).unwrap().as_slice());

    // Make request that will use the store
    let (status, body) = make_request("GET", "/api/endpoint", "district9", None);

    // Verify store operations
    let calls = store.calls();
    assert!(calls.contains(&key_value::Call::Get("key".to_string())));
}
```

### 4. Handle Multi-Tenancy in Tests

**Pattern for Testing Multi-Tenant Isolation:**
```rust
#[spin_test]
fn test_tenant_isolation() {
    // Open stores for different tenants
    let store_d9 = key_value::Store::open("district9");
    let store_d12 = key_value::Store::open("district12");

    // Create data in district9
    let data_d9 = json!({"district": "9"});
    let (status, _) = make_request("POST", "/api/data", "district9", Some(&data_d9.to_string()));
    assert_eq!(status, 201);

    // Try to access from district12 - should fail
    let (status, _) = make_request("GET", "/api/data/d9-id", "district12", None);
    assert_eq!(status, 404, "Should not access other district's data");

    // Verify stores were accessed independently
    assert!(!store_d9.calls().is_empty());
    assert!(!store_d12.calls().is_empty());
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Wrong Response Type in Handlers

**Problem:**
```rust
// This causes the panic!
pub fn handler(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    // ...
}
```

**Solution:**
```rust
// Return Response directly
pub fn handler(req: Request, params: Params) -> Response {
    match inner_handler(req, params) {
        Ok(resp) => resp.into_response(),
        Err(e) => error_response(&e)
    }
}
```

### Pitfall 2: Invalid Test Data

**Problem:**
```rust
// Enum values must match serde serialization
let data = json!({
    "order_type": "PretrialRelease"  // Wrong: PascalCase
});
```

**Solution:**
```rust
// Use snake_case for enums with #[serde(rename_all = "snake_case")]
let data = json!({
    "order_type": "pretrial_release"  // Correct: snake_case
});
```

### Pitfall 3: String IDs Instead of UUIDs

**Problem:**
```rust
let data = json!({
    "judge_id": "judge-001"  // Wrong: not a valid UUID
});
```

**Solution:**
```rust
let data = json!({
    "judge_id": "d45463d9-c01e-5d65-9c6a-f879e574cdca"  // Correct: valid UUID
});
```

### Pitfall 4: Missing Required Fields

**Problem:**
```rust
let attorney = json!({
    "bar_number": "12345",
    "name": "John Doe"
    // Missing: address field
});
```

**Solution:**
```rust
let attorney = json!({
    "bar_number": "12345",
    "first_name": "John",
    "last_name": "Doe",
    "address": {
        "street": "123 Main St",
        "city": "New York",
        "state": "NY",
        "zip": "10001"
    }
});
```

## Test Data Validation Checklist

Before running tests, ensure:

1. ✅ All enum values use snake_case (not PascalCase)
2. ✅ All ID fields contain valid UUIDs (not string placeholders)
3. ✅ All required fields are present in JSON payloads
4. ✅ Date fields use ISO format (YYYY-MM-DD)
5. ✅ Numeric fields use correct types (not strings)
6. ✅ Store names match tenant IDs exactly

## Helper Functions Template

```rust
// Standard test helper module
mod test_helpers {
    use super::*;

    pub fn make_request(/* ... */) -> (u16, String) {
        // Implementation shown above
    }

    pub fn get_test_uuid(index: u8) -> String {
        format!("d45463d9-c01e-5d65-9c6a-f879e574cd{:02x}", index)
    }

    pub fn get_test_date(days_offset: i32) -> String {
        // Return date in YYYY-MM-DD format
        "2024-01-01"  // Simplified example
    }
}
```

## Running Tests

```bash
# Run all tests
spin test run

# Run specific test file
spin test run --filter criminal_case

# Run with verbose output for debugging
RUST_BACKTRACE=1 spin test run
```

## Debugging Failed Tests

When tests fail with the panic error:

1. Check handler return types - must be `Response`, not `ApiResult<impl IntoResponse>`
2. Verify test data matches expected enum serialization
3. Ensure all required fields are present
4. Validate UUID formats
5. Check that stores are properly opened before use

## Migration from Old Patterns

If migrating from header-based to URL-based routing:

```rust
// Old pattern
let (status, body) = make_request("GET", "/api/judges", "district9", None);

// New pattern (URL-based)
let (status, body) = make_request_no_header("GET", "/api/courts/district9/judges", None);

// Hybrid support (both should work)
assert_eq!(status_header, status_url);
assert_eq!(body_header, body_url);
```

## Best Practices

1. **Always open stores before use** - Even if not directly accessing them
2. **Use consistent test data** - Create reusable test data functions
3. **Test both success and failure cases** - Verify error handling
4. **Check store operations** - Use `store.calls()` to verify data access
5. **Test multi-tenant isolation** - Ensure districts can't access each other's data
6. **Document test purposes** - Use descriptive test names and comments

## Common Assertions

```rust
// Status code assertions
assert_eq!(status, 200, "Should return 200 OK");
assert!(status == 201 || status == 409, "Should create or conflict");

// Body content assertions
assert!(body.contains("error"), "Should contain error message");
let response: Value = serde_json::from_str(&body).unwrap();
assert_eq!(response["field"], "value");

// Store operation assertions
let calls = store.calls();
assert!(!calls.is_empty(), "Store should be accessed");
assert!(calls.contains(&key_value::Call::Set("key".to_string())));
```

## References

- [Spin Test SDK Documentation](https://developer.fermyon.com/spin/testing)
- [Spin KV Store API](https://developer.fermyon.com/spin/kv-store-api)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)