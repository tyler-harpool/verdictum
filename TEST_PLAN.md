# Comprehensive Test Plan for All Endpoints

## Test Pattern Template

Every test file should follow this structure:

```rust
use spin_test_sdk::{
    spin_test,
    bindings::{fermyon::spin_test_virt::key_value, wasi::http}
};

// Helper function with proper district header
fn make_request(method: &str, path: &str, district: &str, body: Option<&str>) -> (u16, String) {
    // 1. Create headers with district
    // 2. Add content-type if needed
    // 3. Build request
    // 4. Execute and return (status, body)
}

#[spin_test]
fn test_endpoint_name() {
    // 1. Open KV store for district
    let store = key_value::Store::open("district9");

    // 2. Pre-populate test data if needed
    store.set("key", data);

    // 3. Make request
    let (status, body) = make_request(...);

    // 4. Assert response
    assert_eq!(status, expected);

    // 5. Verify store operations
    assert_eq!(store.calls(), expected_calls);
}
```

## Test Categories by Priority

### Priority 1: Core CRUD Operations
These are the most critical tests that ensure basic functionality.

#### 1. Criminal Cases (17 endpoints)
**File:** `tests/src/criminal_case_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/cases | test_create_case_success | Create new case with valid data |
| POST /api/cases | test_create_case_duplicate | Attempt duplicate case number (409) |
| GET /api/cases/:id | test_get_case_by_id_found | Retrieve existing case |
| GET /api/cases/:id | test_get_case_by_id_not_found | 404 for non-existent |
| GET /api/cases/by-number/:num | test_get_case_by_number | Find by case number |
| GET /api/cases/by-judge/:judge | test_get_cases_by_judge | List judge's cases |
| PATCH /api/cases/:id/status | test_update_case_status | Change case status |
| DELETE /api/cases/:id | test_delete_case | Soft delete case |
| POST /api/cases/:id/defendants | test_add_defendant | Add defendant to case |
| POST /api/cases/:id/plea | test_enter_plea | Record plea entry |
| GET /api/cases/statistics | test_get_statistics | Aggregate statistics |

#### 2. Judges (14 endpoints)
**File:** `tests/src/judge_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/judges | test_create_judge | Create new judge |
| GET /api/judges | test_get_all_judges | List all judges |
| GET /api/judges/:id | test_get_judge_by_id | Get specific judge |
| GET /api/judges/available | test_get_available_judges | Filter available judges |
| PATCH /api/judges/:id/status | test_update_judge_status | Update status |
| POST /api/judges/:id/conflicts | test_add_conflict | Add conflict of interest |
| GET /api/judges/conflicts/check/:party | test_check_conflicts | Check for conflicts |
| POST /api/assignments | test_assign_case | Assign case to judge |
| GET /api/judges/workload | test_get_workload | Get workload stats |

#### 3. Attorneys (67 endpoints)
**File:** `tests/src/attorney_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/attorneys | test_create_attorney | Create attorney record |
| GET /api/attorneys | test_list_attorneys | List all attorneys |
| GET /api/attorneys/:bar | test_get_by_bar_number | Find by bar number |
| PATCH /api/attorneys/:bar | test_update_attorney | Update attorney info |
| POST /api/attorneys/:bar/cases | test_add_case | Associate with case |
| GET /api/attorneys/:bar/metrics | test_get_metrics | Performance metrics |
| POST /api/attorneys/cja-panel | test_add_to_cja | Add to CJA panel |
| GET /api/attorneys/cja-panel | test_get_cja_panel | List CJA attorneys |

### Priority 2: Document Management

#### 4. Opinions (24 endpoints)
**File:** `tests/src/opinion_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/opinions | test_create_opinion | Create new opinion |
| GET /api/opinions | test_list_opinions | List all opinions |
| GET /api/opinions/:id | test_get_opinion | Get specific opinion |
| PUT /api/opinions/:id | test_update_opinion | Update opinion |
| DELETE /api/opinions/:id | test_delete_opinion | Delete opinion |
| POST /api/opinions/:id/publish | test_publish_opinion | Publish opinion |
| POST /api/opinions/:id/drafts | test_create_draft | Create draft version |

#### 5. Orders (23 endpoints)
**File:** `tests/src/order_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/orders | test_create_order | Create court order |
| GET /api/orders | test_list_orders | List orders |
| GET /api/orders/:id | test_get_order | Get specific order |
| POST /api/orders/:id/sign | test_sign_order | Sign order |
| GET /api/orders/templates | test_get_templates | List templates |
| POST /api/orders/from-template | test_create_from_template | Use template |

### Priority 3: Scheduling & Deadlines

#### 6. Docket/Calendar (27 endpoints)
**File:** `tests/src/docket_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/docket/entries | test_create_entry | Create docket entry |
| GET /api/docket/case/:id | test_get_case_docket | Get case docket |
| POST /api/calendar/events | test_schedule_event | Schedule hearing |
| GET /api/calendar/judge/:id | test_judge_schedule | Judge's calendar |
| GET /api/calendar/available-slot/:id | test_find_slot | Find available time |
| POST /api/speedy-trial/:id | test_init_speedy_trial | Start clock |

#### 7. Deadlines (26 endpoints)
**File:** `tests/src/deadline_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/deadlines | test_create_deadline | Create deadline |
| GET /api/deadlines/urgent | test_get_urgent | Urgent deadlines |
| POST /api/deadlines/:id/complete | test_complete | Mark complete |
| POST /api/deadlines/:id/extensions | test_request_extension | Request extension |
| GET /api/compliance/report | test_compliance_report | Generate report |

#### 8. Sentencing (31 endpoints)
**File:** `tests/src/sentencing_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| POST /api/sentencing | test_create_sentencing | Record sentence |
| GET /api/sentencing/:id | test_get_sentencing | Get sentence details |
| POST /api/sentencing/guidelines | test_calculate_guidelines | Calculate guidelines |
| GET /api/sentencing/pending | test_get_pending | Pending sentences |

### Priority 4: Configuration & Admin

#### 9. Configuration (9 endpoints)
**File:** `tests/src/config_tests_v2.rs`

| Endpoint | Test Name | Test Scenario |
|----------|-----------|---------------|
| GET /api/config | test_get_config | Get district config |
| PUT /api/config/overrides/district | test_update_district | Update district config |
| PUT /api/config/overrides/judge | test_update_judge | Judge-specific config |
| POST /api/config/preview | test_preview | Preview changes |

## Test Data Strategy

### Standard Test Districts
- **district9**: Primary test district (from District 9 movie)
- **district12**: Secondary test district (from Hunger Games)

### Standard Test Data

```rust
// Judge test data
const TEST_JUDGE: &str = r#"{
    "id": "judge-001",
    "name": "Hon. Ellen Ripley",
    "district": "district9",
    "status": "active"
}"#;

// Case test data
const TEST_CASE: &str = r#"{
    "case_number": "CR-2024-001",
    "title": "United States v. Test",
    "judge_id": "judge-001",
    "status": "active"
}"#;

// Attorney test data
const TEST_ATTORNEY: &str = r#"{
    "bar_number": "12345",
    "first_name": "John",
    "last_name": "Doe",
    "email": "jdoe@law.com"
}"#;
```

## Multi-Tenancy Test Requirements

Each domain should have at least one test that:
1. Creates data in district9
2. Creates data in district12
3. Verifies isolation between districts
4. Tests both header-based and URL-based routing

Example:
```rust
#[spin_test]
fn test_multi_tenant_isolation() {
    let store_d9 = key_value::Store::open("district9");
    let store_d12 = key_value::Store::open("district12");

    // Create in district9
    make_request("POST", "/api/judges", "district9", judge_d9);

    // Create in district12
    make_request("POST", "/api/judges", "district12", judge_d12);

    // Verify separate stores were used
    assert!(!store_d9.calls().is_empty());
    assert!(!store_d12.calls().is_empty());
}
```

## Error Handling Tests

Each domain should test:
1. Missing required fields (400)
2. Invalid data format (400)
3. Non-existent resource (404)
4. Duplicate creation (409)
5. Missing tenant header (400)

## Performance Test Considerations

For endpoints that return lists:
1. Test with empty store
2. Test with single item
3. Test with multiple items (10+)
4. Verify pagination if implemented

## Implementation Order

1. **Week 1**: Priority 1 tests (Criminal Cases, Judges, Attorneys)
2. **Week 2**: Priority 2 tests (Opinions, Orders)
3. **Week 3**: Priority 3 tests (Docket, Deadlines, Sentencing)
4. **Week 4**: Priority 4 tests (Config) + cleanup

## Success Metrics

- [ ] All endpoints have at least one happy path test
- [ ] All CRUD operations tested (Create, Read, Update, Delete)
- [ ] Multi-tenancy isolation verified for each domain
- [ ] Both routing patterns (header and URL) tested
- [ ] Error cases covered (400, 404, 409)
- [ ] Store operations verified with `.calls()`
- [ ] No "tenant_not_specified" errors
- [ ] Tests use district9 and district12 consistently

## Common Issues to Avoid

1. **Don't forget district headers** - Every request needs a tenant
2. **Pre-populate stores** for GET/UPDATE/DELETE tests
3. **Verify store calls** to ensure correct data access
4. **Use consistent test data** across related tests
5. **Test both routing patterns** for migration validation

## Test Execution

```bash
# Build the application
spin build

# Run all tests
spin test run

# Run specific test file (if supported)
spin test run --filter judge_tests_v2
```

## Maintenance

- Review and update tests when adding new endpoints
- Keep test data consistent with domain models
- Update this plan when adding new features
- Document any spin-test limitations encountered