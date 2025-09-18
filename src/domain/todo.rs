//! Domain models and business logic for ToDo items
//!
//! This module contains the core ToDo struct and its associated methods
//! for persisting and retrieving ToDo items from Spin's key-value store.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use spin_sdk::key_value::Store;
use uuid::Uuid;
use utoipa::ToSchema;

/// Prefix used for all ToDo items in the key-value store
const KEY_PREFIX: &str = "todo-";

/// Core domain model representing a ToDo item
///
/// This struct represents a single ToDo item with its associated metadata.
/// ToDo items are persisted to Spin's key-value store with soft delete support.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ToDo {
    /// Unique identifier for the ToDo item
    pub id: Uuid,
    /// The content/description of the ToDo item
    pub contents: String,
    /// Whether the ToDo item has been marked as completed
    pub is_completed: bool,
    /// Soft delete flag - items are not physically removed from storage
    pub is_deleted: bool,
}
impl ToDo {
    /// Create a new ToDo item with the given contents
    ///
    /// The ToDo item is created with:
    /// - A new UUID v4 identifier
    /// - The provided contents
    /// - Marked as incomplete (is_completed = false)
    /// - Not deleted (is_deleted = false)
    pub fn new(contents: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            contents,
            is_completed: false,
            is_deleted: false,
        }
    }

    /// Build a storage key for a ToDo item with the given ID
    ///
    /// Keys are formatted as "{KEY_PREFIX}{uuid}" (e.g., "todo-123e4567-e89b...")
    fn build_key(id: Uuid) -> String {
        format!("{}{}", KEY_PREFIX, id)
    }

    /// Get the storage key for this ToDo item
    fn get_key(&self) -> String {
        Self::build_key(self.id)
    }

    /// Persist this ToDo item to the key-value store
    ///
    /// Saves the current state of the ToDo item to Spin's default key-value store.
    /// This method can be used for both creating new items and updating existing ones.
    pub fn save(&self) -> Result<()> {
        let store = Store::open_default()?;
        store.set_json(self.get_key(), &self)
    }

    /// Retrieve a ToDo item by its ID
    ///
    /// Returns:
    /// - `Ok(Some(todo))` if the item exists
    /// - `Ok(None)` if the item doesn't exist
    /// - `Err` if there's a storage error
    pub fn get_by_id(id: Uuid) -> Result<Option<Self>> {
        let store = Store::open_default()?;
        let key = Self::build_key(id);
        store.get_json::<Self>(key)
    }

    /// Retrieve all ToDo items from storage
    ///
    /// Returns a vector of all ToDo items stored in the key-value store,
    /// including both completed and deleted items. The caller should filter
    /// based on the `is_deleted` flag if only active items are needed.
    ///
    /// This method:
    /// 1. Gets all keys from the store
    /// 2. Filters for keys with the ToDo prefix
    /// 3. Deserializes each item from JSON
    /// 4. Collects successful deserializations into a vector
    pub fn get_all() -> Result<Vec<Self>> {
        let store = Store::open_default()?;

        Ok(store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(KEY_PREFIX))
            .filter_map(|key| store.get_json::<Self>(key).ok())
            .map(|td| td.unwrap())
            .collect())
    }
}
