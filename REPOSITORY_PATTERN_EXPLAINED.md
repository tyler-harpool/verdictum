# Repository Pattern Implementation Explained

## Current State

The system uses a **Repository Factory** pattern with **concrete type returns**. This is a pragmatic choice that balances flexibility with simplicity.

## How It Works

```rust
// Handler gets a concrete repository type
let repo = RepositoryFactory::attorney_repo(&req); // Returns SpinKvAttorneyRepository
```

## Why Concrete Types (Not Trait Objects)?

### Current Implementation
```rust
pub fn attorney_repo(req: &Request) -> SpinKvAttorneyRepository {
    let tenant_id = tenant::get_tenant_id(req);
    let store_name = tenant::get_store_name(&tenant_id);
    SpinKvAttorneyRepository::with_store(store_name)
}
```

### Pros ✅
- **Simple**: No Arc<dyn Trait> complexity
- **Fast**: No dynamic dispatch overhead
- **Type-safe**: Compiler knows exact types
- **Works well**: For single storage backend (Spin KV)

### Cons ❌
- **Less flexible**: Can't swap backends at runtime
- **Requires recompilation**: To change storage backend

## How to Support PostgreSQL?

### Option 1: Feature Flags (Recommended)
```rust
#[cfg(feature = "spin-kv")]
pub fn attorney_repo(req: &Request) -> SpinKvAttorneyRepository {
    // Current implementation
}

#[cfg(feature = "postgres")]
pub fn attorney_repo(req: &Request) -> PostgresAttorneyRepository {
    // PostgreSQL implementation
}
```

### Option 2: Trait Objects (More Complex)
```rust
pub fn attorney_repo(req: &Request) -> Arc<dyn AttorneyRepository> {
    match config.backend {
        Backend::SpinKV => Arc::new(SpinKvAttorneyRepository::with_store(store)),
        Backend::Postgres => Arc::new(PostgresAttorneyRepository::new(conn)),
    }
}
```

## Current Multi-Tenant Implementation

### Data Flow
```
HTTP Request
    ↓
RepositoryFactory::attorney_repo(&req)
    ↓
1. Extract tenant from:
   - Headers (X-Court-District: SDNY)
   - Subdomain (sdny.lexodus.gov)
   - Query (?tenant=sdny)
    ↓
2. Generate store name: "tenant_sdny"
    ↓
3. Return SpinKvAttorneyRepository::with_store("tenant_sdny")
    ↓
All operations scoped to SDNY data only
```

### Tenant Isolation
Each tenant gets a **completely separate KV store**:
- `tenant_sdny` - Southern District of New York
- `tenant_edny` - Eastern District of New York
- `tenant_cdca` - Central District of California

## Repository Consistency Issues

### Problem
Different repositories have inconsistent constructors:
- `SpinKvAttorneyRepository::new()` returns `Result<Self>`
- `SpinKvSentencingRepository::new()` returns `ApiResult<Self>`
- `with_store()` methods return `Self`

### Solution
The factory handles this by only using `with_store()` which returns `Self`. The error handling is done via `.expect()` inside `with_store()`.

## Future PostgreSQL Support

When adding PostgreSQL support, you would:

### 1. Create PostgreSQL Adapter
```rust
// src/adapters/postgres_attorney_repository.rs
pub struct PostgresAttorneyRepository {
    pool: Arc<PgPool>,
    tenant_id: String,
}

impl PostgresAttorneyRepository {
    pub fn with_tenant(pool: Arc<PgPool>, tenant_id: String) -> Self {
        Self { pool, tenant_id }
    }
}

impl AttorneyRepository for PostgresAttorneyRepository {
    fn save_attorney(&self, attorney: Attorney) -> Result<Attorney> {
        // SQL with tenant isolation:
        // INSERT INTO attorneys (tenant_id, ...) VALUES ($1, ...)
    }
}
```

### 2. Update Factory (with feature flags)
```rust
impl RepositoryFactory {
    #[cfg(feature = "postgres")]
    pub fn attorney_repo(req: &Request) -> PostgresAttorneyRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let pool = get_db_pool(); // From connection pool
        PostgresAttorneyRepository::with_tenant(pool, tenant_id)
    }

    #[cfg(feature = "spin-kv")]
    pub fn attorney_repo(req: &Request) -> SpinKvAttorneyRepository {
        // Current implementation
    }
}
```

### 3. Compile with different features
```bash
# For Spin KV deployment
cargo build --features spin-kv

# For PostgreSQL deployment
cargo build --features postgres
```

## Summary

The current implementation:
1. ✅ **Works well** for Spin KV with multi-tenancy
2. ✅ **Simple and performant**
3. ✅ **Properly isolates tenant data**
4. ⚠️ **Requires recompilation** to change backends
5. ⚠️ **Has some inconsistent constructor signatures** (but handled by factory)

This is a **pragmatic tradeoff** that works well for the current needs while keeping the door open for future PostgreSQL support via feature flags.