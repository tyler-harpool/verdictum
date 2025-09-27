//! Attorney domain tests
//!
//! This module contains tests for attorney management endpoints

// Core CRUD operations
pub mod create_attorney;
pub mod get_attorney_by_id;
pub mod update_attorney;
pub mod delete_attorney;
pub mod list_attorneys;
pub mod search_attorneys;
pub mod pagination_tests;
pub mod attorney_case_tests;
pub mod representation_history_tests;
pub mod conflict_check_tests;