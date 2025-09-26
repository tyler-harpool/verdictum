//! Common utilities for Spin KV Store adapters
//!
//! This module provides shared functionality for working with
//! Spin's key-value stores, particularly for multi-tenant scenarios.

use spin_sdk::key_value::Store;
use anyhow::{Result, anyhow};

/// Validates the store name and opens the store
///
/// This function ensures that:
/// 1. The store name is not empty
/// 2. The store name is not the special "tenant_not_specified" value
/// 3. The store can be successfully opened
pub fn open_validated_store(store_name: &str) -> Result<Store> {
    // Check for invalid store names
    if store_name.is_empty() {
        return Err(anyhow!("Store name cannot be empty"));
    }

    if store_name == "tenant_not_specified" || store_name == "TENANT_NOT_SPECIFIED" {
        return Err(anyhow!(
            "Missing required header: X-Court-District or X-Tenant-ID. \
             Please specify a valid district identifier."
        ));
    }

    // Attempt to open the store
    Store::open(store_name).map_err(|e| {
        anyhow!("Failed to open store '{}': {}", store_name, e)
    })
}

/// Checks if a store name is valid for opening
pub fn is_valid_store_name(store_name: &str) -> bool {
    !store_name.is_empty()
        && store_name != "tenant_not_specified"
        && store_name != "TENANT_NOT_SPECIFIED"
}