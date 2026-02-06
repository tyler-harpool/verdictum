//! Port trait for the rules evaluation engine
//!
//! Defines the contract for evaluating court rules against filing contexts.
//! The engine selects applicable rules, resolves priority ordering, evaluates
//! conditions recursively, and produces a compliance report.

use crate::domain::filing_pipeline::{ComplianceReport, FilingContext};
use crate::domain::rule::{Rule, RuleCondition, TriggerEvent};
use crate::error::ApiError;

/// Port trait for the rules evaluation engine
///
/// Implementations evaluate a set of court rules against a filing context
/// to produce a compliance report indicating whether a filing should be
/// blocked, flagged for review, or allowed to proceed.
pub trait RulesEngine {
    /// Evaluate a set of rules against a filing context
    ///
    /// Selects applicable rules, sorts by priority, evaluates conditions,
    /// and collects all actions into a single compliance report.
    fn evaluate(&self, context: &FilingContext, rules: &[Rule]) -> Result<ComplianceReport, ApiError>;

    /// Select applicable rules for a given jurisdiction and trigger event
    ///
    /// Filters rules by jurisdiction match (or global applicability),
    /// trigger event presence, and whether the rule is currently in effect.
    fn select_rules(&self, jurisdiction: &str, trigger: &TriggerEvent, all_rules: &[Rule]) -> Vec<Rule>;

    /// Sort rules by priority weight (highest weight first)
    fn resolve_priority(&self, matching_rules: Vec<Rule>) -> Vec<Rule>;

    /// Evaluate a single condition against a filing context
    ///
    /// Recursively evaluates compound conditions (And, Or, Not) and
    /// checks field-level conditions against context fields and metadata.
    fn evaluate_condition(&self, condition: &RuleCondition, context: &FilingContext) -> bool;
}
