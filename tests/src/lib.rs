//! Lexodus API Integration Tests
//!
//! This test suite validates the API endpoints for the Federal Court Case Management System.
//! Tests are organized by domain, with each domain containing its specific endpoint tests.

// Domain modules
pub mod monitoring;
pub mod attorney;
// TODO: Enable case module once criminal case repository is implemented
// pub mod case;
// TODO: Enable judge_tests module once judge repository is fully implemented
// pub mod judge_tests;