//! HTTP request handlers for the ToDo API
//!
//! This module contains all the HTTP request handlers for the API endpoints,
//! including the ToDo CRUD operations and API documentation endpoints.

/// API documentation handlers
pub mod docs;
/// Health check endpoint
pub(crate) mod health;
/// ToDo item CRUD operation handlers
pub(crate) mod todo;
