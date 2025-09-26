# Test Analysis and Use Case Documentation

## Failing Tests Analysis

### 1. test-create-deadline-with-invalid-data-returns-bad-request
**Use Case**: Validates that the API properly rejects invalid deadline creation requests
**Legitimacy**: ✅ VALID - Essential for API input validation
**Expected**: Return HTTP 400 for invalid JSON structure
**Test Location**: tests/src/deadline_tests.rs:457

### 2. test-create-opinion-returns-created
**Use Case**: Validates successful opinion creation in the system
**Legitimacy**: ✅ VALID - Core CRUD operation for opinions
**Expected**: Return HTTP 201 when creating a valid opinion
**Test Location**: tests/src/opinion_crud_tests.rs:101

### 3. test-create-opinion-with-invalid-data-returns-bad-request
**Use Case**: Validates that the API properly rejects invalid opinion creation requests
**Legitimacy**: ✅ VALID - Essential for API input validation
**Expected**: Return HTTP 400 for invalid JSON structure
**Test Location**: tests/src/opinion_crud_tests.rs:133

### 4. test-get-all-judges-url-based
**Use Case**: Tests retrieving all judges for a district using URL-based routing
**Legitimacy**: ✅ VALID - Tests URL migration for judge listing
**Expected**: Return HTTP 200 with array of judges
**Test Location**: tests/src/judge_tests.rs (URL-based test)

### 5. test-get-case-by-id-url-based
**Use Case**: Tests retrieving a specific criminal case by ID using URL-based routing
**Legitimacy**: ✅ VALID - Core read operation for criminal cases
**Expected**: Return HTTP 200 with case details
**Test Location**: tests/src/criminal_case_tests.rs (URL-based test)

### 6. test-get-case-statistics-header-based
**Use Case**: Tests retrieving case statistics using header-based routing
**Legitimacy**: ✅ VALID - Tests backward compatibility for statistics endpoint
**Expected**: Return HTTP 200 with statistics data
**Test Location**: tests/src/criminal_case_tests.rs (header-based test)

### 7. test-order-missing-district-returns-error
**Use Case**: Validates that order endpoints require district identification
**Legitimacy**: ✅ VALID - Security/multi-tenancy validation
**Expected**: Return HTTP 400 when district header/parameter is missing
**Test Location**: tests/src/order_tests.rs

### 8. test-update-case-status-url-based
**Use Case**: Tests updating criminal case status using URL-based routing
**Legitimacy**: ✅ VALID - Core update operation for case management
**Expected**: Return HTTP 200 when updating valid case
**Test Location**: tests/src/criminal_case_tests.rs (URL-based test)

## Summary

All 8 failing tests are **legitimate and necessary**:
- 3 tests validate input validation (bad request handling)
- 4 tests validate core CRUD operations (create, read, update)
- 1 test validates multi-tenancy security (missing district)

These tests failing indicate real issues in the implementation that need investigation.

## Root Cause Analysis

### Primary Issue: Tenant ID Resolution
The main issue appears to be with tenant identification in the repository layer:

1. **No Tenant Specified**: When the district header is missing, the code returns `TENANT_NOT_SPECIFIED` which causes store opening to fail
2. **Store Name Generation**: The store name is generated as `tenant_{district}` but Spin KV might not have these stores pre-configured
3. **Error Handling**: When tenant is not specified, operations fail with "AccessDenied" errors

### Test Failure Patterns:

1. **Invalid Data Tests (400 expected, 404 received)**:
   - `test-create-deadline-with-invalid-data-returns-bad-request`
   - `test-create-opinion-with-invalid-data-returns-bad-request`
   - These tests expect 400 (Bad Request) but get 404, likely because the endpoint routing fails before validation

2. **CRUD Operations (various errors)**:
   - `test-create-opinion-returns-created`
   - `test-get-all-judges-url-based`
   - `test-get-case-by-id-url-based`
   - `test-update-case-status-url-based`
   - These fail due to tenant store access issues

3. **Missing District Tests (working correctly)**:
   - `test-order-missing-district-returns-error`
   - This is actually working as designed - returns error when district is missing

### The URL Migration Impact:

The URL-based routing extracts the district from the URL path and adds it as a header before calling the original handlers. This should work, but the issue is that:

1. The test environment might not have the KV stores configured
2. The default store fallback mechanism is not working
3. The error messages are misleading (404 instead of proper error codes)