//! HTTP request handlers for the ToDo and Criminal Case APIs
//!
//! This module contains all the HTTP request handlers for the API endpoints,
//! including the ToDo CRUD operations, Criminal Case management,
//! and API documentation endpoints.

/// Administrative handlers for multi-tenant operations
pub(crate) mod admin;
/// Attorney and party management handlers
pub(crate) mod attorney;
/// Criminal case management handlers (demonstrating hexagonal architecture)
pub(crate) mod criminal_case;
/// Deadline tracking and compliance handlers
pub(crate) mod deadline;
/// Docket and calendar management handlers
pub(crate) mod docket;
/// API documentation handlers
pub mod docs;
/// Feature flag management handlers
pub(crate) mod features;
/// Health check endpoint
pub(crate) mod health;
/// Judge management handlers
pub(crate) mod judge;
/// Judicial opinion management handlers
pub(crate) mod opinion;
/// Judicial order management handlers
pub(crate) mod order;
/// PDF generation handlers for court documents
pub(crate) mod pdf_working;
/// Federal forms generation handlers
pub(crate) mod federal_forms;
/// Batch PDF generation handlers
pub(crate) mod pdf_batch;
/// Federal sentencing management handlers
pub(crate) mod sentencing;
/// ToDo item CRUD operation handlers
pub(crate) mod todo;
