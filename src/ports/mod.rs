//! Ports (interfaces) for hexagonal architecture
//!
//! This module defines the trait interfaces that represent the boundaries
//! of our application. These ports allow the core domain to interact with
//! external systems without depending on their implementations.

pub mod attorney_repository;
pub mod case_repository;
pub mod config_repository;
pub mod deadline_engine;
pub mod deadline_repository;
pub mod docket_repository;
pub mod document_generator;
pub mod document_repository;
pub mod feature_repository;
pub mod judge_repository;
pub mod rules_engine;
pub mod rules_repository;
pub mod sentencing_repository;
pub mod privacy_engine;
pub mod signature_repository;