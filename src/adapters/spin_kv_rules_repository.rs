//! Spin Key-Value Store implementation for rules repository
//!
//! This adapter implements the RulesRepository traits using Spin's
//! built-in key-value store for persistence.

use crate::adapters::store_utils::open_validated_store;
use crate::domain::rule::{Rule, RuleCategory, RuleStatus, TriggerEvent};
use crate::ports::rules_repository::{RulesRepository, RuleQuery, RuleQueryRepository};
use anyhow::Result;
use spin_sdk::key_value::Store;
use uuid::Uuid;

const RULE_KEY_PREFIX: &str = "rule-";

/// Spin KV implementation of the RulesRepository
pub struct SpinKvRulesRepository {
    store: Store,
}

impl SpinKvRulesRepository {
    /// Create repository with specific store name for multi-tenancy
    pub fn with_store(store_name: String) -> Self {
        let store = open_validated_store(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }

    fn build_rule_key(id: Uuid) -> String {
        format!("{}{}", RULE_KEY_PREFIX, id)
    }
}

impl RulesRepository for SpinKvRulesRepository {
    fn save_rule(&self, rule: &Rule) -> Result<()> {
        let key = Self::build_rule_key(rule.id);
        self.store.set_json(&key, rule)?;
        Ok(())
    }

    fn find_rule_by_id(&self, id: Uuid) -> Result<Option<Rule>> {
        let key = Self::build_rule_key(id);
        self.store.get_json::<Rule>(&key)
    }

    fn find_all_rules(&self) -> Result<Vec<Rule>> {
        let rules: Vec<Rule> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(RULE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<Rule>(key.as_str()).ok())
            .filter_map(|rule| rule)
            .collect();

        Ok(rules)
    }

    fn find_rules_by_category(&self, category: RuleCategory) -> Result<Vec<Rule>> {
        let rules = self.find_all_rules()?;
        Ok(rules.into_iter().filter(|r| r.category == category).collect())
    }

    fn find_rules_by_trigger(&self, trigger: TriggerEvent) -> Result<Vec<Rule>> {
        let rules = self.find_all_rules()?;
        Ok(rules.into_iter().filter(|r| r.triggers.contains(&trigger)).collect())
    }

    fn find_rules_by_status(&self, status: RuleStatus) -> Result<Vec<Rule>> {
        let rules = self.find_all_rules()?;
        Ok(rules.into_iter().filter(|r| r.status == status).collect())
    }

    fn find_active_rules_for_jurisdiction(&self, jurisdiction: &str) -> Result<Vec<Rule>> {
        let rules = self.find_all_rules()?;
        let jurisdiction_lower = jurisdiction.to_lowercase();

        Ok(rules.into_iter().filter(|r| {
            r.is_in_effect() &&
            r.jurisdiction.as_ref().map_or(true, |j| {
                j.to_lowercase() == jurisdiction_lower
            })
        }).collect())
    }

    fn delete_rule(&self, id: Uuid) -> Result<bool> {
        let key = Self::build_rule_key(id);

        let exists = self.store.exists(&key)?;
        if exists {
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl RuleQueryRepository for SpinKvRulesRepository {
    fn search_rules(&self, query: RuleQuery) -> Result<(Vec<Rule>, usize)> {
        let mut rules = self.find_all_rules()?;

        // Apply filters
        if let Some(category) = query.category {
            rules.retain(|r| r.category == category);
        }

        if let Some(trigger) = query.trigger {
            rules.retain(|r| r.triggers.contains(&trigger));
        }

        if let Some(status) = query.status {
            rules.retain(|r| r.status == status);
        }

        if let Some(jurisdiction) = query.jurisdiction {
            let jurisdiction_lower = jurisdiction.to_lowercase();
            rules.retain(|r| {
                r.jurisdiction.as_ref().map_or(false, |j| {
                    j.to_lowercase() == jurisdiction_lower
                })
            });
        }

        if let Some(source) = query.source {
            rules.retain(|r| r.source == source);
        }

        // Sort by priority weight descending (highest priority first)
        rules.sort_by(|a, b| b.priority.weight().cmp(&a.priority.weight()));

        // Get total count before pagination
        let total = rules.len();

        // Apply pagination
        let paginated: Vec<Rule> = rules
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok((paginated, total))
    }
}
