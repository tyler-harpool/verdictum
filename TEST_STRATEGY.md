# Test Strategy for Federal Court Case Management System

## Overview
This document outlines the proper testing strategy for the Lexodus Federal Court Case Management System. We've moved from testing implementation details to testing API behavior, business logic, and system integration.

## Test Philosophy

### What We Test
1. **API Contract** - Does the endpoint accept correct inputs and return correct outputs?
2. **Business Logic** - Are domain rules correctly enforced?
3. **Integration** - Do components work together correctly?

### What We DON'T Test
- Implementation details (how data is stored)
- Internal state management
- Private methods
- Mock behaviors

## Test Structure

### 1. API Contract Tests (`api_contract_tests.rs`)

These tests verify the API behaves according to its contract:

```rust
#[spin_test]
fn test_attorney_create_contract() {
    // TEST: Valid attorney creation
    let attorney = json!({...});
    let (status, body) = make_api_request(Method::Post, "/api/attorneys", "district", Some(attorney));

    // Contract: POST /api/attorneys returns 200 on success
    assert_eq!(status, 200);

    // Contract: Response contains attorney with ID
    assert!(response["id"].is_string());
}
```

**Key Principles:**
- Test HTTP status codes
- Test request/response structure
- Test required vs optional fields
- Test error responses

### 2. Business Logic Tests (`business_logic_tests.rs`)

These tests verify domain rules are enforced:

```rust
#[spin_test]
fn test_attorney_bar_number_uniqueness() {
    // RULE: Bar numbers must be unique within district
    create_attorney(bar_number: "CA12345");
    let duplicate = create_attorney(bar_number: "CA12345");

    // Business Rule: Duplicate bar number rejected
    assert!(duplicate.status == 400 || duplicate.status == 409);
}
```

**Key Principles:**
- Test validation rules
- Test business constraints
- Test state transitions
- Test authorization rules

### 3. Integration Tests (`integration_tests.rs`)

These tests verify components work together:

```rust
#[spin_test]
fn test_attorney_complete_workflow() {
    // TEST: Complete attorney lifecycle
    let attorney = create_attorney();
    update_attorney(attorney.id);
    add_bar_admission(attorney.id);
    add_to_cja_panel(attorney.id);

    // Verify complete attorney data
    let final_attorney = get_attorney(attorney.id);
    assert!(final_attorney.has_all_expected_data());
}
```

**Key Principles:**
- Test end-to-end workflows
- Test multi-step operations
- Test component interactions
- Test error recovery

## Multi-Tenancy Testing

All tests must respect the multi-tenant architecture:

```rust
// ALWAYS include district header
headers.append("X-Court-District", "district-id");

// Test district isolation
create_in_district1();
assert_not_found_in_district2();
```

## Test Data Management

### Use Realistic Data
```rust
// GOOD: Realistic attorney data
let attorney = json!({
    "bar_number": "CA12345",
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@law.com",
    "phone": "415-555-0100",
    "address": {
        "street1": "123 Market St",
        "city": "San Francisco",
        "state": "CA",
        "zip_code": "94105",
        "country": "USA"
    }
});

// BAD: Incomplete or unrealistic data
let attorney = json!({
    "name": "test",
    "email": "test"
});
```

### Test Data Isolation
- Each test creates its own data
- Use unique identifiers (e.g., `TEST-{timestamp}-{random}`)
- Clean up is automatic (each test runs in isolation)

## Error Testing

### Test All Error Cases
```rust
// Missing required field
test_missing_bar_number() -> 400

// Invalid format
test_invalid_email() -> 400

// Not found
test_get_nonexistent() -> 404

// Unauthorized
test_unauthorized_access() -> 401

// Conflict
test_duplicate_creation() -> 409

// Server error
test_internal_error() -> 500
```

## Performance Considerations

### Test Under Load
```rust
#[spin_test]
fn test_handles_multiple_attorneys() {
    // Create multiple attorneys
    for i in 0..100 {
        create_attorney(format!("BAR-{}", i));
    }

    // Verify system still responsive
    let list = get_all_attorneys();
    assert!(list.len() >= 100);
}
```

## Migration Path

### From Old Tests to New Tests

1. **Identify test purpose**
   - Is it testing API behavior? → api_contract_tests.rs
   - Is it testing business rules? → business_logic_tests.rs
   - Is it testing workflows? → integration_tests.rs

2. **Remove implementation details**
   - Remove direct KV store mocking
   - Remove internal state checks
   - Focus on observable behavior

3. **Use proper API calls**
   ```rust
   // OLD: Direct KV store manipulation
   store.set("attorneys:123", data);

   // NEW: Use API to create state
   make_api_request(Method::Post, "/api/attorneys", district, data);
   ```

## Running Tests

```bash
# Run all tests
spin test run

# Run specific test file
spin test run --filter api_contract

# Run with verbose output
spin test run --verbose
```

## Test Checklist

Before committing tests, ensure:

- [ ] Tests use proper API calls, not internal methods
- [ ] Tests include proper district headers
- [ ] Tests verify API contracts (status codes, response structure)
- [ ] Tests verify business rules are enforced
- [ ] Tests handle error cases
- [ ] Tests use realistic data
- [ ] Tests are independent (no shared state)
- [ ] Tests have clear assertions with meaningful messages

## Common Pitfalls to Avoid

### 1. Testing Implementation Details
```rust
// BAD: Testing internal storage
assert!(store.get("attorneys:123").is_some());

// GOOD: Testing API behavior
assert_eq!(get_attorney("123").status, 200);
```

### 2. Incomplete Error Testing
```rust
// BAD: Only testing happy path
create_attorney(valid_data) -> 200

// GOOD: Testing all cases
create_attorney(valid_data) -> 200
create_attorney(missing_field) -> 400
create_attorney(invalid_email) -> 400
create_attorney(duplicate) -> 409
```

### 3. Ignoring Multi-Tenancy
```rust
// BAD: No district isolation
create_attorney();

// GOOD: Proper district handling
create_attorney_in_district("district1");
verify_not_in_district("district2");
```

## Conclusion

The new test strategy focuses on:
1. **Behavior over implementation**
2. **API contracts over internal state**
3. **Business rules over mock verification**
4. **Integration over isolation**

This approach ensures our tests:
- Are maintainable
- Test what matters
- Don't break when implementation changes
- Provide confidence in system behavior