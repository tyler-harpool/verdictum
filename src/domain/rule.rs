//! Rule engine domain model for federal court system
//!
//! This module defines structured rules that govern court procedures, deadlines,
//! filing requirements, and policies. Phase 1 covers CRUD management of rules;
//! evaluation logic is deferred to Phase 2.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

/// A court rule governing procedures, deadlines, or policies
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub source: RuleSource,
    pub category: RuleCategory,
    pub triggers: Vec<TriggerEvent>,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub priority: RulePriority,
    pub status: RuleStatus,
    pub jurisdiction: Option<String>,
    pub citation: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub supersedes_rule_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

impl Rule {
    /// Create a new rule with default timestamps
    pub fn new(
        name: String,
        description: String,
        source: RuleSource,
        category: RuleCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            source,
            category,
            triggers: Vec::new(),
            conditions: Vec::new(),
            actions: Vec::new(),
            priority: RulePriority::FederalRule,
            status: RuleStatus::Draft,
            jurisdiction: None,
            citation: None,
            effective_date: None,
            expiration_date: None,
            supersedes_rule_id: None,
            created_at: now,
            updated_at: now,
            created_by: None,
        }
    }

    /// Check if the rule is currently in effect based on dates and status
    pub fn is_in_effect(&self) -> bool {
        if self.status != RuleStatus::Active {
            return false;
        }

        let now = Utc::now();

        if let Some(effective) = self.effective_date {
            if now < effective {
                return false;
            }
        }

        if let Some(expiration) = self.expiration_date {
            if now > expiration {
                return false;
            }
        }

        true
    }
}

/// Source authority for a court rule
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleSource {
    Frcp,
    FrcrP,
    Fre,
    Frap,
    LocalRule,
    AdminProcedure,
    StandingOrder,
    Statute,
    GeneralOrder,
}

/// Category of court rule
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleCategory {
    Filing,
    Deadline,
    Privacy,
    Procedural,
    Fee,
    Assignment,
    Service,
    Sealing,
    Sentencing,
    Discovery,
}

/// Events that can trigger rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerEvent {
    CaseFiled,
    MotionFiled,
    OrderIssued,
    DocumentFiled,
    StatusChanged,
    DeadlineApproaching,
    PleaEntered,
    SentencingScheduled,
    CaseAssigned,
    CaseReassigned,
    AppearanceFiled,
    ExtensionRequested,
    ManualEvaluation,
}

/// Actions a rule can produce when triggered (Phase 2 evaluation)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleAction {
    GenerateDeadline {
        description: String,
        days_from_trigger: i32,
    },
    RequireRedaction {
        fields: Vec<String>,
    },
    SendNotification {
        recipient: String,
        message: String,
    },
    BlockFiling {
        reason: String,
    },
    RequireFee {
        amount_cents: u64,
        description: String,
    },
    FlagForReview {
        reason: String,
    },
    LogCompliance {
        message: String,
    },
}

/// Priority level for rule evaluation ordering
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RulePriority {
    Statutory,
    FederalRule,
    Administrative,
    Local,
    StandingOrderPriority,
}

impl RulePriority {
    /// Numeric weight for sorting (higher = higher priority)
    pub fn weight(&self) -> u32 {
        match self {
            RulePriority::StandingOrderPriority => 50,
            RulePriority::Local => 40,
            RulePriority::Administrative => 30,
            RulePriority::FederalRule => 20,
            RulePriority::Statutory => 10,
        }
    }
}

/// Status of a rule in its lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleStatus {
    Active,
    Inactive,
    Draft,
    Superseded,
    Archived,
}

/// Recursive condition tree for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleCondition {
    And { conditions: Vec<RuleCondition> },
    Or { conditions: Vec<RuleCondition> },
    Not {
        #[schema(value_type = Object)]
        condition: Box<RuleCondition>,
    },
    FieldEquals { field: String, value: String },
    FieldContains { field: String, value: String },
    FieldExists { field: String },
    FieldGreaterThan { field: String, value: String },
    FieldLessThan { field: String, value: String },
    Always,
}

/// Request to create a new rule
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: String,
    pub source: RuleSource,
    pub category: RuleCategory,
    #[serde(default)]
    pub triggers: Vec<TriggerEvent>,
    #[serde(default)]
    pub conditions: Vec<RuleCondition>,
    #[serde(default)]
    pub actions: Vec<RuleAction>,
    pub priority: Option<RulePriority>,
    pub status: Option<RuleStatus>,
    pub jurisdiction: Option<String>,
    pub citation: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub supersedes_rule_id: Option<Uuid>,
    pub created_by: Option<String>,
}

/// Request to update an existing rule (partial update)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: Option<RuleSource>,
    pub category: Option<RuleCategory>,
    pub triggers: Option<Vec<TriggerEvent>>,
    pub conditions: Option<Vec<RuleCondition>>,
    pub actions: Option<Vec<RuleAction>>,
    pub priority: Option<RulePriority>,
    pub status: Option<RuleStatus>,
    pub jurisdiction: Option<String>,
    pub citation: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub supersedes_rule_id: Option<Uuid>,
}

/// Placeholder for Phase 2 compliance evaluation results
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComplianceResult {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub compliant: bool,
    pub message: String,
    pub evaluated_at: DateTime<Utc>,
}
