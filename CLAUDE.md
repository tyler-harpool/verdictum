# Claude Implementation Specification: Hexagonal Spin SDK Applications

## OVERVIEW

This specification defines the **REQUIRED** implementation patterns for building Spin SDK applications using hexagonal architecture. Follow these patterns exactly when building Spin applications.

## MANDATORY PROJECT STRUCTURE

```
my-spin-app/
├── src/
│   ├── lib.rs                    # Application entry point
│   ├── domain/                   # PURE business logic - NO external dependencies
│   │   ├── mod.rs
│   │   ├── models/              # Domain entities and value objects
│   │   │   ├── mod.rs
│   │   │   └── [entity].rs
│   │   ├── services/            # Business logic orchestration
│   │   │   ├── mod.rs
│   │   │   └── [entity]_service.rs
│   │   ├── ports/               # Trait definitions (interfaces)
│   │   │   ├── mod.rs
│   │   │   ├── repositories.rs
│   │   │   └── notifications.rs
│   │   └── errors.rs            # Domain error types
│   ├── inbound/                 # Adapters that CALL our domain
│   │   ├── mod.rs
│   │   ├── http/
│   │   │   ├── mod.rs
│   │   │   ├── handlers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── docs.rs
│   │   │   │   └── [entity].rs
│   │   │   └── models/          # HTTP request/response models
│   │   │       ├── mod.rs
│   │   │       └── [entity].rs
│   │   └── server.rs
│   └── outbound/                # Adapters our domain CALLS
│       ├── mod.rs
│       ├── repositories/
│       │   ├── mod.rs
│       │   ├── kv_[entity]_repository.rs
│       │   ├── sqlite_[entity]_repository.rs
│       │   └── postgres_[entity]_repository.rs
│       └── notifications/
│           └── [type]_notifier.rs
├── tests/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── spin.toml
└── Cargo.toml
```

## IMPLEMENTATION REQUIREMENTS

### 1. DOMAIN MODELS (src/domain/models/)

**REQUIRED PATTERN:**
```rust
// src/domain/models/[entity].rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Core domain entity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = serde_json::json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "field": "value"
}))]
pub struct [Entity] {
    pub id: [Entity]Id,
    // ... other fields with proper value objects
}

/// Request to create entity
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct Create[Entity]Request {
    // Only fields needed for creation
}

/// Request to update entity
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct Update[Entity]Request {
    // Optional fields for updates
}

// Value objects for validation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct [Entity]Id(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct [Entity][Field](pub String);

impl [Entity][Field] {
    pub fn new(value: String) -> Result<Self, String> {
        // REQUIRED: Validation logic here
        if value.trim().is_empty() {
            return Err("Field cannot be empty".to_string());
        }
        Ok(Self(value.trim().to_string()))
    }
}
```

**REQUIREMENTS:**
- All domain models MUST use utoipa `ToSchema` derive
- All string fields MUST be wrapped in value objects with validation
- Create/Update requests MUST be separate from entity models
- All models MUST be serializable and deserializable

### 2. DOMAIN PORTS (src/domain/ports/)

**REQUIRED PATTERN:**
```rust
// src/domain/ports/repositories.rs
use async_trait::async_trait;
use crate::domain::{models::*, errors::*};

#[async_trait]
pub trait [Entity]Repository: Send + Sync + Clone + 'static {
    async fn create(&self, request: Create[Entity]Request) -> Result<[Entity], [Entity]Error>;
    async fn find_by_id(&self, id: [Entity]Id) -> Result<Option<[Entity]>, [Entity]Error>;
    async fn find_all(&self) -> Result<Vec<[Entity]>, [Entity]Error>;
    async fn update(&self, id: [Entity]Id, request: Update[Entity]Request) -> Result<Option<[Entity]>, [Entity]Error>;
    async fn delete(&self, id: [Entity]Id) -> Result<bool, [Entity]Error>;
}

#[async_trait]
pub trait [Entity]NotificationService: Send + Sync + Clone + 'static {
    async fn notify_created(&self, entity: &[Entity]) -> Result<(), [Entity]Error>;
    async fn notify_updated(&self, entity: &[Entity]) -> Result<(), [Entity]Error>;
}
```

**REQUIREMENTS:**
- All traits MUST be async with proper bounds: `Send + Sync + Clone + 'static`
- All methods MUST return domain error types
- Repository MUST include standard CRUD operations
- Notification services MUST be separate traits

### 3. DOMAIN ERRORS (src/domain/errors.rs)

**REQUIRED PATTERN:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum [Entity]Error {
    #[error("Entity not found with ID: {id}")]
    NotFound { id: String },

    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    #[error("Entity already exists: {message}")]
    Duplicate { message: String },

    #[error("Repository operation failed: {message}")]
    RepositoryError { message: String },

    #[error("Notification failed: {message}")]
    NotificationError { message: String },

    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl [Entity]Error {
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData { message: message.into() }
    }

    pub fn repository_error(message: impl Into<String>) -> Self {
        Self::RepositoryError { message: message.into() }
    }
}
```

**REQUIREMENTS:**
- MUST use `thiserror::Error`
- MUST include NotFound, InvalidData, Duplicate, RepositoryError variants
- MUST include Unknown variant with `anyhow::Error`
- MUST provide helper constructors

### 4. DOMAIN SERVICES (src/domain/services/)

**REQUIRED PATTERN:**
```rust
// src/domain/services/[entity]_service.rs
use async_trait::async_trait;
use crate::domain::{models::*, ports::*, errors::*};

#[async_trait]
pub trait [Entity]Service: Send + Sync + Clone + 'static {
    async fn create(&self, request: Create[Entity]Request) -> Result<[Entity], [Entity]Error>;
    async fn get(&self, id: [Entity]Id) -> Result<[Entity], [Entity]Error>;
    async fn list(&self) -> Result<Vec<[Entity]>, [Entity]Error>;
    async fn update(&self, id: [Entity]Id, request: Update[Entity]Request) -> Result<[Entity], [Entity]Error>;
    async fn delete(&self, id: [Entity]Id) -> Result<(), [Entity]Error>;
}

#[derive(Clone)]
pub struct [Entity]ServiceImpl<R, N>
where
    R: [Entity]Repository,
    N: [Entity]NotificationService,
{
    repository: R,
    notifier: N,
}

impl<R, N> [Entity]ServiceImpl<R, N>
where
    R: [Entity]Repository,
    N: [Entity]NotificationService,
{
    pub fn new(repository: R, notifier: N) -> Self {
        Self { repository, notifier }
    }
}

#[async_trait]
impl<R, N> [Entity]Service for [Entity]ServiceImpl<R, N>
where
    R: [Entity]Repository,
    N: [Entity]NotificationService,
{
    async fn create(&self, request: Create[Entity]Request) -> Result<[Entity], [Entity]Error> {
        let entity = self.repository.create(request).await?;

        // REQUIRED: Handle notification failures gracefully
        if let Err(e) = self.notifier.notify_created(&entity).await {
            eprintln!("Notification failed: {}", e);
        }

        Ok(entity)
    }

    async fn get(&self, id: [Entity]Id) -> Result<[Entity], [Entity]Error> {
        self.repository.find_by_id(id).await?
            .ok_or_else(|| [Entity]Error::NotFound { id: id.0.to_string() })
    }

    // ... implement all other methods
}
```

**REQUIREMENTS:**
- Service trait MUST define business operations
- Implementation MUST be generic over repository and notification traits
- Notification failures MUST NOT fail the operation (log instead)
- All operations MUST go through the service layer

### 5. KV REPOSITORY ADAPTER (src/outbound/repositories/)

**REQUIRED PATTERN:**
```rust
// src/outbound/repositories/kv_[entity]_repository.rs
use async_trait::async_trait;
use spin_sdk::key_value::Store;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::{models::*, ports::*, errors::*};

#[derive(Clone)]
pub struct Kv[Entity]Repository {
    store: Store,
}

impl Kv[Entity]Repository {
    pub fn new() -> Result<Self, [Entity]Error> {
        let store = Store::open_default()
            .map_err(|e| [Entity]Error::repository_error(format!("Failed to open KV store: {}", e)))?;
        Ok(Self { store })
    }
}

#[async_trait]
impl [Entity]Repository for Kv[Entity]Repository {
    async fn create(&self, request: Create[Entity]Request) -> Result<[Entity], [Entity]Error> {
        let entity = [Entity] {
            id: [Entity]Id(Uuid::new_v4()),
            // ... map from request
            created_at: Utc::now(),
            // ... other fields
        };

        let key = format!("[entity]:{}", entity.id.0);
        let value = serde_json::to_vec(&entity)
            .map_err(|e| [Entity]Error::repository_error(format!("Serialization failed: {}", e)))?;

        self.store.set(&key, &value)
            .map_err(|e| [Entity]Error::repository_error(format!("Failed to store: {}", e)))?;

        Ok(entity)
    }

    async fn find_by_id(&self, id: [Entity]Id) -> Result<Option<[Entity]>, [Entity]Error> {
        let key = format!("[entity]:{}", id.0);

        match self.store.get(&key) {
            Ok(Some(data)) => {
                let entity: [Entity] = serde_json::from_slice(&data)
                    .map_err(|e| [Entity]Error::repository_error(format!("Deserialization failed: {}", e)))?;
                Ok(Some(entity))
            }
            Ok(None) => Ok(None),
            Err(e) => Err([Entity]Error::repository_error(format!("Failed to get: {}", e))),
        }
    }

    // REQUIRED: Implement all repository methods
}
```

**REQUIREMENTS:**
- MUST use `spin_sdk::key_value::Store`
- MUST handle all serialization/deserialization errors
- Key format MUST be: `"[entity]:{id}"`
- MUST implement all repository trait methods

### 6. HTTP MODELS (src/inbound/http/models/)

**REQUIRED PATTERN:**
```rust
// src/inbound/http/models/[entity].rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domain::models as domain;

#[derive(Serialize, ToSchema)]
#[schema(example = serde_json::json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "field": "value",
    "createdAt": "2025-09-26T10:00:00Z"
}))]
pub struct [Entity]Response {
    pub id: String,
    // ... fields with HTTP naming (camelCase)
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct Create[Entity]HttpRequest {
    // ... HTTP fields
}

#[derive(Deserialize, ToSchema)]
pub struct Update[Entity]HttpRequest {
    // ... HTTP fields
}

// REQUIRED: Conversion implementations
impl From<domain::[Entity]> for [Entity]Response {
    fn from(entity: domain::[Entity]) -> Self {
        Self {
            id: entity.id.0.to_string(),
            // ... map all fields
            created_at: entity.created_at.to_rfc3339(),
        }
    }
}

impl TryFrom<Create[Entity]HttpRequest> for domain::Create[Entity]Request {
    type Error = String;

    fn try_from(request: Create[Entity]HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            // ... validate and convert fields using domain value objects
        })
    }
}
```

**REQUIREMENTS:**
- HTTP models MUST be separate from domain models
- MUST use camelCase for JSON fields (with serde rename)
- MUST provide From/TryFrom conversions
- MUST use utoipa ToSchema derive

### 7. HTTP HANDLERS (src/inbound/http/handlers/)

**REQUIRED PATTERN:**
```rust
// src/inbound/http/handlers/[entity].rs
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use uuid::Uuid;
use crate::{
    domain::{models::*, services::*, errors::*},
    inbound::http::models::[entity]::*,
};

#[utoipa::path(
    post,
    path = "/api/[entities]",
    tags = ["[entities]"],
    summary = "Create new [entity]",
    request_body = Create[Entity]HttpRequest,
    responses(
        (status = 201, description = "Created", body = [Entity]Response),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Already exists"),
        (status = 500, description = "Internal error")
    )
)]
pub async fn create_[entity]<S: [Entity]Service>(
    req: Request,
    _params: Params,
    service: S
) -> anyhow::Result<impl IntoResponse> {
    // REQUIRED: Parse HTTP request
    let body: Create[Entity]HttpRequest = serde_json::from_slice(req.body())
        .map_err(|_| anyhow::anyhow!("Invalid JSON"))?;

    // REQUIRED: Convert to domain model
    let domain_request = body.try_into()
        .map_err(|e| anyhow::anyhow!("Invalid request: {}", e))?;

    // REQUIRED: Call domain service
    match service.create(domain_request).await {
        Ok(entity) => {
            let response = [Entity]Response::from(entity);
            Ok(ResponseBuilder::new(201)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&response)?)
                .build())
        }
        Err([Entity]Error::InvalidData { message }) => {
            Ok(ResponseBuilder::new(400)
                .body(format!(r#"{{"error": "{}"}}"#, message))
                .build())
        }
        Err([Entity]Error::Duplicate { .. }) => {
            Ok(ResponseBuilder::new(409)
                .body(r#"{"error": "Already exists"}"#)
                .build())
        }
        Err(_) => {
            Ok(ResponseBuilder::new(500)
                .body(r#"{"error": "Internal server error"}"#)
                .build())
        }
    }
}

// REQUIRED: Implement all CRUD handlers with utoipa path annotations
```

**REQUIREMENTS:**
- ALL handlers MUST have utoipa path annotations
- MUST be generic over service trait
- MUST handle all domain error types with appropriate HTTP status codes
- MUST validate and convert between HTTP and domain models

### 8. OPENAPI DOCUMENTATION (src/inbound/http/handlers/docs.rs)

**REQUIRED PATTERN:**
```rust
use utoipa::{OpenApi, ServerBuilder};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "[App Name] API",
        description = "API built with hexagonal architecture and Spin SDK",
        version = "1.0.0",
        contact(
            name = "Development Team",
            email = "dev@example.com"
        )
    ),
    tags(
        (name = "[entities]", description = "[Entity] management endpoints")
    ),
    paths(
        crate::inbound::http::handlers::[entity]::create_[entity],
        crate::inbound::http::handlers::[entity]::get_[entity],
        // ... all handlers
    ),
    components(
        schemas(
            crate::domain::models::[entity]::[Entity],
            crate::inbound::http::models::[entity]::[Entity]Response,
            crate::inbound::http::models::[entity]::Create[Entity]HttpRequest,
            // ... all models
        )
    )
)]
struct ApiDoc;

pub fn get_openapi_spec(_req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let openapi = ApiDoc::openapi();
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(openapi.to_pretty_json()?)
        .build())
}
```

**REQUIREMENTS:**
- MUST include all handlers in paths
- MUST include all models in components/schemas
- MUST be accessible at `/docs/openapi.json`

### 9. SPIN TEST CONFIGURATION (spin.toml)

**REQUIRED PATTERN:**
```toml
spin_manifest_version = 2

[application]
name = "[app-name]"
version = "1.0.0"

[[trigger.http]]
route = "/api/..."
component = "[app-name]"

[component.[app-name]]
source = "target/wasm32-wasip1/release/[app_name].wasm"
allowed_outbound_hosts = []

[component.[app-name].build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]

[component.[app-name].tool.spin-test]
source = "tests/target/wasm32-wasip1/release/tests.wasm"
build = "cd tests && cargo build --target wasm32-wasip1 --release"
```

### 10. TESTING REQUIREMENTS (tests/src/lib.rs)

**REQUIRED PATTERN:**
```rust
use spin_test_sdk::{spin_test, bindings::fermyon::spin_test_virt::http_helper};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

// REQUIRED: Mock repository for testing
#[derive(Clone)]
struct Mock[Entity]Repository {
    entities: Arc<Mutex<Vec<domain::[Entity]>>>,
    should_fail: Arc<Mutex<bool>>,
}

#[async_trait]
impl [Entity]Repository for Mock[Entity]Repository {
    async fn create(&self, request: domain::Create[Entity]Request) -> Result<domain::[Entity], domain::[Entity]Error> {
        if *self.should_fail.lock().await {
            return Err(domain::[Entity]Error::repository_error("Mock failure"));
        }

        let entity = domain::[Entity] {
            id: domain::[Entity]Id(uuid::Uuid::new_v4()),
            // ... create from request
        };

        self.entities.lock().await.push(entity.clone());
        Ok(entity)
    }

    // ... implement all methods
}

// REQUIRED: Test domain service
#[spin_test]
async fn test_create_[entity]_success() {
    let mock_repo = Mock[Entity]Repository::new();
    let mock_notifier = Mock[Entity]NotificationService::new();
    let service = [Entity]ServiceImpl::new(mock_repo.clone(), mock_notifier);

    let request = domain::Create[Entity]Request {
        // ... test data
    };

    let result = service.create(request).await;
    assert!(result.is_ok());
}

// REQUIRED: Test HTTP integration
#[spin_test]
fn test_http_create_[entity]() {
    let request_body = r#"{"field": "value"}"#;

    let response = http_helper::make_request(
        http_helper::Method::Post,
        "/api/[entities]",
        &[("content-type", "application/json")],
        Some(request_body.as_bytes())
    );

    assert_eq!(response.status, 201);
}
```

**REQUIREMENTS:**
- MUST test domain service with mocked dependencies
- MUST test HTTP endpoints with spin-test
- MUST test error scenarios
- MUST verify OpenAPI spec compliance

### 11. APPLICATION ASSEMBLY (src/lib.rs)

**REQUIRED PATTERN:**
```rust
use spin_sdk::http::{IntoResponse, Request, Router};
use crate::{
    domain::services::[Entity]ServiceImpl,
    outbound::repositories::kv_[entity]_repository::Kv[Entity]Repository,
    outbound::notifications::Email[Entity]Notifier,
    inbound::http::handlers,
};

#[derive(Clone)]
struct AppState<S: [Entity]Service> {
    [entity]_service: S,
}

#[spin_sdk::http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // REQUIRED: Composition root - assemble dependencies here
    let repository = Kv[Entity]Repository::new()?;
    let notifier = Email[Entity]Notifier::new();
    let [entity]_service = [Entity]ServiceImpl::new(repository, notifier);

    let state = AppState { [entity]_service };

    let mut router = Router::default();

    // REQUIRED: Register all routes
    router.post("/api/[entities]", {
        let service = state.[entity]_service.clone();
        move |req, params| async move {
            handlers::[entity]::create_[entity](req, params, service).await
        }
    });

    router.get("/docs/openapi.json", handlers::docs::get_openapi_spec);

    Ok(router.handle(req))
}
```

**REQUIREMENTS:**
- Dependency injection MUST happen in main/lib.rs
- ALL dependencies MUST be composed at application startup
- Router MUST register all API endpoints
- OpenAPI spec MUST be available at `/docs/openapi.json`

## ACCEPTANCE CRITERIA

When implementing a Spin application following this spec, you MUST:

1. ✅ **Domain Isolation**: Domain models and services NEVER import from `outbound/` or `inbound/`
2. ✅ **Interface Segregation**: All external dependencies accessed through traits only
3. ✅ **Error Handling**: Complete domain error types with proper HTTP status mapping
4. ✅ **Testing**: Mock all ports for unit tests, integration tests with spin-test
5. ✅ **Documentation**: Complete OpenAPI spec generated from utoipa annotations
6. ✅ **Storage Abstraction**: Repository pattern allows swapping KV → SQLite → PostgreSQL
7. ✅ **Dependency Flow**: All dependencies point inward toward domain
8. ✅ **Separation of Concerns**: HTTP models separate from domain models

## MIGRATION STRATEGY

To change storage technology, ONLY modify the composition root in `src/lib.rs`:

```rust
// Current: KV Storage
let repository = Kv[Entity]Repository::new()?;

// Future: SQLite
let repository = Sqlite[Entity]Repository::new("sqlite://app.db").await?;

// Future: PostgreSQL
let repository = Postgres[Entity]Repository::new("postgresql://...").await?;
```

NO other code changes required. Domain, services, handlers, and tests remain unchanged.

## UTOIPA (OpenAPI DOCUMENTATION)

### Cargo.toml Setup

The project uses `utoipa` for auto-generated OpenAPI specs. Dependencies in `Cargo.toml`:

```toml
[dependencies]
utoipa = { version = "5.4.0", features = ["uuid", "chrono"] }
utoipa-swagger-ui = "9.0.2"
```

**REQUIREMENTS:**
- `utoipa` MUST include `uuid` and `chrono` features for proper schema generation of `Uuid` and `DateTime<Utc>` fields
- `utoipa-swagger-ui` provides the `/docs` Swagger UI endpoint

### ToSchema on Domain Models

All domain models that appear in API requests or responses MUST derive `ToSchema`:

```rust
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = serde_json::json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "caseNumber": "1:24-cr-00001",
    "title": "United States v. Smith"
}))]
pub struct CriminalCase {
    pub id: CaseId,
    pub case_number: CaseNumber,
    pub title: CaseTitle,
    // ...
}
```

**REQUIREMENTS:**
- Every domain model used in an API response MUST derive `ToSchema`
- Every HTTP request/response model MUST derive `ToSchema`
- Use `#[schema(example = ...)]` to provide meaningful example values
- Value objects (`CaseId`, `CaseNumber`, etc.) MUST also derive `ToSchema`
- Nested types referenced by `ToSchema` structs MUST themselves derive `ToSchema`

### Handler Path Annotations

Every HTTP handler MUST have a full `#[utoipa::path]` annotation:

```rust
#[utoipa::path(
    get,
    path = "/api/courts/{district}/cases/{id}",
    tags = ["cases"],
    summary = "Get a criminal case by ID",
    description = "Retrieves a single criminal case record by its UUID, scoped to the specified court district.",
    params(
        ("district" = String, Path, description = "Court district code (e.g., SDNY, EDTX)"),
        ("id" = String, Path, description = "Case UUID")
    ),
    responses(
        (status = 200, description = "Case found", body = CriminalCaseResponse),
        (status = 404, description = "Case not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_case<S: CaseService>(
    req: Request,
    params: Params,
    service: S,
) -> anyhow::Result<impl IntoResponse> {
    // ...
}
```

**REQUIREMENTS:**
- MUST specify HTTP method (`get`, `post`, `put`, `delete`)
- MUST specify the full path with path parameters in `{braces}`
- MUST include `tags` for Swagger UI grouping
- MUST include `summary` (short) and optionally `description` (detailed)
- MUST list all path parameters with `params(...)`
- MUST list all possible response status codes with `responses(...)`
- Response bodies MUST reference the HTTP response model type, not domain type
- Error responses MUST reference `ErrorResponse` or equivalent

### OpenAPI Spec Assembly (docs.rs)

The `ApiDoc` struct in `src/handlers/docs.rs` MUST register all paths and schemas:

```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Lexodus - Federal Judicial Case Management API",
        description = "Complete federal court case management with 250+ REST endpoints",
        version = "5.0.0",
        contact(
            name = "Tyler Harpool",
            email = "tylerharpool@gmail.com"
        )
    ),
    tags(
        (name = "cases", description = "Criminal case management"),
        (name = "judges", description = "Judge management"),
        (name = "attorneys", description = "Attorney management"),
        (name = "deadlines", description = "Deadline tracking"),
        (name = "docket", description = "Docket & calendar"),
        (name = "orders", description = "Judicial orders"),
        (name = "opinions", description = "Judicial opinions"),
        (name = "rules", description = "Rules engine"),
        (name = "filing", description = "Electronic filing"),
        (name = "sentencing", description = "Sentencing management"),
        (name = "health", description = "Health & monitoring"),
        (name = "admin", description = "Administrative endpoints")
    ),
    paths(
        // ALL handler functions must be listed here
    ),
    components(
        schemas(
            // ALL ToSchema models must be listed here
        )
    )
)]
struct ApiDoc;
```

**REQUIREMENTS:**
- Every handler function MUST be listed in `paths(...)`
- Every `ToSchema` model used in requests/responses MUST be listed in `components(schemas(...))`
- Tags MUST match the tags used in handler `#[utoipa::path]` annotations
- API version MUST match `Cargo.toml` version
- Spec MUST be served at `/docs/openapi.json`
- Swagger UI MUST be served at `/docs`

### Keeping the Spec in Sync

When adding new endpoints or models:

1. **New handler** → Add `#[utoipa::path]` annotation → Add to `paths(...)` in `ApiDoc`
2. **New domain model** → Derive `ToSchema` → Add to `components(schemas(...))` in `ApiDoc`
3. **New HTTP model** → Derive `ToSchema` → Add to `components(schemas(...))` in `ApiDoc`
4. **Modified model** → Ensure `#[schema(example)]` is updated
5. **Removed endpoint** → Remove from both handler and `ApiDoc`

**Compile-time enforcement:** If a `#[utoipa::path]` references a type not in `components(schemas(...))`, the build will fail. Use this as a safety net.

## TESTING REQUIREMENTS (4 Layers)

### Layer 1: Unit Tests — Pure Domain Logic

Test domain models, value objects, and validation in isolation. No external dependencies.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_number_validation_valid() {
        let result = CaseNumber::new("1:24-cr-00001".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "1:24-cr-00001");
    }

    #[test]
    fn test_case_number_validation_empty() {
        let result = CaseNumber::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_case_number_validation_trims_whitespace() {
        let result = CaseNumber::new("  1:24-cr-00001  ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "1:24-cr-00001");
    }

    #[test]
    fn test_entity_serialization_roundtrip() {
        let case = CriminalCase {
            id: CaseId(Uuid::new_v4()),
            case_number: CaseNumber("1:24-cr-00001".to_string()),
            // ... all fields
        };
        let json = serde_json::to_string(&case).unwrap();
        let deserialized: CriminalCase = serde_json::from_str(&json).unwrap();
        assert_eq!(case.id.0, deserialized.id.0);
    }

    #[test]
    fn test_entity_deserialization_from_json() {
        let json = r#"{"id": "550e8400-e29b-41d4-a716-446655440000", "caseNumber": "1:24-cr-00001"}"#;
        let result: Result<CriminalCase, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        let err = ApiError::NotFound { id: "abc".to_string() };
        assert!(format!("{}", err).contains("abc"));
    }
}
```

**REQUIREMENTS:**
- Located inline in domain model files with `#[cfg(test)]`
- Test all value object validation (valid, invalid, edge cases)
- Test serialization roundtrips (serialize → deserialize → assert equal)
- Test deserialization from raw JSON strings
- Test error type Display implementations
- NO external dependencies (no KV store, no HTTP, no network)

### Layer 2: Service Tests — Domain Services with Mocked Ports

Test business logic orchestration using mock implementations of repository and notification traits.

```rust
// tests/src/service_tests.rs (or inline in service files)
#[derive(Clone)]
struct MockCaseRepository {
    cases: Arc<Mutex<Vec<CriminalCase>>>,
    should_fail: bool,
}

impl MockCaseRepository {
    fn new() -> Self {
        Self {
            cases: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
        }
    }

    fn with_failure() -> Self {
        Self {
            cases: Arc::new(Mutex::new(Vec::new())),
            should_fail: true,
        }
    }

    fn with_seed_data(cases: Vec<CriminalCase>) -> Self {
        Self {
            cases: Arc::new(Mutex::new(cases)),
            should_fail: false,
        }
    }
}

#[async_trait]
impl CaseRepository for MockCaseRepository {
    async fn create(&self, request: CreateCaseRequest) -> Result<CriminalCase, ApiError> {
        if self.should_fail {
            return Err(ApiError::RepositoryError { message: "Mock failure".into() });
        }
        // ... create and store entity
    }
    // ... implement all methods
}

// Happy path
#[spin_test]
async fn test_create_case_success() {
    let repo = MockCaseRepository::new();
    let notifier = MockNotifier::new();
    let service = CaseServiceImpl::new(repo.clone(), notifier);

    let request = CreateCaseRequest { /* ... */ };
    let result = service.create(request).await;

    assert!(result.is_ok());
    let case = result.unwrap();
    assert!(!case.id.0.is_nil());
}

// Error path
#[spin_test]
async fn test_create_case_repository_failure() {
    let repo = MockCaseRepository::with_failure();
    let notifier = MockNotifier::new();
    let service = CaseServiceImpl::new(repo, notifier);

    let request = CreateCaseRequest { /* ... */ };
    let result = service.create(request).await;

    assert!(result.is_err());
}

// Notification failure should not fail the operation
#[spin_test]
async fn test_create_case_notification_failure_succeeds() {
    let repo = MockCaseRepository::new();
    let notifier = MockNotifier::with_failure();
    let service = CaseServiceImpl::new(repo, notifier);

    let request = CreateCaseRequest { /* ... */ };
    let result = service.create(request).await;

    assert!(result.is_ok()); // Should succeed despite notification failure
}
```

**REQUIREMENTS:**
- Mock ALL repository and notification traits
- Mock types MUST implement `Clone` (use `Arc<Mutex<>>` for shared state)
- Provide `new()`, `with_failure()`, and `with_seed_data()` constructors
- Test happy path for every service method
- Test error paths (repository failures, not-found, duplicates)
- Verify notification failures do NOT cause operation failures
- NO real KV store access — mocks only

### Layer 3: HTTP Integration Tests — spin-test Endpoint Tests

Test full HTTP request/response cycle through the Spin runtime using `spin-test-sdk`.

```rust
// tests/src/lib.rs
use spin_test_sdk::{spin_test, bindings::fermyon::spin_test_virt::http_helper};

#[spin_test]
fn test_create_case_http_201() {
    let body = r#"{
        "caseNumber": "1:24-cr-00001",
        "title": "United States v. Smith",
        "district": "SDNY"
    }"#;

    let response = http_helper::make_request(
        http_helper::Method::Post,
        "/api/cases",
        &[("content-type", "application/json")],
        Some(body.as_bytes()),
    );

    assert_eq!(response.status, 201);
    let result: serde_json::Value = serde_json::from_slice(&response.body.unwrap()).unwrap();
    assert!(result.get("id").is_some());
    assert_eq!(result["caseNumber"], "1:24-cr-00001");
}

#[spin_test]
fn test_create_case_http_400_invalid_json() {
    let response = http_helper::make_request(
        http_helper::Method::Post,
        "/api/cases",
        &[("content-type", "application/json")],
        Some(b"not-json"),
    );

    assert_eq!(response.status, 400);
}

#[spin_test]
fn test_get_case_http_404() {
    let response = http_helper::make_request(
        http_helper::Method::Get,
        "/api/cases/00000000-0000-0000-0000-000000000000",
        &[],
        None,
    );

    assert_eq!(response.status, 404);
}

#[spin_test]
fn test_openapi_spec_available() {
    let response = http_helper::make_request(
        http_helper::Method::Get,
        "/docs/openapi.json",
        &[],
        None,
    );

    assert_eq!(response.status, 200);
    let spec: serde_json::Value = serde_json::from_slice(&response.body.unwrap()).unwrap();
    assert!(spec.get("openapi").is_some());
    assert!(spec.get("paths").is_some());
}

// URL-based multi-tenant routes
#[spin_test]
fn test_create_case_url_tenant_201() {
    let body = r#"{"caseNumber": "1:24-cr-00001", "title": "United States v. Smith"}"#;

    let response = http_helper::make_request(
        http_helper::Method::Post,
        "/api/courts/sdny/cases",
        &[("content-type", "application/json")],
        Some(body.as_bytes()),
    );

    assert_eq!(response.status, 201);
}
```

**REQUIREMENTS:**
- Located in `tests/src/lib.rs` and submodule files
- Uses `spin_test_sdk` with `#[spin_test]` attribute
- Test every endpoint: create (201), get (200), list (200), update (200), delete (204)
- Test error cases: invalid JSON (400), not found (404), duplicate (409)
- Test both legacy header-based routes AND URL-based multi-tenant routes
- Verify OpenAPI spec endpoint returns valid JSON with `openapi` and `paths` keys
- Verify response body structure (required fields present, correct types)
- Tests build to `tests/target/wasm32-wasip1/release/` WASM target

### Layer 4: E2E Tests (Optional)

For critical user flows, optional end-to-end tests that exercise the full deployed application.

```bash
# Run against a locally deployed Spin app
spin up &
sleep 2

# Test case lifecycle
CASE_ID=$(curl -s -X POST http://localhost:3000/api/cases \
  -H "Content-Type: application/json" \
  -d '{"caseNumber":"1:24-cr-00001","title":"Test Case"}' | jq -r '.id')

curl -s http://localhost:3000/api/cases/$CASE_ID | jq '.caseNumber'
# Expected: "1:24-cr-00001"

curl -s -X DELETE http://localhost:3000/api/cases/$CASE_ID
# Expected: 204
```

**REQUIREMENTS:**
- Only for critical workflows (case lifecycle, filing pipeline, deadline chains)
- Run against `spin up` local deployment
- NOT required in CI — run manually before major releases
- Document expected outputs inline

### Running Tests Cheat Sheet

```bash
# Layer 1: Unit tests (pure domain logic)
cargo test --lib

# Layer 1+2: All non-integration tests
cargo test

# Layer 3: spin-test integration tests (requires spin-test CLI)
spin test

# Layer 3: Build test WASM only (for CI)
cd tests && cargo build --target wasm32-wasip1 --release

# Run a specific test
cargo test test_case_number_validation

# Run tests for a specific module
cargo test domain::criminal_case::tests

# Layer 4: E2E (manual)
spin up &
./e2e-tests.sh
```

**CI Pipeline MUST:**
1. Run `cargo test --lib` (unit + service tests)
2. Run `spin test` (HTTP integration tests)
3. Fail the build on any test failure
4. Report test results in PR checks

## CODE QUALITY STANDARDS

rust// CORRECT: Improve existing src/domain/models/attorney.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Attorney {
    pub id: AttorneyId,
    pub name: AttorneyName,
    pub license_number: LicenseNumber,
    // Enhanced: Added new fields during refactoring
    pub specializations: Vec<Specialization>,
    pub bar_admissions: Vec<BarAdmission>,
}

// WRONG: Don't create src/domain/models/attorney-enhanced.rs
Architecture Consistency
When refactoring, maintain the hexagonal architecture:

Domain files stay in src/domain/
Adapter files stay in src/adapters/
Keep the same interfaces (repository traits, service traits)
Enhance implementations without breaking contracts

This ensures your refactored code fits seamlessly into the existing hexagonal structure.
