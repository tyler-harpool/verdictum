//! Filing pipeline domain model for federal court system
//!
//! This module defines types for the document filing pipeline, including
//! compliance checking, filing submissions, and filing receipts. The
//! pipeline validates filings against court rules before accepting them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Context information about a filing used during rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilingContext {
    /// Type of case (e.g., "criminal", "civil")
    pub case_type: String,
    /// Type of document being filed (e.g., "motion", "brief")
    pub document_type: String,
    /// Role of the person filing (e.g., "plaintiff_attorney", "defendant")
    pub filer_role: String,
    /// Identifier for the jurisdiction (e.g., "CACD")
    pub jurisdiction_id: String,
    /// Division within the jurisdiction, if applicable
    pub division: Option<String>,
    /// Assigned judge for the case, if any
    pub assigned_judge: Option<String>,
    /// Service method used for the filing
    pub service_method: Option<super::deadline_calc::ServiceMethod>,
    /// Additional metadata as key-value pairs
    #[schema(value_type = Object)]
    pub metadata: serde_json::Value,
}

/// A document filing submission before compliance validation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilingSubmission {
    /// Case number in standard federal format
    pub case_number: String,
    /// Type of document being filed
    pub document_type: String,
    /// Name of the person filing the document
    pub filer_name: String,
    /// Role of the person filing
    pub filer_role: String,
    /// Full text of the document, if available
    pub document_text: Option<String>,
    /// Additional metadata as key-value pairs
    #[schema(value_type = Object)]
    pub metadata: serde_json::Value,
}

/// Report of compliance check results for a filing
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComplianceReport {
    /// Individual rule evaluation results
    pub results: Vec<RuleResult>,
    /// Whether the filing is blocked from proceeding
    pub blocked: bool,
    /// Reasons the filing is blocked, if any
    pub block_reasons: Vec<String>,
    /// Non-blocking warnings about the filing
    pub warnings: Vec<String>,
    /// Deadlines computed as a result of the filing
    pub deadlines: Vec<super::deadline_calc::DeadlineResult>,
}

/// Result of evaluating a single rule against a filing
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RuleResult {
    /// Identifier of the rule that was evaluated
    pub rule_id: Uuid,
    /// Name of the rule
    pub rule_name: String,
    /// Whether the rule's conditions were matched
    pub matched: bool,
    /// Description of the action taken (or would be taken)
    pub action_taken: String,
    /// Human-readable message about the evaluation result
    pub message: String,
}

/// Receipt issued after a filing is accepted into the system
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilingReceipt {
    /// Unique identifier for the filing
    pub filing_id: Uuid,
    /// Case number the document was filed in
    pub case_number: String,
    /// Timestamp when the filing was accepted
    pub filed_at: DateTime<Utc>,
    /// Type of document that was filed
    pub document_type: String,
    /// Assigned docket entry number, if available
    pub docket_number: Option<u32>,
    /// Compliance report for the filing
    pub compliance_report: ComplianceReport,
    /// Notice of Electronic Filing, if generated
    pub nef: Option<super::nef::NoticeOfElectronicFiling>,
}
