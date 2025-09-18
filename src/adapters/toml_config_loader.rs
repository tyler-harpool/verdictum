//! Distributed TOML configuration file loader
//!
//! This adapter loads configuration from multiple TOML files in a hierarchy:
//! 1. System-wide defaults (base_defaults.toml)
//! 2. Court-type specific config (e.g., courts/district.toml)
//! 3. District-specific config (e.g., districts/SDNY.toml)
//!
//! Each level can override values from the previous level.

use crate::domain::config::Configuration;
use crate::error::ApiError;
use serde_json::Value;
use std::collections::HashMap;

/// Loads configuration from distributed TOML files
pub struct TomlConfigLoader {
    /// Cached configurations by file path
    configs: HashMap<String, String>,
}

impl TomlConfigLoader {
    /// Create a new TOML loader
    pub fn new() -> Result<Self, ApiError> {
        let mut configs = HashMap::new();

        // Load available config files
        // In production, these would be loaded dynamically from filesystem
        // For now, we include them in the binary
        configs.insert("base_defaults".to_string(),
            include_str!("../../config/base_defaults.toml").to_string());
        configs.insert("courts/district".to_string(),
            include_str!("../../config/courts/district.toml").to_string());
        configs.insert("courts/bankruptcy".to_string(),
            include_str!("../../config/courts/bankruptcy.toml").to_string());
        configs.insert("courts/appellate".to_string(),
            include_str!("../../config/courts/appellate.toml").to_string());
        configs.insert("districts/SDNY".to_string(),
            include_str!("../../config/districts/SDNY.toml").to_string());

        Ok(Self { configs })
    }

    /// Create a new TOML loader with custom content (for testing)
    #[allow(dead_code)]
    pub fn with_content(content: String) -> Self {
        let mut configs = HashMap::new();
        configs.insert("test".to_string(), content);
        Self { configs }
    }

    /// Load configuration for a specific court type and district
    pub fn load_for_district(&self, court_type: &str, district: Option<&str>) -> Result<Configuration, ApiError> {
        let mut config = Configuration::new();

        // 1. Load base defaults
        if let Some(base_content) = self.configs.get("base_defaults") {
            let base_config = self.parse_toml(base_content)?;
            config.merge(&base_config);
        }

        // 2. Load court-type specific config
        let court_path = format!("courts/{}", court_type);
        if let Some(court_content) = self.configs.get(&court_path) {
            let court_config = self.parse_toml(court_content)?;
            config.merge(&court_config);
        }

        // 3. Load district-specific config if provided
        if let Some(district_code) = district {
            let district_path = format!("districts/{}", district_code);
            if let Some(district_content) = self.configs.get(&district_path) {
                let district_config = self.parse_toml(district_content)?;
                config.merge(&district_config);
            }
        }

        Ok(config)
    }

    /// Load base configuration for a court type (no district overrides)
    pub fn load_court_base(&self, court_type: &str) -> Result<Configuration, ApiError> {
        self.load_for_district(court_type, None)
    }

    /// Parse a TOML string into a Configuration
    fn parse_toml(&self, content: &str) -> Result<Configuration, ApiError> {
        let toml_value: toml::Value = content.parse()
            .map_err(|e: toml::de::Error| ApiError::InternalServerError(format!("Failed to parse TOML: {}", e)))?;

        let json_value = self.toml_to_json(toml_value)?;
        Ok(Configuration::from_value(json_value))
    }

    /// Get list of available court types
    pub fn get_available_court_types(&self) -> Vec<String> {
        self.configs.keys()
            .filter(|k| k.starts_with("courts/"))
            .map(|k| k.strip_prefix("courts/").unwrap().to_string())
            .collect()
    }

    /// Get list of configured districts
    pub fn get_configured_districts(&self) -> Vec<String> {
        self.configs.keys()
            .filter(|k| k.starts_with("districts/"))
            .map(|k| k.strip_prefix("districts/").unwrap().to_string())
            .collect()
    }

    /// Convert TOML value to JSON value
    fn toml_to_json(&self, toml_value: toml::Value) -> Result<Value, ApiError> {
        match toml_value {
            toml::Value::String(s) => Ok(Value::String(s)),
            toml::Value::Integer(i) => Ok(Value::Number(i.into())),
            toml::Value::Float(f) => {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| ApiError::InternalServerError("Invalid float value".to_string()))
            }
            toml::Value::Boolean(b) => Ok(Value::Bool(b)),
            toml::Value::Array(arr) => {
                let json_array: Result<Vec<Value>, ApiError> =
                    arr.into_iter().map(|v| self.toml_to_json(v)).collect();
                Ok(Value::Array(json_array?))
            }
            toml::Value::Table(table) => {
                let mut json_map = serde_json::Map::new();
                for (key, value) in table {
                    json_map.insert(key, self.toml_to_json(value)?);
                }
                Ok(Value::Object(json_map))
            }
            toml::Value::Datetime(dt) => Ok(Value::String(dt.to_string())),
        }
    }
}

impl Default for TomlConfigLoader {
    fn default() -> Self {
        Self::new().expect("Failed to load configuration files")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_toml_config() {
        let toml_content = r#"
[case_assignment]
auto_generate_pdf = true
max_days_to_file = 30

[deadlines]
default_response_days = 21
appeal_window_days = 30
"#;

        let loader = TomlConfigLoader::with_content(toml_content.to_string());
        let config = loader.parse_toml(toml_content).unwrap();

        assert_eq!(config.get_bool("case_assignment.auto_generate_pdf"), Some(true));
        assert_eq!(config.get_i64("case_assignment.max_days_to_file"), Some(30));
        assert_eq!(config.get_i64("deadlines.default_response_days"), Some(21));
        assert_eq!(config.get_i64("deadlines.appeal_window_days"), Some(30));
    }

    #[test]
    fn test_toml_arrays() {
        let toml_content = r#"
[document_rules]
allowed_formats = ["pdf", "docx", "txt"]
"#;

        let loader = TomlConfigLoader::with_content(toml_content.to_string());
        let config = loader.parse_toml(toml_content).unwrap();

        let formats = config.get_array("document_rules.allowed_formats").unwrap();
        assert_eq!(formats.len(), 3);
        assert_eq!(formats[0].as_str(), Some("pdf"));
        assert_eq!(formats[1].as_str(), Some("docx"));
        assert_eq!(formats[2].as_str(), Some("txt"));
    }
}