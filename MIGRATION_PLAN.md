# URL-Based Tenant Migration Plan

## Overview
Migrate from header-based tenant identification (`X-Court-District`) to URL-based routing while maintaining backward compatibility and KV store isolation.

## URL Structure

### Option 1: Simple (Recommended)
```
/api/courts/{district}/{resource}
/api/courts/sdny/cases
/api/courts/nybk/cases
/api/courts/edtx/opinions
```

### Option 2: Hierarchical
```
/api/courts/{type}/{district}/{resource}
/api/courts/district/sdny/cases
/api/courts/bankruptcy/nybk/cases
/api/courts/appellate/ca2/opinions
```

## Implementation Phases

### Phase 1: Foundation (No Breaking Changes)
1. Create `UrlTenantExtractor` utility
2. Create `UrlRepositoryFactory` that extracts from path
3. Add parallel URL routes in lib.rs
4. Keep all existing header-based routes

### Phase 2: Domain Migration (Incremental)
Migrate one domain at a time:
1. **Config** - Simple, stateless, good test case
2. **Health** - Trivial endpoint
3. **Cases** - Core domain, high value
4. **Judges** - Related to cases
5. **Attorneys** - Related to cases
6. **Docket** - Complex, do later
7. **Orders/Opinions** - Complex, do last

### Phase 3: Testing & Validation
- Run both endpoints in parallel
- Compare responses
- Monitor usage patterns
- Validate KV isolation

### Phase 4: Deprecation
1. Add deprecation headers to old endpoints
2. Update documentation
3. Notify consumers
4. Remove old endpoints after grace period

## Technical Implementation

### 1. New Tenant Extractor
```rust
// src/utils/url_tenant.rs
pub fn extract_tenant_from_path(path: &str) -> Option<String> {
    // Extract from /api/courts/{district}/...
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 4 && parts[1] == "api" && parts[2] == "courts" {
        Some(parts[3].to_string())
    } else {
        None
    }
}
```

### 2. Enhanced Repository Factory
```rust
// src/utils/repository_factory.rs
impl RepositoryFactory {
    pub fn case_repo_from_path(path: &str) -> SpinKvCaseRepository {
        let tenant_id = extract_tenant_from_path(path)
            .unwrap_or("TENANT_NOT_SPECIFIED".to_string());
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvCaseRepository::with_store(store_name)
    }
}
```

### 3. New Handlers
```rust
// src/handlers/cases_url.rs
pub fn get_cases_by_district(req: Request, params: Params) -> Response {
    let district = params.get("district").unwrap();
    let repo = SpinKvCaseRepository::with_store(district);
    // ... rest of logic
}
```

### 4. Router Updates
```rust
// In lib.rs
// Keep existing routes
router.get("/api/cases", handlers::criminal_case::search_cases);

// Add new URL-based routes
router.get("/api/courts/:district/cases", handlers::cases_url::search_cases);
router.post("/api/courts/:district/cases", handlers::cases_url::create_case);
```

## Benefits of This Approach

1. **No Breaking Changes** - Existing clients continue working
2. **Gradual Migration** - Move one domain at a time
3. **A/B Testing** - Run both in parallel to validate
4. **Clean Architecture** - Hexagonal design makes this easy
5. **Same KV Isolation** - Store selection logic unchanged

## Migration Checklist

### Foundation (✅ Complete)
- [x] Create UrlTenantExtractor utility
- [x] Update repository factory with URL support
- [x] Create parallel URL handlers for config
- [x] Add URL routes to router
- [x] Create comprehensive migration test suite

### Config Domain (✅ Complete)
- [x] Implement URL-based handlers
- [x] Test both routing patterns work
- [x] Verify KV store isolation maintained
- [x] Migration tests passing

### Remaining Domains (🚧 In Progress)
- [ ] Cases domain
- [ ] Judges domain
- [ ] Attorneys domain
- [ ] Docket domain
- [ ] Orders domain
- [ ] Opinions domain
- [ ] Deadlines domain
- [ ] Sentencing domain

### Final Steps
- [ ] Add deprecation headers to old endpoints
- [ ] Update API documentation
- [ ] Remove old header-based routes (after grace period)

## Risk Mitigation

1. **Data Consistency** - Same KV stores, just different routing
2. **Performance** - No change in data access patterns
3. **Security** - Same tenant isolation, clearer boundaries
4. **Rollback** - Can instantly revert by removing new routes

## Success Criteria

- All endpoints accessible via both patterns
- No performance degradation
- Automated tests pass for both patterns
- KV isolation maintained
- Clear migration path for clients