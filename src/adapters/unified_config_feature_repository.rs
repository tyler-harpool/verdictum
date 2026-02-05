//! Unified adapter that bridges feature management to the configuration system
//!
//! This adapter implements the FeatureRepository trait by delegating to the
//! ConfigRepository, providing a unified system for both settings and features.

use crate::error::ApiError;
use crate::ports::config_repository::ConfigRepository;
use crate::ports::feature_repository::{
    FeatureAuditEntry, FeatureRepository, FeatureSet, FeatureUsage,
    RolloutPhase, RolloutStatus,
};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use spin_sdk::key_value::Store;
use std::collections::HashMap;
use std::sync::Arc;

/// Adapter that unifies features with the configuration system
pub struct UnifiedConfigFeatureRepository {
    /// The underlying configuration repository
    config_repo: Arc<dyn ConfigRepository>,
    /// Store name for audit and analytics
    store_name: String,
}

impl UnifiedConfigFeatureRepository {
    /// Create a new unified repository
    pub fn new(config_repo: Arc<dyn ConfigRepository>, store_name: String) -> Self {
        Self {
            config_repo,
            store_name,
        }
    }

    /// Get the KV store for audit and analytics
    fn get_store(&self) -> Result<Store, ApiError> {
        Store::open(&self.store_name).map_err(|e| {
            ApiError::StorageError(format!("Failed to open store '{}': {}", self.store_name, e))
        })
    }

    /// Convert configuration to feature set
    fn config_to_feature_set(&self, config: &crate::domain::config::Configuration, district: &str) -> FeatureSet {
        let court_type = config.get_court_type();

        // Extract different feature categories
        let mut core = HashMap::new();
        let mut court_specific = HashMap::new();
        let mut advanced = HashMap::new();
        let mut experimental = HashMap::new();
        let mut integrations = HashMap::new();

        // Core features
        if let Some(Value::Object(map)) = config.get("features.core") {
            for (key, value) in map {
                core.insert(key.clone(), value.as_bool().unwrap_or(false));
            }
        }

        // Court-specific features
        let court_path = format!("features.{}", court_type);
        if let Some(Value::Object(map)) = config.get(&court_path) {
            for (key, value) in map {
                court_specific.insert(key.clone(), value.as_bool().unwrap_or(false));
            }
        }

        // Advanced features
        if let Some(Value::Object(map)) = config.get("features.advanced") {
            for (key, value) in map {
                advanced.insert(key.clone(), value.as_bool().unwrap_or(false));
            }
        }

        // Experimental features
        if let Some(Value::Object(map)) = config.get("features.experimental") {
            for (key, value) in map {
                experimental.insert(key.clone(), value.as_bool().unwrap_or(false));
            }
        }

        // Integration features
        if let Some(Value::Object(map)) = config.get("features.integrations") {
            for (key, value) in map {
                integrations.insert(key.clone(), value.as_bool().unwrap_or(false));
            }
        }

        FeatureSet {
            court_type,
            district: district.to_string(),
            core,
            court_specific,
            advanced,
            experimental,
            integrations,
        }
    }

    /// Store an audit entry
    async fn store_audit_entry(&self, entry: FeatureAuditEntry) -> Result<(), ApiError> {
        let store = self.get_store()?;
        let key = format!("feature_audit:{}:{}", entry.district, entry.timestamp.timestamp_millis());

        let json = serde_json::to_vec(&entry)
            .map_err(|e| ApiError::SerializationError(format!("Failed to serialize audit entry: {}", e)))?;

        store.set(&key, &json)
            .map_err(|e| ApiError::StorageError(format!("Failed to store audit entry: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl FeatureRepository for UnifiedConfigFeatureRepository {
    async fn get_features(&self, district: &str, judge_id: Option<&str>) -> Result<FeatureSet, ApiError> {
        // Get merged configuration
        let config = self.config_repo.get_merged_config(district, judge_id).await?;

        // Convert to feature set
        Ok(self.config_to_feature_set(&config, district))
    }

    async fn is_feature_enabled(
        &self,
        district: &str,
        judge_id: Option<&str>,
        feature: &str,
    ) -> Result<bool, ApiError> {
        let config = self.config_repo.get_merged_config(district, judge_id).await?;
        Ok(config.is_feature_enabled(feature))
    }

    async fn set_feature(
        &self,
        district: &str,
        feature: &str,
        enabled: bool,
        changed_by: &str,
        reason: Option<&str>,
    ) -> Result<(), ApiError> {
        // Get existing district overrides
        let mut overrides = self.config_repo
            .get_district_overrides(district)
            .await?
            .unwrap_or_default();

        // Update the feature
        let feature_path = if feature.starts_with("features.") {
            feature.to_string()
        } else {
            format!("features.{}", feature)
        };

        overrides.add(feature_path.clone(), json!(enabled));

        // Save the overrides
        self.config_repo.save_district_overrides(district, &overrides).await?;

        // Store audit entry
        let audit = FeatureAuditEntry {
            timestamp: Utc::now(),
            district: district.to_string(),
            judge_id: None,
            feature: feature.to_string(),
            old_value: !enabled, // Simplified - should fetch actual old value
            new_value: enabled,
            changed_by: changed_by.to_string(),
            reason: reason.map(|s| s.to_string()),
        };

        self.store_audit_entry(audit).await?;

        Ok(())
    }

    async fn set_judge_feature(
        &self,
        district: &str,
        judge_id: &str,
        feature: &str,
        enabled: bool,
        changed_by: &str,
        reason: Option<&str>,
    ) -> Result<(), ApiError> {
        // Get existing judge overrides
        let mut overrides = self.config_repo
            .get_judge_overrides(district, judge_id)
            .await?
            .unwrap_or_default();

        // Update the feature
        let feature_path = if feature.starts_with("features.") {
            feature.to_string()
        } else {
            format!("features.{}", feature)
        };

        overrides.add(feature_path.clone(), json!(enabled));

        // Save the overrides
        self.config_repo.save_judge_overrides(district, judge_id, &overrides).await?;

        // Store audit entry
        let audit = FeatureAuditEntry {
            timestamp: Utc::now(),
            district: district.to_string(),
            judge_id: Some(judge_id.to_string()),
            feature: feature.to_string(),
            old_value: !enabled,
            new_value: enabled,
            changed_by: changed_by.to_string(),
            reason: reason.map(|s| s.to_string()),
        };

        self.store_audit_entry(audit).await?;

        Ok(())
    }

    async fn get_rollout_status(&self, feature: &str) -> Result<RolloutStatus, ApiError> {
        let store = self.get_store()?;
        let key = format!("rollout:{}", feature);

        match store.get(&key) {
            Ok(Some(data)) => {
                let status: RolloutStatus = serde_json::from_slice(&data)
                    .map_err(|e| ApiError::SerializationError(format!("Failed to deserialize rollout status: {}", e)))?;
                Ok(status)
            }
            Ok(None) => {
                // Return default status if not found
                Ok(RolloutStatus {
                    feature: feature.to_string(),
                    enabled_districts: Vec::new(),
                    rollout_percentages: HashMap::new(),
                    started_at: None,
                    target_completion: None,
                    phase: RolloutPhase::Planning,
                })
            }
            Err(e) => Err(ApiError::StorageError(format!("Failed to get rollout status: {}", e))),
        }
    }

    async fn set_rollout_percentage(
        &self,
        district: &str,
        feature: &str,
        percentage: u8,
    ) -> Result<(), ApiError> {
        let store = self.get_store()?;
        let key = format!("rollout:{}:percentage:{}", feature, district);

        store.set(&key, &[percentage])
            .map_err(|e| ApiError::StorageError(format!("Failed to set rollout percentage: {}", e)))?;

        Ok(())
    }

    async fn get_feature_usage(
        &self,
        district: &str,
        feature: Option<&str>,
    ) -> Result<Vec<FeatureUsage>, ApiError> {
        let store = self.get_store()?;
        let prefix = match feature {
            Some(f) => format!("usage:{}:{}", district, f),
            None => format!("usage:{}", district),
        };

        // This is a simplified implementation
        // In production, you'd want to scan keys with the prefix
        let key = format!("{}:summary", prefix);

        match store.get(&key) {
            Ok(Some(data)) => {
                let usage: Vec<FeatureUsage> = serde_json::from_slice(&data)
                    .map_err(|e| ApiError::SerializationError(format!("Failed to deserialize usage: {}", e)))?;
                Ok(usage)
            }
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err(ApiError::StorageError(format!("Failed to get usage: {}", e))),
        }
    }

    async fn record_usage(
        &self,
        district: &str,
        feature: &str,
        user_id: &str,
    ) -> Result<(), ApiError> {
        let store = self.get_store()?;
        let key = format!("usage:{}:{}:{}", district, feature, Utc::now().timestamp_millis());

        let usage = json!({
            "district": district,
            "feature": feature,
            "user_id": user_id,
            "timestamp": Utc::now()
        });

        let data = serde_json::to_vec(&usage)
            .map_err(|e| ApiError::SerializationError(format!("Failed to serialize usage: {}", e)))?;

        store.set(&key, &data)
            .map_err(|e| ApiError::StorageError(format!("Failed to record usage: {}", e)))?;

        Ok(())
    }

    async fn get_audit_log(
        &self,
        district: Option<&str>,
        feature: Option<&str>,
        _limit: usize,
    ) -> Result<Vec<FeatureAuditEntry>, ApiError> {
        // Simplified implementation
        // In production, you'd scan keys and filter
        let _store = self.get_store()?;
        let _prefix = match (district, feature) {
            (Some(d), Some(f)) => format!("feature_audit:{}:{}", d, f),
            (Some(d), None) => format!("feature_audit:{}", d),
            (None, _) => "feature_audit:".to_string(),
        };

        // This would need proper key scanning in production
        Ok(Vec::new())
    }

    async fn get_districts_with_feature(&self, _feature: &str) -> Result<Vec<String>, ApiError> {
        // This would require scanning all districts
        // For now, return empty list
        Ok(Vec::new())
    }

    async fn rollout_feature(
        &self,
        feature: &str,
        districts: Vec<String>,
        phase: RolloutPhase,
    ) -> Result<(), ApiError> {
        for district in &districts {
            self.set_feature(district, feature, true, "system", Some("Feature rollout")).await?;
        }

        // Update rollout status
        let store = self.get_store()?;
        let key = format!("rollout:{}", feature);

        let status = RolloutStatus {
            feature: feature.to_string(),
            enabled_districts: districts,
            rollout_percentages: HashMap::new(),
            started_at: Some(Utc::now()),
            target_completion: None,
            phase,
        };

        let data = serde_json::to_vec(&status)
            .map_err(|e| ApiError::SerializationError(format!("Failed to serialize rollout: {}", e)))?;

        store.set(&key, &data)
            .map_err(|e| ApiError::StorageError(format!("Failed to save rollout: {}", e)))?;

        Ok(())
    }

    async fn rollback_feature(&self, feature: &str, reason: &str) -> Result<(), ApiError> {
        // Get all districts with this feature
        let districts = self.get_districts_with_feature(feature).await?;

        for district in districts {
            self.set_feature(&district, feature, false, "system", Some(reason)).await?;
        }

        Ok(())
    }

    async fn get_court_type_features(&self, court_type: &str) -> Result<HashMap<String, bool>, ApiError> {
        // Get base configuration to see available features for court type
        let config = self.config_repo.get_base_config().await?;

        let mut features = HashMap::new();
        let path = format!("features.{}", court_type);

        if let Some(Value::Object(map)) = config.get(&path) {
            for (key, value) in map {
                features.insert(key.clone(), value.as_bool().unwrap_or(false));
            }
        }

        Ok(features)
    }
}