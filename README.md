# Spin ToDo API

A RESTful API for managing ToDo items built with [Spin](https://github.com/fermyon/spin) and Rust. This application demonstrates how to build a complete CRUD API using WebAssembly and Spin's key-value store for persistence.

## Features

- 🚀 **Fast and Lightweight** - Built with Rust and compiled to WebAssembly
- 💾 **Persistent Storage** - Uses Spin's built-in key-value store
- 📝 **Complete CRUD Operations** - Create, Read, Update, and Delete ToDo items
- 🔄 **Toggle Completion** - Mark ToDo items as completed or incomplete
- 🗑️ **Soft Delete** - Items are marked as deleted rather than permanently removed
- 📚 **OpenAPI Documentation** - Interactive API documentation with Swagger UI
- 🆔 **UUID-based IDs** - Each ToDo item gets a unique UUID identifier
- 📄 **Pagination** - Efficient handling of large ToDo lists with configurable page sizes
- 🔍 **Filtering** - Filter ToDo items by completion status
- ❤️ **Health Check** - Monitoring endpoint for deployment and health checks
- ✅ **Input Validation** - Validates request data with meaningful error messages
- 🛡️ **Error Handling** - Comprehensive error handling with typed errors and consistent JSON responses

## API Documentation

### Interactive Documentation

Once the application is running, you can access the interactive Swagger UI documentation at:
- Local: `http://localhost:3000/docs`
- OpenAPI JSON: `http://localhost:3000/docs/openapi-description.json`

### API Endpoints

#### Get All ToDo Items (with Pagination)
```http
GET /api/todos?page=1&limit=20&completed=false
```
Returns a paginated list of active (non-deleted) ToDo items.

**Query Parameters:**
- `page` (optional): Page number (default: 1, min: 1)
- `limit` (optional): Items per page (default: 20, min: 1, max: 100)
- `completed` (optional): Filter by completion status (true/false)

**Response:** `200 OK`
```json
{
  "items": [
    {
      "id": "059c7906-ce72-4433-94df-441beb14d96a",
      "contents": "Buy groceries",
      "isCompleted": false
    }
  ],
  "total": 42,
  "page": 1,
  "limit": 20,
  "totalPages": 3,
  "hasNext": true,
  "hasPrevious": false
}
```

#### Health Check
```http
GET /api/health
```
Check the health status of the API and storage connectivity.

**Response:** `200 OK` | `503 Service Unavailable`
```json
{
  "status": "healthy",
  "version": "2.0.0",
  "storage": "connected",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Get ToDo Item by ID
```http
GET /api/todos/:id
```
Retrieve a specific ToDo item using its UUID.

**Response:** `200 OK` | `404 Not Found` | `400 Bad Request`
```json
{
  "id": "059c7906-ce72-4433-94df-441beb14d96a",
  "contents": "Buy groceries",
  "isCompleted": false
}
```

#### Create ToDo Item
```http
POST /api/todos
Content-Type: application/json

{
  "contents": "Buy groceries"
}
```
Creates a new ToDo item with the provided contents.

**Validation:**
- Content must not be empty
- Content must not exceed 1000 characters

**Response:** `201 Created`
- Headers: `Location: /api/todos/{id}`
```json
{
  "id": "059c7906-ce72-4433-94df-441beb14d96a",
  "contents": "Buy groceries",
  "isCompleted": false
}
```

**Error Response:** `400 Bad Request`
```json
{
  "error": "Bad Request",
  "status": 400,
  "details": "ToDo content cannot be empty"
}
```

#### Toggle ToDo Completion
```http
POST /api/todos/:id/toggle
```
Toggles the completion status of a ToDo item.

**Response:** `204 No Content` | `404 Not Found` | `400 Bad Request`

#### Delete ToDo Item
```http
DELETE /api/todos/:id
```
Soft deletes a ToDo item (marks as deleted but doesn't remove from storage).

**Response:** `204 No Content` | `404 Not Found` | `400 Bad Request`

## Prerequisites

To build and run the Spin application on your local machine, you must have:

- **Spin CLI** version `3.3.1` or newer
  - Install: `curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash`
- **Rust** version `1.86.0` or newer with the `wasm32-wasip1` target
  - Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Add WASM target: `rustup target add wasm32-wasip1`

## Building & Running

### Build the Application
```bash
spin build
```

### Run Locally
```bash
spin up
```
The API will be available at `http://localhost:3000`

To use a different port:
```bash
spin up --listen 0.0.0.0:8080
```

### Deploy to Fermyon Cloud
```bash
spin deploy
```

## Project Structure

```
spin-todo-api/
├── src/
│   ├── lib.rs              # Main application entry point and router setup
│   ├── error.rs            # Custom error types and error handling
│   ├── domain/
│   │   ├── mod.rs          # Domain module exports
│   │   └── todo.rs         # ToDo entity and business logic
│   └── handlers/
│       ├── mod.rs          # Handler module exports
│       ├── todo.rs         # ToDo CRUD operation handlers with pagination
│       ├── health.rs       # Health check endpoint
│       └── docs.rs         # OpenAPI documentation handlers
├── spin.toml               # Spin application configuration
├── Cargo.toml              # Rust dependencies
└── README.md               # This file
```

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## Architecture

This application follows a clean architecture pattern with clear separation of concerns:

- **Domain Layer** (`src/domain/`): Core business logic and entities
- **Handler Layer** (`src/handlers/`): HTTP request handlers and response formatting
- **Error Layer** (`src/error.rs`): Centralized error handling with typed errors
- **Infrastructure**: Spin's key-value store for persistence

### Key Design Decisions

- **Soft Deletes**: Items are marked as deleted rather than physically removed, allowing for potential recovery
- **UUID Identifiers**: Each ToDo gets a globally unique identifier for reliable identification
- **Pagination**: Efficient handling of large datasets with configurable page sizes
- **Typed Errors**: Custom error types ensure consistent error responses and proper HTTP status codes
- **OpenAPI Integration**: All endpoints are fully documented with request/response schemas

### Storage

ToDo items are stored in Spin's key-value store with keys prefixed by `todo-` followed by the UUID. The storage mechanism supports:
- Automatic serialization/deserialization to JSON
- Soft delete functionality
- UUID-based retrieval
- Atomic operations for consistency

## API Error Handling

All error responses follow a consistent JSON structure:
```json
{
  "error": "Error Type",
  "status": 400,
  "details": "Detailed error message"
}
```

**Common Error Codes:**
- `400 Bad Request`: Invalid input data or parameters
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server-side errors
- `503 Service Unavailable`: Storage connectivity issues

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).

## Author

Tyler Harpool - [GitHub](https://github.com/tyler-harpool)

## Testing the API

### Using cURL

Remember to use quotes around URLs with query parameters:

```bash
# Create a ToDo
curl -X POST "http://localhost:3000/api/todos" \
  -H "Content-Type: application/json" \
  -d '{"contents": "Test item"}'

# Get paginated todos
curl "http://localhost:3000/api/todos?page=1&limit=5"

# Filter by completion status
curl "http://localhost:3000/api/todos?completed=false"

# Toggle completion
curl -X POST "http://localhost:3000/api/todos/{id}/toggle"

# Check health
curl "http://localhost:3000/api/health"
```

### Using Swagger UI

Navigate to `http://localhost:3000/docs` for an interactive API explorer where you can:
- View all endpoint documentation
- Test API calls directly from the browser
- See request/response schemas
- Download the OpenAPI specification

## Changelog

### Version 2.0.0
- Added pagination support for listing ToDos
- Added filtering by completion status
- Implemented comprehensive error handling with typed errors
- Added health check endpoint for monitoring
- Enhanced input validation with meaningful error messages
- Improved OpenAPI documentation with complete schemas
- Added request/response examples in documentation

### Version 1.0.0
- Initial release with basic CRUD operations
- Soft delete functionality
- UUID-based identification
- OpenAPI documentation

## Acknowledgments

- Built with [Fermyon Spin](https://www.fermyon.com/spin)
- Documentation powered by [utoipa](https://github.com/juhaku/utoipa)
- Based on the guide: [OpenAPI Docs for Spin with Rust](https://www.fermyon.com/blog/openapi-docs-for-spin-with-rust)
