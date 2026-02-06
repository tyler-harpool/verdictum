//! Adapters (implementations) for hexagonal architecture
//!
//! This module contains concrete implementations of the ports,
//! handling the actual integration with external systems.

pub mod deadline_engine_impl;
pub mod pdf_writer_adapter;
pub mod store_utils;
pub mod spin_kv_attorney_repository;
pub mod spin_kv_case_repository;
pub mod spin_kv_config_repository;
pub mod spin_kv_deadline_repository;
pub mod spin_kv_docket_repository;
pub mod spin_kv_document_repository;
pub mod spin_kv_judge_repository;
pub mod rule_loader;
pub mod rules_engine_impl;
pub mod spin_kv_rules_repository;
pub mod spin_kv_sentencing_repository;
pub mod spin_kv_signature_repository;
pub mod toml_config_loader;
pub mod privacy_engine_impl;
pub mod unified_config_feature_repository;