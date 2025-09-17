//! Spin ToDo API - A RESTful API for managing ToDo items
//!
//! This application provides a complete CRUD API for ToDo items using
//! Spin's WebAssembly runtime and key-value store for persistence.
//!
//! ## Features
//! - Create, read, update, and delete ToDo items
//! - Toggle completion status of ToDo items
//! - Persistent storage using Spin's key-value store
//! - Interactive OpenAPI documentation with Swagger UI
//! - Soft delete functionality (items are marked as deleted, not removed)
//!
//! ## API Endpoints
//! - `GET /api/todos` - Retrieve all ToDo items
//! - `GET /api/todos/:id` - Get a specific ToDo item
//! - `POST /api/todos` - Create a new ToDo item
//! - `POST /api/todos/:id/toggle` - Toggle completion status
//! - `DELETE /api/todos/:id` - Delete a ToDo item (soft delete)
//! - `GET /docs` - Interactive API documentation
//! - `GET /docs/openapi-description.json` - OpenAPI specification

use spin_sdk::http::{IntoResponse, Request, Router};
use spin_sdk::http_component;

mod domain;
mod handlers;

/// Main HTTP component handler for the Spin ToDo API
///
/// This function sets up the router with all API endpoints and documentation routes.
/// It's the entry point for all HTTP requests to the application.
#[http_component]
fn handle_spin_todo_api(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::default();

    // ToDo API endpoints
    router.get("/api/todos", handlers::todo::get_all);
    router.get("/api/todos/:id", handlers::todo::get_by_id);
    router.post("/api/todos", handlers::todo::create_todo);
    router.post("/api/todos/:id/toggle", handlers::todo::toggle_by_id);
    router.delete("/api/todos/:id", handlers::todo::delete_by_id);

    // Documentation endpoints
    router.get(
        "/docs/openapi-description.json",
        handlers::docs::get_openapi_description,
    );
    router.get("/docs/*", handlers::docs::render_openapi_docs_ui);

    Ok(router.handle(req))
}
