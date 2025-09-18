//! Adapters (implementations) for hexagonal architecture
//!
//! This module contains concrete implementations of the ports,
//! handling the actual integration with external systems.

pub mod spin_kv_attorney_repository;
pub mod spin_kv_case_repository;
pub mod spin_kv_deadline_repository;
pub mod spin_kv_docket_repository;
pub mod spin_kv_document_repository;
pub mod spin_kv_judge_repository;
pub mod spin_kv_sentencing_repository;