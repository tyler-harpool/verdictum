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

## API Documentation

### Interactive Documentation

Once the application is running, you can access the interactive Swagger UI documentation at:
- Local: `http://localhost:3000/docs`
- OpenAPI JSON: `http://localhost:3000/docs/openapi-description.json`

### API Endpoints

#### Get All ToDo Items
```http
GET /api/todos
```
Returns all active (non-deleted) ToDo items.

**Response:** `200 OK`
```json
[
  {
    "id": "059c7906-ce72-4433-94df-441beb14d96a",
    "contents": "Buy groceries",
    "isCompleted": false
  }
]
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

**Response:** `201 Created`
- Headers: `Location: /api/todos/{id}`
```json
{
  "id": "059c7906-ce72-4433-94df-441beb14d96a",
  "contents": "Buy groceries",
  "isCompleted": false
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
│   ├── domain/
│   │   ├── mod.rs          # Domain module exports
│   │   └── todo.rs         # ToDo entity and business logic
│   └── handlers/
│       ├── mod.rs          # Handler utilities and common code
│       ├── todo.rs         # ToDo CRUD operation handlers
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
- **Infrastructure**: Spin's key-value store for persistence

### Storage

ToDo items are stored in Spin's key-value store with keys prefixed by `todo-` followed by the UUID. The storage mechanism supports:
- Automatic serialization/deserialization to JSON
- Soft delete functionality
- UUID-based retrieval

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).

## Author

Tyler Harpool - [GitHub](https://github.com/tyler-harpool)

## Acknowledgments

- Built with [Fermyon Spin](https://www.fermyon.com/spin)
- Documentation powered by [utoipa](https://github.com/juhaku/utoipa)
- Based on the guide: [OpenAPI Docs for Spin with Rust](https://www.fermyon.com/blog/openapi-docs-for-spin-with-rust)
