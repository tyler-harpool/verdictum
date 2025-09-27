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
