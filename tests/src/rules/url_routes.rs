//! Rules engine URL-based route tests
//!
//! Tests for /api/courts/:district/rules endpoints.
//!
//! LIMITATION: URL-based route handlers cannot be tested with the current
//! spin-test-virt version. The URL handlers internally call `headers.set()`
//! via `add_district_header()` to inject the X-Court-District header before
//! delegating to the header-based handlers. The `fields.set` WASI HTTP method
//! is not implemented in spin-test-virt (triggers an `unreachable` WASM trap
//! at wasi/http.rs:370), causing any test that hits a URL-based route to crash.
//!
//! The URL handlers are thin wrappers that:
//! 1. Extract the district from the URL path parameter
//! 2. Set the X-Court-District header on the request
//! 3. Delegate to the corresponding header-based handler
//!
//! All underlying business logic is thoroughly tested via header-based route
//! tests in create_rule.rs, get_rule.rs, query_rules.rs, update_rule.rs,
//! and delete_rule.rs.
//!
//! When spin-test-virt adds support for `fields.set`, these tests should be
//! implemented:
//! - test_create_rule_via_url_route: POST /api/courts/district9/rules
//! - test_get_rule_via_url_route: GET /api/courts/district9/rules/:id
//! - test_list_rules_via_url_route: GET /api/courts/district9/rules
