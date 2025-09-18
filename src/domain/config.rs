//! Domain model for hierarchical configuration system
//!
//! Supports base configuration from TOML with district and judge-level overrides
//! stored in KV stores. Configuration values are flexible using serde_json::Value.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use utoipa::ToSchema;

/// Configuration structure that matches our TOML format
/// but uses serde_json::Value for flexibility
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Configuration {
    /// The actual configuration values as a nested JSON structure
    #[serde(flatten)]
    pub values: HashMap<String, Value>,
}

impl Configuration {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Get the court type from configuration
    pub fn get_court_type(&self) -> String {
        self.get_string("features.court_type")
            .unwrap_or_else(|| "district".to_string())
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature_path: &str) -> bool {
        // Handle the special case of court-type specific features
        let full_path = if !feature_path.starts_with("features.") {
            format!("features.{}", feature_path)
        } else {
            feature_path.to_string()
        };

        self.get_bool(&full_path).unwrap_or(false)
    }

    /// Get all enabled features for the current court type
    pub fn get_enabled_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        let court_type = self.get_court_type();

        // Check core features
        if let Some(Value::Object(core)) = self.get("features.core") {
            for (key, value) in core {
                if value.as_bool().unwrap_or(false) {
                    features.push(format!("core.{}", key));
                }
            }
        }

        // Check court-type specific features
        let court_section = format!("features.{}", court_type);
        if let Some(Value::Object(court_features)) = self.get(&court_section) {
            for (key, value) in court_features {
                if value.as_bool().unwrap_or(false) {
                    features.push(format!("{}.{}", court_type, key));
                }
            }
        }

        // Check advanced features
        if let Some(Value::Object(advanced)) = self.get("features.advanced") {
            for (key, value) in advanced {
                if value.as_bool().unwrap_or(false) {
                    features.push(format!("advanced.{}", key));
                }
            }
        }

        features
    }

    /// Check if this is a bankruptcy court
    pub fn is_bankruptcy_court(&self) -> bool {
        self.get_court_type() == "bankruptcy" ||
        self.is_feature_enabled("features.bankruptcy.enabled")
    }

    /// Check if this is a trade court
    pub fn is_trade_court(&self) -> bool {
        self.get_court_type() == "trade" ||
        self.is_feature_enabled("features.trade.enabled")
    }

    /// Check if this is a FISA court
    pub fn is_fisa_court(&self) -> bool {
        self.get_court_type() == "fisa" ||
        self.is_feature_enabled("features.fisa.enabled")
    }

    /// Check if this is a tax court
    pub fn is_tax_court(&self) -> bool {
        self.get_court_type() == "tax" ||
        self.is_feature_enabled("features.tax.enabled")
    }

    /// Check if this is a claims court
    pub fn is_claims_court(&self) -> bool {
        self.get_court_type() == "claims" ||
        self.is_feature_enabled("features.claims.enabled")
    }

    /// Check if this is PTAB (Patent Trial and Appeal Board)
    pub fn is_patent_board(&self) -> bool {
        self.get_court_type() == "patent" ||
        self.is_feature_enabled("features.patent.enabled")
    }

    /// Check if this is TTAB (Trademark Trial and Appeal Board)
    pub fn is_trademark_board(&self) -> bool {
        self.get_court_type() == "trademark" ||
        self.is_feature_enabled("features.trademark.enabled")
    }

    /// Check if this is ITC (International Trade Commission)
    pub fn is_itc(&self) -> bool {
        self.get_court_type() == "itc" ||
        self.is_feature_enabled("features.itc.enabled")
    }

    /// Check if this is MSPB (Merit Systems Protection Board)
    pub fn is_merit_board(&self) -> bool {
        self.get_court_type() == "merit" ||
        self.is_feature_enabled("features.merit.enabled")
    }

    /// Check if this is the Alien Terrorist Removal Court
    pub fn is_alien_terrorist_court(&self) -> bool {
        self.get_court_type() == "alien_terrorist" ||
        self.is_feature_enabled("features.alien_terrorist.enabled")
    }

    /// Get features available for the current court type
    pub fn get_court_features(&self) -> HashMap<String, bool> {
        let mut features = HashMap::new();
        let court_type = self.get_court_type();

        let section_path = format!("features.{}", court_type);
        if let Some(Value::Object(map)) = self.get(&section_path) {
            for (key, value) in map {
                features.insert(key.clone(), value.as_bool().unwrap_or(false));
            }
        }

        features
    }

    /// Create configuration from JSON value
    pub fn from_value(value: Value) -> Self {
        match value {
            Value::Object(map) => Self {
                values: map.into_iter().collect(),
            },
            _ => Self::new(),
        }
    }

    /// Get a value by dot-notation path (e.g., "case_assignment.auto_generate_pdf")
    pub fn get(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        let mut current = self.values.get(parts[0])?;

        for part in &parts[1..] {
            match current {
                Value::Object(map) => {
                    current = map.get(*part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Set a value by dot-notation path
    pub fn set(&mut self, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            // Direct assignment to root
            self.values.insert(parts[0].to_string(), value);
            return;
        }

        // For nested paths, we need to ensure the structure exists
        let first = parts[0];
        let rest_path = parts[1..].join(".");

        // Get or create the first level
        let entry = self.values.entry(first.to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        // Recursively set the value
        Self::set_nested_value(entry, &rest_path, value);
    }

    /// Helper function to set nested values
    fn set_nested_value(current: &mut Value, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            // Last part - set the value
            if let Value::Object(map) = current {
                map.insert(parts[0].to_string(), value);
            } else {
                // Current value is not an object, replace it
                *current = Value::Object(serde_json::Map::new());
                if let Value::Object(map) = current {
                    map.insert(parts[0].to_string(), value);
                }
            }
            return;
        }

        // Not the last part - ensure we have an object and recurse
        if !current.is_object() {
            *current = Value::Object(serde_json::Map::new());
        }

        if let Value::Object(map) = current {
            let first = parts[0];
            let rest_path = parts[1..].join(".");

            let entry = map.entry(first.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));

            Self::set_nested_value(entry, &rest_path, value);
        }
    }

    /// Merge another configuration into this one (other takes precedence)
    pub fn merge(&mut self, other: &Configuration) {
        for (key, value) in &other.values {
            match (&self.values.get(key), value) {
                (Some(Value::Object(_)), Value::Object(_)) => {
                    // Both are objects - deep merge
                    if let Some(existing) = self.values.get_mut(key) {
                        Self::deep_merge_values(existing, value);
                    }
                }
                _ => {
                    // Simple override
                    self.values.insert(key.clone(), value.clone());
                }
            }
        }
    }

    /// Deep merge two JSON values
    fn deep_merge_values(target: &mut Value, source: &Value) {
        match (&mut *target, source) {
            (Value::Object(ref mut target_map), Value::Object(source_map)) => {
                for (key, value) in source_map {
                    match target_map.get_mut(key) {
                        Some(existing) => Self::deep_merge_values(existing, value),
                        None => {
                            target_map.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            (target_val, source_val) => {
                // For non-objects, source overwrites target
                *target_val = source_val.clone();
            }
        }
    }

    /// Get a boolean value from the configuration
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path)?.as_bool()
    }

    /// Get a string value from the configuration
    pub fn get_string(&self, path: &str) -> Option<String> {
        self.get(path)?.as_str().map(|s| s.to_string())
    }

    /// Get an integer value from the configuration
    pub fn get_i64(&self, path: &str) -> Option<i64> {
        self.get(path)?.as_i64()
    }

    /// Get a float value from the configuration
    pub fn get_f64(&self, path: &str) -> Option<f64> {
        self.get(path)?.as_f64()
    }

    /// Get an array value from the configuration
    pub fn get_array(&self, path: &str) -> Option<&Vec<Value>> {
        self.get(path)?.as_array()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration override structure for storing partial configs in KV
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigOverride {
    /// Flat map of dot-notation paths to values
    /// e.g., "case_assignment.auto_generate_pdf" => true
    pub overrides: HashMap<String, Value>,
}

impl ConfigOverride {
    /// Create a new override set
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
        }
    }

    /// Add an override
    pub fn add(&mut self, path: String, value: Value) {
        self.overrides.insert(path, value);
    }

    /// Apply these overrides to a configuration
    pub fn apply_to(&self, config: &mut Configuration) {
        for (path, value) in &self.overrides {
            config.set(path, value.clone());
        }
    }
}

impl Default for ConfigOverride {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a configuration (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigMetadata {
    /// District this config is for
    pub district: String,
    /// Judge ID if judge-specific
    pub judge: Option<String>,
    /// Base configuration file used
    pub base_config: String,
    /// Whether district overrides are applied
    pub has_district_overrides: bool,
    /// Whether judge overrides are applied
    pub has_judge_overrides: bool,
    /// When this config was cached/generated
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// Complete configuration response (config + metadata)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigResponse {
    /// The merged configuration values
    #[serde(flatten)]
    pub config: Configuration,
    /// Metadata about this configuration
    #[serde(rename = "_metadata")]
    pub metadata: ConfigMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_set_values() {
        let mut config = Configuration::new();

        // Set nested value
        config.set("case_assignment.auto_generate_pdf", json!(true));

        // Get the value
        assert_eq!(config.get_bool("case_assignment.auto_generate_pdf"), Some(true));
    }

    #[test]
    fn test_merge_configurations() {
        let mut base = Configuration::new();
        base.set("deadlines.default_response_days", json!(21));
        base.set("deadlines.appeal_window_days", json!(30));

        let mut override_config = Configuration::new();
        override_config.set("deadlines.default_response_days", json!(30));

        base.merge(&override_config);

        assert_eq!(base.get_i64("deadlines.default_response_days"), Some(30));
        assert_eq!(base.get_i64("deadlines.appeal_window_days"), Some(30));
    }

    #[test]
    fn test_overrides() {
        let mut config = Configuration::new();
        config.set("workflow.auto_docket_on_filing", json!(false));

        let mut overrides = ConfigOverride::new();
        overrides.add("workflow.auto_docket_on_filing".to_string(), json!(true));

        overrides.apply_to(&mut config);

        assert_eq!(config.get_bool("workflow.auto_docket_on_filing"), Some(true));
    }
}