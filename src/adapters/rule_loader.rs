//! TOML-based rule configuration loader
//!
//! Loads court rules from TOML configuration strings, allowing rules
//! to be defined in configuration files and loaded into the system
//! without manual construction.

use crate::domain::rule::{
    Rule, RuleAction, RuleCategory, RuleCondition, RulePriority, RuleSource, RuleStatus,
    TriggerEvent,
};
use crate::error::ApiError;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

/// Intermediate TOML representation wrapping a list of rules
#[derive(Deserialize)]
struct TomlRulesConfig {
    #[serde(default)]
    rules: Vec<TomlRule>,
}

/// Intermediate TOML representation of a single rule
#[derive(Deserialize)]
struct TomlRule {
    name: String,
    description: String,
    source: RuleSource,
    category: RuleCategory,
    #[serde(default)]
    triggers: Vec<TriggerEvent>,
    #[serde(default)]
    conditions: Vec<RuleCondition>,
    #[serde(default)]
    actions: Vec<RuleAction>,
    #[serde(default = "default_priority")]
    priority: RulePriority,
    #[serde(default = "default_status")]
    status: RuleStatus,
    jurisdiction: Option<String>,
    citation: Option<String>,
    effective_date: Option<DateTime<Utc>>,
    expiration_date: Option<DateTime<Utc>>,
    supersedes_rule_id: Option<Uuid>,
    created_by: Option<String>,
}

fn default_priority() -> RulePriority {
    RulePriority::FederalRule
}

fn default_status() -> RuleStatus {
    RuleStatus::Draft
}

/// Load rules from a TOML configuration string
///
/// Expects TOML in the format:
/// ```toml
/// [[rules]]
/// name = "Privacy Redaction Rule"
/// description = "Require redaction of SSN in civil filings"
/// source = "frcp"
/// category = "privacy"
/// triggers = ["document_filed"]
/// priority = "federal_rule"
/// status = "active"
///
/// [[rules.conditions]]
/// type = "field_equals"
/// field = "case_type"
/// value = "civil"
///
/// [[rules.actions]]
/// type = "require_redaction"
/// fields = ["ssn", "date_of_birth"]
/// ```
pub fn load_rules_from_toml(toml_content: &str) -> Result<Vec<Rule>, ApiError> {
    let config: TomlRulesConfig = toml::from_str(toml_content)
        .map_err(|e| ApiError::ValidationError(format!("Failed to parse rules TOML: {}", e)))?;

    let now = Utc::now();

    let rules = config
        .rules
        .into_iter()
        .map(|toml_rule| {
            Rule {
                id: Uuid::new_v4(),
                name: toml_rule.name,
                description: toml_rule.description,
                source: toml_rule.source,
                category: toml_rule.category,
                triggers: toml_rule.triggers,
                conditions: toml_rule.conditions,
                actions: toml_rule.actions,
                priority: toml_rule.priority,
                status: toml_rule.status,
                jurisdiction: toml_rule.jurisdiction,
                citation: toml_rule.citation,
                effective_date: toml_rule.effective_date,
                expiration_date: toml_rule.expiration_date,
                supersedes_rule_id: toml_rule.supersedes_rule_id,
                created_at: now,
                updated_at: now,
                created_by: toml_rule.created_by,
            }
        })
        .collect();

    Ok(rules)
}
