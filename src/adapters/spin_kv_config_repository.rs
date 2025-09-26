//! Spin KV adapter for configuration repository
//!
//! This adapter implements the ConfigRepository trait using Spin's Key-Value store,
//! providing persistent storage for district and judge configuration overrides.

use crate::adapters::store_utils::open_validated_store;
use crate::adapters::toml_config_loader::TomlConfigLoader;
use crate::domain::config::{Configuration, ConfigOverride};
use crate::error::ApiError;
use crate::ports::config_repository::ConfigRepository;
use async_trait::async_trait;
use spin_sdk::key_value::{Error as KvError, Store};

/// Spin KV implementation of the configuration repository
pub struct SpinKvConfigRepository {
    /// The KV store name (tenant-specific)
    store_name: String,
    /// District code for loading appropriate base config
    district_code: String,
    /// Court type (district, bankruptcy, appellate, etc.)
    court_type: String,
    /// TOML loader for base configuration
    toml_loader: TomlConfigLoader,
}

impl SpinKvConfigRepository {
    /// Create a new repository for a specific district and court type
    pub fn with_district(store_name: String, district_code: String, court_type: String) -> Self {
        Self {
            store_name,
            district_code,
            court_type,
            toml_loader: TomlConfigLoader::new().expect("Failed to load configuration files"),
        }
    }

    /// Create a new repository with a specific store name (defaults to district court)
    pub fn with_store(store_name: String) -> Self {
        Self::with_district(store_name, "GENERIC".to_string(), "district".to_string())
    }

    /// Create a new repository with default store
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_store("default".to_string())
    }

    /// Get the KV store instance
    fn get_store(&self) -> Result<Store, ApiError> {
        open_validated_store(&self.store_name).map_err(|e| match e.downcast_ref::<KvError>() {
            Some(KvError::NoSuchStore) => ApiError::NotFound(format!("Store '{}' not found. Ensure the tenant is configured.", self.store_name)),
            Some(KvError::AccessDenied) => ApiError::Forbidden(format!("Access denied to store '{}'", self.store_name)),
            _ => ApiError::InternalServerError(format!("Failed to open store '{}': {}", self.store_name, e)),
        })
    }

    /// Generate key for district overrides
    fn district_key(&self, district_id: &str) -> String {
        format!("config:district:{}", district_id)
    }

    /// Generate key for judge overrides
    fn judge_key(&self, district_id: &str, judge_id: &str) -> String {
        format!("config:judge:{}:{}", district_id, judge_id)
    }
}

#[async_trait]
impl ConfigRepository for SpinKvConfigRepository {
    async fn get_base_config(&self) -> Result<Configuration, ApiError> {
        // Load the appropriate base config based on court type and district
        self.toml_loader.load_for_district(
            &self.court_type,
            Some(&self.district_code)
        )
    }

    async fn get_district_overrides(&self, district_id: &str) -> Result<Option<ConfigOverride>, ApiError> {
        let store = self.get_store()?;
        let key = self.district_key(district_id);

        match store.get(&key) {
            Ok(Some(data)) => {
                let overrides: ConfigOverride = serde_json::from_slice(&data)
                    .map_err(|e| ApiError::InternalServerError(format!("Failed to deserialize district overrides: {}", e)))?;
                Ok(Some(overrides))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ApiError::InternalServerError(format!("Failed to get district overrides: {}", e))),
        }
    }

    async fn get_judge_overrides(&self, district_id: &str, judge_id: &str) -> Result<Option<ConfigOverride>, ApiError> {
        let store = self.get_store()?;
        let key = self.judge_key(district_id, judge_id);

        match store.get(&key) {
            Ok(Some(data)) => {
                let overrides: ConfigOverride = serde_json::from_slice(&data)
                    .map_err(|e| ApiError::InternalServerError(format!("Failed to deserialize judge overrides: {}", e)))?;
                Ok(Some(overrides))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ApiError::InternalServerError(format!("Failed to get judge overrides: {}", e))),
        }
    }

    async fn save_district_overrides(&self, district_id: &str, overrides: &ConfigOverride) -> Result<(), ApiError> {
        let store = self.get_store()?;
        let key = self.district_key(district_id);

        let json = serde_json::to_vec(overrides)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize district overrides: {}", e)))?;

        store.set(&key, &json)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to save district overrides: {}", e)))?;

        Ok(())
    }

    async fn save_judge_overrides(&self, district_id: &str, judge_id: &str, overrides: &ConfigOverride) -> Result<(), ApiError> {
        let store = self.get_store()?;
        let key = self.judge_key(district_id, judge_id);

        let json = serde_json::to_vec(overrides)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize judge overrides: {}", e)))?;

        store.set(&key, &json)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to save judge overrides: {}", e)))?;

        Ok(())
    }

    async fn delete_district_overrides(&self, district_id: &str) -> Result<(), ApiError> {
        let store = self.get_store()?;
        let key = self.district_key(district_id);

        store.delete(&key)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to delete district overrides: {}", e)))?;

        Ok(())
    }

    async fn delete_judge_overrides(&self, district_id: &str, judge_id: &str) -> Result<(), ApiError> {
        let store = self.get_store()?;
        let key = self.judge_key(district_id, judge_id);

        store.delete(&key)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to delete judge overrides: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let repo = SpinKvConfigRepository::new();

        assert_eq!(repo.district_key("SDNY"), "config:district:SDNY");
        assert_eq!(repo.judge_key("SDNY", "judge-123"), "config:judge:SDNY:judge-123");
    }
}