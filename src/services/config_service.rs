//! Configuration service for managing hierarchical configurations
//!
//! This service handles the business logic for configuration management,
//! including merging configurations, caching, and validation.

use crate::domain::config::{ConfigMetadata, ConfigOverride, ConfigResponse};
use crate::error::ApiError;
use crate::ports::config_repository::ConfigRepository;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Service for managing configurations
pub struct ConfigService {
    repository: Arc<dyn ConfigRepository>,
}

impl ConfigService {
    /// Create a new configuration service
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }

    /// Get merged configuration for a district and optional judge
    pub async fn get_config(
        &self,
        district_id: &str,
        judge_id: Option<&str>,
    ) -> Result<ConfigResponse, ApiError> {
        // Get base configuration
        let _base_config = self.repository.get_base_config().await?;

        // Check for district overrides
        let has_district_overrides = self.repository
            .get_district_overrides(district_id)
            .await?
            .is_some();

        // Check for judge overrides
        let has_judge_overrides = if let Some(judge_id) = judge_id {
            self.repository
                .get_judge_overrides(district_id, judge_id)
                .await?
                .is_some()
        } else {
            false
        };

        // Get the merged configuration
        let config = self.repository.get_merged_config(district_id, judge_id).await?;

        // Create metadata
        let metadata = ConfigMetadata {
            district: district_id.to_string(),
            judge: judge_id.map(|s| s.to_string()),
            base_config: "base.toml".to_string(),
            has_district_overrides,
            has_judge_overrides,
            cached_at: Utc::now(),
        };

        Ok(ConfigResponse { config, metadata })
    }

    /// Get district-level overrides only
    pub async fn get_district_overrides(
        &self,
        district_id: &str,
    ) -> Result<Option<ConfigOverride>, ApiError> {
        self.repository.get_district_overrides(district_id).await
    }

    /// Get judge-level overrides only
    pub async fn get_judge_overrides(
        &self,
        district_id: &str,
        judge_id: &str,
    ) -> Result<Option<ConfigOverride>, ApiError> {
        self.repository.get_judge_overrides(district_id, judge_id).await
    }

    /// Update district-level configuration overrides
    pub async fn update_district_config(
        &self,
        district_id: &str,
        updates: HashMap<String, Value>,
    ) -> Result<(), ApiError> {
        // Get existing overrides or create new
        let mut overrides = self.repository
            .get_district_overrides(district_id)
            .await?
            .unwrap_or_default();

        // Apply updates
        for (path, value) in updates {
            overrides.add(path, value);
        }

        // Save back to repository
        self.repository.save_district_overrides(district_id, &overrides).await
    }

    /// Update judge-level configuration overrides
    pub async fn update_judge_config(
        &self,
        district_id: &str,
        judge_id: &str,
        updates: HashMap<String, Value>,
    ) -> Result<(), ApiError> {
        // Get existing overrides or create new
        let mut overrides = self.repository
            .get_judge_overrides(district_id, judge_id)
            .await?
            .unwrap_or_default();

        // Apply updates
        for (path, value) in updates {
            overrides.add(path, value);
        }

        // Save back to repository
        self.repository.save_judge_overrides(district_id, judge_id, &overrides).await
    }

    /// Clear district-level overrides (revert to base config)
    pub async fn clear_district_overrides(&self, district_id: &str) -> Result<(), ApiError> {
        self.repository.delete_district_overrides(district_id).await
    }

    /// Clear judge-level overrides (revert to district config)
    pub async fn clear_judge_overrides(
        &self,
        district_id: &str,
        judge_id: &str,
    ) -> Result<(), ApiError> {
        self.repository.delete_judge_overrides(district_id, judge_id).await
    }

    /// Preview what configuration would look like with proposed changes
    pub async fn preview_config_changes(
        &self,
        district_id: &str,
        judge_id: Option<&str>,
        proposed_changes: HashMap<String, Value>,
    ) -> Result<ConfigResponse, ApiError> {
        // Get current configuration
        let mut config = self.repository.get_merged_config(district_id, judge_id).await?;

        // Apply proposed changes
        for (path, value) in proposed_changes {
            config.set(&path, value);
        }

        // Create metadata
        let metadata = ConfigMetadata {
            district: district_id.to_string(),
            judge: judge_id.map(|s| s.to_string()),
            base_config: "base.toml".to_string(),
            has_district_overrides: true, // Preview always shows as having overrides
            has_judge_overrides: judge_id.is_some(),
            cached_at: Utc::now(),
        };

        Ok(ConfigResponse { config, metadata })
    }

    /// Validate configuration updates before applying
    pub fn validate_updates(&self, updates: &HashMap<String, Value>) -> Result<(), ApiError> {
        for (path, value) in updates {
            // Validate path format
            if path.is_empty() || path.starts_with('.') || path.ends_with('.') {
                return Err(ApiError::BadRequest(format!("Invalid configuration path: {}", path)));
            }

            // Validate specific known paths and their types
            self.validate_known_path(path, value)?;
        }

        Ok(())
    }

    /// Validate known configuration paths and their expected types
    fn validate_known_path(&self, path: &str, value: &Value) -> Result<(), ApiError> {
        // Define expected types for known paths
        let expected_types = [
            ("deadline", "number"),
            ("days", "number"),
            ("enabled", "boolean"),
            ("max_", "number"),
            ("min_", "number"),
            ("require_", "boolean"),
            ("allow_", "boolean"),
            ("auto_", "boolean"),
            ("_format", "string"),
            ("_pattern", "string"),
        ];

        for (pattern, expected_type) in expected_types {
            if path.contains(pattern) {
                match expected_type {
                    "number" => {
                        if !value.is_number() {
                            return Err(ApiError::BadRequest(
                                format!("Path '{}' expects a number, got {:?}", path, value)
                            ));
                        }
                    }
                    "boolean" => {
                        if !value.is_boolean() {
                            return Err(ApiError::BadRequest(
                                format!("Path '{}' expects a boolean, got {:?}", path, value)
                            ));
                        }
                    }
                    "string" => {
                        if !value.is_string() {
                            return Err(ApiError::BadRequest(
                                format!("Path '{}' expects a string, got {:?}", path, value)
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_updates() {
        let service = ConfigService::new(Arc::new(MockRepository));

        let mut updates = HashMap::new();
        updates.insert("deadlines.default_response_days".to_string(), json!(30));
        updates.insert("workflow.auto_docket_on_filing".to_string(), json!(true));

        assert!(service.validate_updates(&updates).is_ok());

        // Test invalid type
        updates.insert("deadlines.default_response_days".to_string(), json!("not a number"));
        assert!(service.validate_updates(&updates).is_err());
    }

    // Mock repository for testing
    struct MockRepository;

    #[async_trait::async_trait]
    impl ConfigRepository for MockRepository {
        async fn get_base_config(&self) -> Result<Configuration, ApiError> {
            Ok(Configuration::new())
        }

        async fn get_district_overrides(&self, _: &str) -> Result<Option<ConfigOverride>, ApiError> {
            Ok(None)
        }

        async fn get_judge_overrides(&self, _: &str, _: &str) -> Result<Option<ConfigOverride>, ApiError> {
            Ok(None)
        }

        async fn save_district_overrides(&self, _: &str, _: &ConfigOverride) -> Result<(), ApiError> {
            Ok(())
        }

        async fn save_judge_overrides(&self, _: &str, _: &str, _: &ConfigOverride) -> Result<(), ApiError> {
            Ok(())
        }

        async fn delete_district_overrides(&self, _: &str) -> Result<(), ApiError> {
            Ok(())
        }

        async fn delete_judge_overrides(&self, _: &str, _: &str) -> Result<(), ApiError> {
            Ok(())
        }
    }
}