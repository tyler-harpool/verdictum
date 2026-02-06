//! Repository port for rule persistence
//!
//! This trait defines the contract for storing and retrieving rules
//! in the federal court system's rules engine.

use crate::domain::rule::{Rule, RuleCategory, RuleStatus, RuleSource, TriggerEvent};
use anyhow::Result;
use uuid::Uuid;

/// Repository trait for rule persistence
pub trait RulesRepository {
    /// Save a rule (create or update)
    fn save_rule(&self, rule: &Rule) -> Result<()>;

    /// Find a rule by ID
    fn find_rule_by_id(&self, id: Uuid) -> Result<Option<Rule>>;

    /// Find all rules
    fn find_all_rules(&self) -> Result<Vec<Rule>>;

    /// Find rules by category
    fn find_rules_by_category(&self, category: RuleCategory) -> Result<Vec<Rule>>;

    /// Find rules by trigger event
    fn find_rules_by_trigger(&self, trigger: TriggerEvent) -> Result<Vec<Rule>>;

    /// Find rules by status
    fn find_rules_by_status(&self, status: RuleStatus) -> Result<Vec<Rule>>;

    /// Find active rules for a specific jurisdiction
    fn find_active_rules_for_jurisdiction(&self, jurisdiction: &str) -> Result<Vec<Rule>>;

    /// Delete a rule
    fn delete_rule(&self, id: Uuid) -> Result<bool>;
}

/// Query parameters for searching rules
#[derive(Debug, Default)]
pub struct RuleQuery {
    pub category: Option<RuleCategory>,
    pub trigger: Option<TriggerEvent>,
    pub status: Option<RuleStatus>,
    pub jurisdiction: Option<String>,
    pub source: Option<RuleSource>,
    pub offset: usize,
    pub limit: usize,
}

/// Extended repository with advanced query capabilities
pub trait RuleQueryRepository: RulesRepository {
    /// Search rules with filters and pagination
    fn search_rules(&self, query: RuleQuery) -> Result<(Vec<Rule>, usize)>;
}
