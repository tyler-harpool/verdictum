# Repository Factory Pattern - Current Implementation

## Overview

The Repository Factory pattern centralizes the creation of tenant-specific repositories, ensuring consistent multi-tenant data isolation across all handlers.

## How It Works

### 1. Request Flow
```
HTTP Request → Handler → RepositoryFactory → Tenant-Specific Repository → Spin KV Store
```

### 2. Factory Implementation

The factory (`src/utils/repository_factory.rs`) provides static methods for each repository type:

```rust
impl RepositoryFactory {
    pub fn attorney_repo(req: &Request) -> SpinKvAttorneyRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvAttorneyRepository::with_store(store_name)
    }

    // Similar methods for all repository types...
}
```

### 3. Tenant Detection

The factory uses `tenant::get_tenant_id()` which checks (in order):
1. `X-Tenant-ID` header
2. `X-Court-District` header
3. Subdomain (e.g., `sdny.lexodus.gov`)
4. Query parameter (`?tenant=sdny`)
5. Defaults to `"default"`

### 4. Store Naming

Each tenant gets an isolated store: `tenant_{district_code}`
- `tenant_sdny` - Southern District of New York
- `tenant_edny` - Eastern District of New York
- `tenant_cdca` - Central District of California

## Usage in Handlers

All handlers now use the factory pattern:

```rust
// Before (without factory)
let repo = SpinKvAttorneyRepository::new()?;

// After (with factory)
let repo = RepositoryFactory::attorney_repo(&req);
```

## Benefits

1. **Centralized tenant logic** - All tenant detection in one place
2. **Consistent isolation** - Every handler automatically gets the correct tenant store
3. **Easy to maintain** - Change tenant logic once, affects all handlers
4. **Type safety** - Returns concrete types, no dynamic dispatch overhead
5. **Simple to use** - Single line to get a tenant-specific repository

## Current Repositories Using Factory

- ✅ Attorney Repository
- ✅ Judge Repository
- ✅ Criminal Case Repository
- ✅ Deadline Repository
- ✅ Docket Repository
- ✅ Document Repository
- ✅ Sentencing Repository
- ✅ Opinion Repository (via macro)
- ✅ Order Repository (via macro)

## Implementation Details

### Repository Constructor Pattern

All repositories implement a `with_store()` method:

```rust
impl SpinKvAttorneyRepository {
    pub fn with_store(store_name: String) -> Self {
        let store = Store::open(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }
}
```

### Handler Pattern

Standard handler implementation:

```rust
pub fn create_attorney(req: Request, _params: Params) -> Response {
    // Get tenant-specific repository
    let repo = RepositoryFactory::attorney_repo(&req);

    // Parse request
    let request: CreateAttorneyRequest = parse_json_body(&req)?;

    // Create domain object using constructor
    let attorney = Attorney::new(
        request.bar_number,
        request.first_name,
        request.last_name,
        request.email,
        request.phone,
    );

    // Save to tenant-specific store
    repo.save_attorney(attorney)?;

    json_response(200, attorney)
}
```

## Testing the Factory

### Unit Test Example
```rust
#[test]
fn test_factory_creates_tenant_specific_repos() {
    let req_sdny = create_request_with_header("X-Court-District", "SDNY");
    let req_edny = create_request_with_header("X-Court-District", "EDNY");

    let repo_sdny = RepositoryFactory::attorney_repo(&req_sdny);
    let repo_edny = RepositoryFactory::attorney_repo(&req_edny);

    // Different tenants get different stores
    assert_ne!(repo_sdny.store_name, repo_edny.store_name);
}
```

### Manual Testing
```bash
# Create attorney in SDNY
curl -X POST http://localhost:3000/api/attorneys \
  -H "X-Court-District: SDNY" \
  -H "Content-Type: application/json" \
  -d '{"bar_number": "NY12345", "first_name": "John", "last_name": "Doe"}'

# Query EDNY - won't see SDNY data
curl http://localhost:3000/api/attorneys \
  -H "X-Court-District: EDNY"
```

## Future Flexibility

While the current implementation returns concrete types (`SpinKvAttorneyRepository`), the factory pattern makes it easier to switch storage backends in the future:

1. **Feature flags** - Compile-time backend selection
2. **Configuration** - Environment-based backend selection
3. **Minimal changes** - Only factory needs updating, not handlers

The factory pattern provides a clean abstraction layer between handlers and repositories, making future changes manageable.