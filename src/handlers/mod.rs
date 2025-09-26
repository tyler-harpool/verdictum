//! HTTP request handlers for the ToDo and Criminal Case APIs
//!
//! This module contains all the HTTP request handlers for the API endpoints,
//! including the ToDo CRUD operations, Criminal Case management,
//! and API documentation endpoints.

/// Administrative handlers for multi-tenant operations
pub(crate) mod admin;
/// Attorney and party management handlers
pub(crate) mod attorney;
/// URL-based attorney handlers (for migration)
pub(crate) mod attorney_url;
/// Configuration management handlers
pub(crate) mod config;
/// URL-based configuration handlers (for migration)
pub(crate) mod config_url;
/// Criminal case management handlers (demonstrating hexagonal architecture)
pub(crate) mod criminal_case;
/// URL-based criminal case handlers (for migration)
pub(crate) mod criminal_case_url;
/// Deadline tracking and compliance handlers
pub(crate) mod deadline;
/// URL-based deadline handlers (for migration)
pub(crate) mod deadline_url;
/// Docket and calendar management handlers
pub(crate) mod docket;
/// URL-based docket handlers (for migration)
pub(crate) mod docket_url;
/// API documentation handlers
pub mod docs;
/// Feature flag management handlers
pub(crate) mod features;
/// Health check endpoint
pub(crate) mod health;
/// Judge management handlers
pub(crate) mod judge;
/// URL-based judge handlers (for migration)
pub(crate) mod judge_url;
/// Judicial opinion management handlers
pub(crate) mod opinion;
/// URL-based opinion handlers (for migration)
pub(crate) mod opinion_url;
/// Judicial order management handlers
pub(crate) mod order;
/// URL-based order handlers (for migration)
pub(crate) mod order_url;
/// PDF generation handlers using hexagonal architecture
pub(crate) mod pdf_hexagonal;
/// Federal sentencing management handlers
pub(crate) mod sentencing;
/// URL-based sentencing handlers (for migration)
pub(crate) mod sentencing_url;
/// ToDo item CRUD operation handlers
pub(crate) mod todo;
