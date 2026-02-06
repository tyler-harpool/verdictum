//! Criminal case domain tests
//!
//! This module contains tests for criminal case management endpoints

// Core CRUD operations
pub mod create_case;
pub mod get_case;
pub mod update_case;
pub mod delete_case;

// Case-specific operations
pub mod case_operations;
pub mod court_events;
pub mod motions;

// Search and query operations
pub mod search_cases;

// Domain enhancement features
pub mod docket_entries;
pub mod evidence;
pub mod sealed_cases;
pub mod speedy_trial;
pub mod victims;