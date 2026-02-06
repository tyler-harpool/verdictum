//! Rules engine domain tests
//!
//! This module contains tests for rules engine management endpoints

// Core CRUD operations
pub mod create_rule;
pub mod get_rule;
pub mod update_rule;
pub mod delete_rule;

// Query and filter operations
pub mod query_rules;

// URL-based route operations
pub mod url_routes;

// Rules evaluation engine tests
pub mod evaluate_rules;
