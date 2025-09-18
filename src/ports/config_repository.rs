//! Configuration repository port for hexagonal architecture
//!
//! This port defines the interface for configuration storage and retrieval,
//! allowing different implementations (TOML files, KV stores, etc.)

use crate::domain::config::{Configuration, ConfigOverride};
use crate::error::ApiError;
use async_trait::async_trait;

/// Repository trait for configuration management
///
/// Implementations can use different storage backends while maintaining
/// the same interface for the application layer.
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    /// Get the base configuration (typically from TOML file)
    async fn get_base_config(&self) -> Result<Configuration, ApiError>;

    /// Get district-level configuration overrides
    async fn get_district_overrides(&self, district_id: &str) -> Result<Option<ConfigOverride>, ApiError>;

    /// Get judge-level configuration overrides
    async fn get_judge_overrides(&self, district_id: &str, judge_id: &str) -> Result<Option<ConfigOverride>, ApiError>;

    /// Save district-level configuration overrides
    async fn save_district_overrides(&self, district_id: &str, overrides: &ConfigOverride) -> Result<(), ApiError>;

    /// Save judge-level configuration overrides
    async fn save_judge_overrides(&self, district_id: &str, judge_id: &str, overrides: &ConfigOverride) -> Result<(), ApiError>;

    /// Delete district-level configuration overrides
    async fn delete_district_overrides(&self, district_id: &str) -> Result<(), ApiError>;

    /// Delete judge-level configuration overrides
    async fn delete_judge_overrides(&self, district_id: &str, judge_id: &str) -> Result<(), ApiError>;

    /// Get merged configuration for a specific context
    /// This is a convenience method that merges base, district, and judge configs
    async fn get_merged_config(&self, district_id: &str, judge_id: Option<&str>) -> Result<Configuration, ApiError> {
        // Get base configuration
        let mut config = self.get_base_config().await?;

        // Apply district overrides if they exist
        if let Some(district_overrides) = self.get_district_overrides(district_id).await? {
            district_overrides.apply_to(&mut config);
        }

        // Apply judge overrides if judge_id is provided and overrides exist
        if let Some(judge_id) = judge_id {
            if let Some(judge_overrides) = self.get_judge_overrides(district_id, judge_id).await? {
                judge_overrides.apply_to(&mut config);
            }
        }

        Ok(config)
    }
}