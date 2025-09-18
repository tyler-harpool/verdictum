# Multi-Tenant Architecture Documentation

## Overview

The Lexodus Federal Court Case Management System implements a comprehensive multi-tenant architecture to support all 94 federal district courts in the United States. Each court district operates as an isolated tenant with complete data separation.

## Architecture Pattern

### Hexagonal Architecture with Multi-Tenancy

```
┌─────────────────────────────────────────────────────────┐
│                    HTTP HANDLERS                         │
│             (Driving Adapters/Controllers)               │
│                                                          │
│  /api/attorneys  /api/judges  /api/cases  /api/dockets  │
└────────────────────┬────────────────────────────────────┘
                     │ Uses
┌────────────────────▼────────────────────────────────────┐
│               REPOSITORY FACTORY                         │
│          (Multi-Tenant Orchestration Layer)              │
│                                                          │
│  • Tenant Detection (headers, subdomain, query params)  │
│  • Store Name Generation (tenant_sdny, tenant_edny)     │
│  • Repository Instance Creation with Correct Store       │
└────────────────────┬────────────────────────────────────┘
                     │ Returns implementations of
┌────────────────────▼────────────────────────────────────┐
│                  PORTS (Interfaces)                      │
│             (Domain/Business Contracts)                  │
│                                                          │
│  AttorneyRepository  JudgeRepository  CaseRepository    │
└────────────────────┬────────────────────────────────────┘
                     │ Implemented by
┌────────────────────▼────────────────────────────────────┐
│                ADAPTERS (Driven Adapters)                │
│            (Infrastructure Implementations)              │
│                                                          │
│  SpinKvAttorneyRepository::with_store(store_name)       │
│  SpinKvJudgeRepository::with_store(store_name)          │
│  SpinKvCaseRepository::with_store(store_name)           │
└──────────────────────────────────────────────────────────┘
```

## Tenant Identification

The system identifies tenants through multiple methods, in order of precedence:

### 1. HTTP Headers
```http
X-Tenant-ID: sdny
X-Court-District: SDNY
```

### 2. Subdomain
```
https://sdny.lexodus.gov/api/cases
```

### 3. Query Parameter
```
https://lexodus.gov/api/cases?tenant=sdny
```

### 4. Default
If no tenant is specified, defaults to "default"

## Federal Court Districts Supported

All 94 federal district courts are supported:

### Major Districts
- **SDNY** - Southern District of New York
- **EDNY** - Eastern District of New York
- **CDCA** - Central District of California
- **NDCA** - Northern District of California
- **NDIL** - Northern District of Illinois
- **SDTX** - Southern District of Texas
- **DDC** - District of Columbia

### Store Naming Convention
Each tenant gets a dedicated store named: `tenant_{district_code}`

Examples:
- `tenant_sdny` - Southern District of New York
- `tenant_edny` - Eastern District of New York
- `tenant_cdca` - Central District of California

## Implementation Details

### Repository Factory

```rust
use crate::utils::repository_factory::RepositoryFactory;

pub fn create_attorney(req: Request, _params: Params) -> Response {
    // Automatically gets the correct tenant-specific repository
    let repo = RepositoryFactory::attorney_repo(&req);

    // All operations now scoped to the tenant
    let attorney = Attorney::new(...);
    repo.save_attorney(attorney)?;
}
```

### Tenant Utility Functions

```rust
// Extract tenant ID from request
let tenant_id = tenant::get_tenant_id(&req);

// Get store name for tenant
let store_name = tenant::get_store_name(&tenant_id);

// Check access permissions
let has_access = tenant::has_tenant_access(user_id, &tenant_id);
```

## API Examples

### Creating an Attorney in SDNY

```bash
curl -X POST https://lexodus.gov/api/attorneys \
  -H "X-Court-District: SDNY" \
  -H "Content-Type: application/json" \
  -d '{
    "bar_number": "NY12345",
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@law.com",
    "phone": "212-555-0100"
  }'
```

### Accessing Cases from EDNY Subdomain

```bash
curl https://edny.lexodus.gov/api/cases
```

### Query Parameter Tenant Selection

```bash
curl "https://lexodus.gov/api/judges?tenant=cdca"
```

## Security Considerations

### Data Isolation
- Complete data isolation between tenants
- No cross-tenant data access possible
- Each tenant has its own Spin KV store

### Tenant ID Sanitization
- Only alphanumeric characters, hyphens, and underscores allowed
- Maximum length: 50 characters
- Converted to lowercase for consistency

### Access Control
- `has_tenant_access()` function for permission checking
- Can be integrated with authentication system
- Per-tenant access control lists

## Benefits of This Architecture

### 1. **Scalability**
- Add new federal districts without code changes
- Independent scaling per district
- No shared resources between tenants

### 2. **Maintainability**
- Single codebase for all districts
- Centralized tenant logic in factory
- Clear separation of concerns

### 3. **Compliance**
- Data sovereignty per district
- Audit trails per tenant
- Jurisdictional data isolation

### 4. **Flexibility**
- Easy to add custom features per district
- Different configurations per tenant
- Gradual rollout of features

## Testing Multi-Tenancy

### Unit Tests
```rust
#[test]
fn test_tenant_isolation() {
    let req_sdny = create_request_with_header("X-Court-District", "SDNY");
    let req_edny = create_request_with_header("X-Court-District", "EDNY");

    let repo_sdny = RepositoryFactory::attorney_repo(&req_sdny);
    let repo_edny = RepositoryFactory::attorney_repo(&req_edny);

    // Verify different stores are used
    assert_ne!(repo_sdny.store_name, repo_edny.store_name);
}
```

### Integration Tests
1. Create data in SDNY tenant
2. Verify data not visible in EDNY tenant
3. Verify subdomain routing works correctly
4. Test header precedence over query parameters

## Migration Guide

### For Existing Handlers

Before (single-tenant):
```rust
let repo = SpinKvAttorneyRepository::new();
```

After (multi-tenant):
```rust
let repo = RepositoryFactory::attorney_repo(&req);
```

### For New Features
Always use the RepositoryFactory pattern for any new repository creation.

## Monitoring and Observability

### Metrics to Track
- Requests per tenant
- Data volume per tenant
- Active users per district
- API usage patterns per district

### Logging
All operations should include tenant context:
```rust
log::info!("Creating attorney for tenant: {}", tenant_id);
```

## Future Enhancements

### Planned Features
1. **Dynamic Tenant Creation** - Add new districts without restart
2. **Tenant-Specific Configuration** - Custom settings per district
3. **Cross-District Search** - Authorized search across districts
4. **Tenant Migration Tools** - Move data between tenants
5. **Backup per Tenant** - Independent backup strategies

### Potential Optimizations
1. Connection pooling per tenant
2. Caching strategies per district
3. Rate limiting per tenant
4. Custom schemas per district type

## Conclusion

This multi-tenant architecture provides a robust, scalable foundation for managing all 94 federal district courts while maintaining complete data isolation and security. The factory pattern ensures consistent tenant handling across all handlers while preserving the clean hexagonal architecture.