//! Feature repository port for managing feature flags
//!
//! This port provides a unified interface for feature management
//! that bridges to the configuration system for hierarchical control.

use crate::error::ApiError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Feature set for a specific court/district
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Court type (district, bankruptcy, fisa, claims, trade)
    pub court_type: String,
    /// District identifier
    pub district: String,
    /// Core features available to all
    pub core: HashMap<String, bool>,
    /// Court-type specific features
    pub court_specific: HashMap<String, bool>,
    /// Advanced features
    pub advanced: HashMap<String, bool>,
    /// Experimental features
    pub experimental: HashMap<String, bool>,
    /// Integration features
    pub integrations: HashMap<String, bool>,
}

/// Rollout status for a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutStatus {
    /// Feature name
    pub feature: String,
    /// Districts where it's enabled
    pub enabled_districts: Vec<String>,
    /// Percentage rollout per district
    pub rollout_percentages: HashMap<String, u8>,
    /// Start date of rollout
    pub started_at: Option<DateTime<Utc>>,
    /// Target completion date
    pub target_completion: Option<DateTime<Utc>>,
    /// Current phase (pilot, beta, general_availability)
    pub phase: RolloutPhase,
}

/// Rollout phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RolloutPhase {
    Planning,
    Pilot,
    Beta,
    GeneralAvailability,
    Deprecated,
}

/// Feature usage analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsage {
    /// Feature name
    pub feature: String,
    /// District
    pub district: String,
    /// Number of times used
    pub usage_count: u64,
    /// Unique users who used it
    pub unique_users: u32,
    /// Last used timestamp
    pub last_used: DateTime<Utc>,
    /// User satisfaction score (1-5)
    pub satisfaction_score: Option<f32>,
}

/// Feature audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAuditEntry {
    /// Timestamp of the change
    pub timestamp: DateTime<Utc>,
    /// District affected
    pub district: String,
    /// Judge ID if judge-specific
    pub judge_id: Option<String>,
    /// Feature that was changed
    pub feature: String,
    /// Previous value
    pub old_value: bool,
    /// New value
    pub new_value: bool,
    /// Who made the change
    pub changed_by: String,
    /// Reason for change
    pub reason: Option<String>,
}

/// Repository trait for feature management
#[async_trait]
pub trait FeatureRepository: Send + Sync {
    /// Get the feature set for a district
    async fn get_features(&self, district: &str, judge_id: Option<&str>) -> Result<FeatureSet, ApiError>;

    /// Check if a specific feature is enabled
    async fn is_feature_enabled(
        &self,
        district: &str,
        judge_id: Option<&str>,
        feature: &str
    ) -> Result<bool, ApiError>;

    /// Enable or disable a feature for a district
    async fn set_feature(
        &self,
        district: &str,
        feature: &str,
        enabled: bool,
        changed_by: &str,
        reason: Option<&str>,
    ) -> Result<(), ApiError>;

    /// Enable or disable a feature for a specific judge
    async fn set_judge_feature(
        &self,
        district: &str,
        judge_id: &str,
        feature: &str,
        enabled: bool,
        changed_by: &str,
        reason: Option<&str>,
    ) -> Result<(), ApiError>;

    /// Get rollout status for a feature
    async fn get_rollout_status(&self, feature: &str) -> Result<RolloutStatus, ApiError>;

    /// Set rollout percentage for a district
    async fn set_rollout_percentage(
        &self,
        district: &str,
        feature: &str,
        percentage: u8,
    ) -> Result<(), ApiError>;

    /// Get feature usage statistics
    async fn get_feature_usage(
        &self,
        district: &str,
        feature: Option<&str>,
    ) -> Result<Vec<FeatureUsage>, ApiError>;

    /// Record feature usage
    async fn record_usage(
        &self,
        district: &str,
        feature: &str,
        user_id: &str,
    ) -> Result<(), ApiError>;

    /// Get audit log for feature changes
    async fn get_audit_log(
        &self,
        district: Option<&str>,
        feature: Option<&str>,
        limit: usize,
    ) -> Result<Vec<FeatureAuditEntry>, ApiError>;

    /// Get districts using a specific feature
    async fn get_districts_with_feature(&self, feature: &str) -> Result<Vec<String>, ApiError>;

    /// Enable a feature for multiple districts (rollout)
    async fn rollout_feature(
        &self,
        feature: &str,
        districts: Vec<String>,
        phase: RolloutPhase,
    ) -> Result<(), ApiError>;

    /// Rollback a feature for all districts
    async fn rollback_feature(&self, feature: &str, reason: &str) -> Result<(), ApiError>;

    /// Get features by court type
    async fn get_court_type_features(&self, court_type: &str) -> Result<HashMap<String, bool>, ApiError>;
}