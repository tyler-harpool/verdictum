//! Domain models for judicial orders
//!
//! This module defines the core types for managing court orders,
//! including minute orders, scheduling orders, and procedural orders.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export common types so existing import paths continue to work
pub use super::common::{ElectronicSignature, ServiceMethod, ServiceStatus};

/// Represents a judicial order in the system
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JudicialOrder {
    pub id: String,
    pub case_id: String,
    pub judge_id: String,
    pub order_type: OrderType,
    pub title: String,
    pub content: String,
    pub status: OrderStatus,
    pub is_sealed: bool,
    pub signature: Option<ElectronicSignature>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub issued_at: Option<DateTime<Utc>>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub related_motions: Vec<String>,
    pub attachments: Vec<String>,
    pub service_list: Vec<ServiceRecord>,
    pub metadata: OrderMetadata,
}

/// Types of judicial orders
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum OrderType {
    MinuteOrder,
    SchedulingOrder,
    ProtectiveOrder,
    DiscoveryOrder,
    SealingOrder,
    ShowCauseOrder,
    TemporaryRestrainingOrder,
    PreliminaryInjunction,
    PermanentInjunction,
    SentencingOrder,
    PretrialOrder,
    DefaultJudgment,
    SummaryJudgment,
    DismissalOrder,
    Other(String),
}

/// Status of a judicial order
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum OrderStatus {
    Draft,
    PendingReview,
    PendingSignature,
    Signed,
    Issued,
    Served,
    Stayed,
    Vacated,
    Appealed,
    Modified,
    Expired,
}

// ElectronicSignature is imported from common module

/// Service record for an order
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceRecord {
    pub party_id: String,
    pub party_name: String,
    pub method: ServiceMethod,
    pub served_at: Option<DateTime<Utc>>,
    pub served_by: Option<String>,
    pub proof_of_service: Option<String>,
    pub status: ServiceStatus,
}

// ServiceMethod and ServiceStatus are imported from common module

/// Metadata for orders
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderMetadata {
    pub requires_compliance: bool,
    pub compliance_deadline: Option<DateTime<Utc>>,
    pub appealable: bool,
    pub appeal_deadline: Option<DateTime<Utc>>,
    pub related_orders: Vec<String>,
    pub supersedes: Option<String>,
    pub tags: Vec<String>,
    pub notes: String,
}

/// Template for creating orders
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderTemplate {
    pub id: String,
    pub name: String,
    pub order_type: OrderType,
    pub description: String,
    pub template_content: String,
    pub variables: Vec<TemplateVariable>,
    pub required_attachments: Vec<String>,
    pub default_service_method: ServiceMethod,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Variable in an order template
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub variable_type: VariableType,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation_rules: Vec<String>,
}

/// Type of template variable
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VariableType {
    Text,
    Number,
    Date,
    Boolean,
    Selection(Vec<String>),
    CaseReference,
    PartyReference,
    JudgeReference,
}

impl JudicialOrder {
    /// Create a new draft order
    pub fn new(
        case_id: String,
        judge_id: String,
        order_type: OrderType,
        title: String,
        content: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            case_id,
            judge_id,
            order_type,
            title,
            content,
            status: OrderStatus::Draft,
            is_sealed: false,
            signature: None,
            created_at: now,
            updated_at: now,
            issued_at: None,
            effective_date: None,
            expiration_date: None,
            related_motions: Vec::new(),
            attachments: Vec::new(),
            service_list: Vec::new(),
            metadata: OrderMetadata {
                requires_compliance: false,
                compliance_deadline: None,
                appealable: true,
                appeal_deadline: None,
                related_orders: Vec::new(),
                supersedes: None,
                tags: Vec::new(),
                notes: String::new(),
            },
        }
    }

    /// Sign the order electronically
    pub fn sign(&mut self, signature: ElectronicSignature) {
        self.signature = Some(signature);
        self.status = OrderStatus::Signed;
        self.updated_at = Utc::now();
    }

    /// Issue the order
    pub fn issue(&mut self) {
        self.status = OrderStatus::Issued;
        self.issued_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Add a service record
    pub fn add_service_record(&mut self, record: ServiceRecord) {
        self.service_list.push(record);
        self.updated_at = Utc::now();
    }

    /// Check if order is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration_date {
            Utc::now() > expiration
        } else {
            false
        }
    }

    /// Check if order requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(
            self.order_type,
            OrderType::TemporaryRestrainingOrder
                | OrderType::ShowCauseOrder
                | OrderType::SealingOrder
        ) && matches!(self.status, OrderStatus::Draft | OrderStatus::PendingSignature)
    }
}

impl OrderTemplate {
    /// Create a new order template
    pub fn new(
        name: String,
        order_type: OrderType,
        description: String,
        template_content: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            order_type,
            description,
            template_content,
            variables: Vec::new(),
            required_attachments: Vec::new(),
            default_service_method: ServiceMethod::ElectronicFiling,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate order content from template
    pub fn generate_content(&self, values: &std::collections::HashMap<String, String>) -> String {
        let mut content = self.template_content.clone();
        for variable in &self.variables {
            if let Some(value) = values.get(&variable.name) {
                content = content.replace(&format!("{{{{{}}}}}", variable.name), value);
            }
        }
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let order = JudicialOrder::new(
            "CASE-123".to_string(),
            "JUDGE-456".to_string(),
            OrderType::SchedulingOrder,
            "Scheduling Order".to_string(),
            "The court hereby orders...".to_string(),
        );

        assert_eq!(order.case_id, "CASE-123");
        assert_eq!(order.judge_id, "JUDGE-456");
        assert!(matches!(order.status, OrderStatus::Draft));
        assert!(!order.is_sealed);
    }

    #[test]
    fn test_template_content_generation() {
        let mut template = OrderTemplate::new(
            "Scheduling Order Template".to_string(),
            OrderType::SchedulingOrder,
            "Standard scheduling order".to_string(),
            "The court orders that {{party_name}} shall {{action}} by {{deadline}}.".to_string(),
        );

        template.variables.push(TemplateVariable {
            name: "party_name".to_string(),
            description: "Name of the party".to_string(),
            variable_type: VariableType::Text,
            required: true,
            default_value: None,
            validation_rules: Vec::new(),
        });

        let mut values = std::collections::HashMap::new();
        values.insert("party_name".to_string(), "Defendant Smith".to_string());
        values.insert("action".to_string(), "file a response".to_string());
        values.insert("deadline".to_string(), "January 15, 2025".to_string());

        let content = template.generate_content(&values);
        assert!(content.contains("Defendant Smith"));
        assert!(content.contains("file a response"));
        assert!(content.contains("January 15, 2025"));
    }
}