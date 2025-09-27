//! Attorney Conflict of Interest Management
//!
//! This module provides comprehensive conflict checking capabilities for attorneys
//! to identify potential conflicts of interest before taking on new cases.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to check for conflicts of interest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictCheckRequest {
    /// Names of parties to check for conflicts against
    pub parties_to_check: Vec<String>,
    /// Names of adverse parties in the potential representation
    pub adverse_parties: Vec<String>,
    /// Optional case ID if this is for a specific case
    pub case_id: Option<String>,
    /// Matter description
    pub matter_description: String,
    /// Court or jurisdiction where representation would occur
    pub jurisdiction: Option<String>,
}

/// Result of a conflict check containing all identified conflicts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictCheckResult {
    /// Unique identifier for this conflict check
    pub check_id: String,
    /// Attorney ID that the check was performed for
    pub attorney_id: String,
    /// Timestamp when the check was performed
    pub check_date: DateTime<Utc>,
    /// Whether any conflicts were found
    pub has_conflicts: bool,
    /// List of all conflicts identified
    pub conflicts: Vec<ConflictDetails>,
    /// Overall recommendation for the attorney
    pub recommendation: ConflictRecommendation,
    /// Additional notes or considerations
    pub notes: Option<String>,
}

/// Types of conflicts that can be identified
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum ConflictType {
    /// Currently representing the party or an adverse party
    DirectRepresentation,
    /// Previously represented the party and matter is substantially related
    FormerClient,
    /// Representing a co-defendant in same or related matter
    CoDefendant,
    /// Personal interest in the outcome of the matter
    PersonalInterest,
    /// Financial interest in a party or the outcome
    FinancialInterest,
    /// Family relationship with a party
    FamilyRelationship,
    /// Business relationship with a party
    BusinessRelationship,
    /// Attorney's law firm represents an adverse party
    FirmConflict,
    /// Positional conflict - taking inconsistent legal positions
    PositionalConflict,
    /// Issue conflict - working on matters with conflicting interests
    IssueConflict,
    /// Other type of conflict
    Other(String),
}

/// Detailed information about a specific conflict
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictDetails {
    /// Unique identifier for this conflict
    pub conflict_id: String,
    /// Type of conflict identified
    pub conflict_type: ConflictType,
    /// Name of the party involved in the conflict
    pub conflicted_party: String,
    /// Description of the conflict
    pub description: String,
    /// Related case ID if applicable
    pub related_case_id: Option<String>,
    /// Related matter description
    pub related_matter: String,
    /// When the conflicting relationship began
    pub relationship_start_date: Option<DateTime<Utc>>,
    /// When the conflicting relationship ended (if applicable)
    pub relationship_end_date: Option<DateTime<Utc>>,
    /// Severity level of the conflict
    pub severity: ConflictSeverity,
    /// Whether this conflict can be waived with client consent
    pub waivable: bool,
    /// Whether a waiver has been obtained
    pub waiver_obtained: bool,
    /// Date waiver was obtained
    pub waiver_date: Option<DateTime<Utc>>,
    /// Additional details about the conflict
    pub additional_details: Option<String>,
}

/// Severity levels for conflicts of interest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum ConflictSeverity {
    /// No actual conflict, but flagged for review
    Informational,
    /// Minor conflict that can typically be waived
    Low,
    /// Moderate conflict requiring careful consideration
    Medium,
    /// Serious conflict that should generally be avoided
    High,
    /// Severe conflict that cannot be waived
    Critical,
}

/// Overall recommendation based on conflict check results
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum ConflictRecommendation {
    /// No conflicts found, safe to proceed
    Proceed,
    /// Minor conflicts found, proceed with caution and consider waivers
    ProceedWithCaution,
    /// Significant conflicts found, obtain waivers before proceeding
    RequireWaivers,
    /// Serious conflicts found, recommend declining representation
    Decline,
    /// Conflicts cannot be waived, must decline representation
    MustDecline,
}

/// Historical conflict check record for audit purposes
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictCheckHistory {
    /// Unique identifier for the historical record
    pub id: String,
    /// The original conflict check result
    pub conflict_check: ConflictCheckResult,
    /// Who performed the conflict check
    pub performed_by: String,
    /// Any actions taken as a result of the check
    pub actions_taken: Vec<ConflictAction>,
    /// Whether the representation was accepted or declined
    pub final_decision: Option<RepresentationDecision>,
    /// Date of final decision
    pub decision_date: Option<DateTime<Utc>>,
}

/// Actions that can be taken in response to conflicts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ConflictAction {
    /// Obtained waiver from affected client
    WaiverObtained { client_name: String, waiver_date: DateTime<Utc> },
    /// Declined the representation
    RepresentationDeclined { reason: String },
    /// Referred case to another attorney
    CaseReferred { referred_to: String },
    /// Withdrew from existing representation
    WithdrewFromExisting { case_id: String, withdrawal_date: DateTime<Utc> },
    /// Sought ethics opinion
    EthicsOpinionRequested { request_date: DateTime<Utc> },
    /// Other action taken
    Other { description: String, action_date: DateTime<Utc> },
}

/// Final decision on whether to accept representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum RepresentationDecision {
    /// Accepted the representation
    Accepted,
    /// Declined the representation
    Declined,
    /// Still under consideration
    Pending,
}

impl ConflictCheckRequest {
    /// Create a new conflict check request
    pub fn new(
        parties_to_check: Vec<String>,
        adverse_parties: Vec<String>,
        matter_description: String,
    ) -> Self {
        Self {
            parties_to_check,
            adverse_parties,
            case_id: None,
            matter_description,
            jurisdiction: None,
        }
    }

    /// Add a case ID to the request
    pub fn with_case_id(mut self, case_id: String) -> Self {
        self.case_id = Some(case_id);
        self
    }

    /// Add jurisdiction information to the request
    pub fn with_jurisdiction(mut self, jurisdiction: String) -> Self {
        self.jurisdiction = Some(jurisdiction);
        self
    }
}

impl ConflictCheckResult {
    /// Create a new conflict check result with no conflicts
    pub fn no_conflicts(attorney_id: String, request: &ConflictCheckRequest) -> Self {
        Self {
            check_id: Uuid::new_v4().to_string(),
            attorney_id,
            check_date: Utc::now(),
            has_conflicts: false,
            conflicts: Vec::new(),
            recommendation: ConflictRecommendation::Proceed,
            notes: None,
        }
    }

    /// Create a new conflict check result with identified conflicts
    pub fn with_conflicts(
        attorney_id: String,
        conflicts: Vec<ConflictDetails>,
        recommendation: ConflictRecommendation,
    ) -> Self {
        let has_conflicts = !conflicts.is_empty();
        Self {
            check_id: Uuid::new_v4().to_string(),
            attorney_id,
            check_date: Utc::now(),
            has_conflicts,
            conflicts,
            recommendation,
            notes: None,
        }
    }

    /// Add notes to the conflict check result
    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Get the highest severity level among all conflicts
    pub fn max_severity(&self) -> Option<ConflictSeverity> {
        self.conflicts.iter()
            .map(|c| &c.severity)
            .max_by(|a, b| conflict_severity_order(a).cmp(&conflict_severity_order(b)))
            .cloned()
    }

    /// Check if any conflicts are non-waivable
    pub fn has_non_waivable_conflicts(&self) -> bool {
        self.conflicts.iter().any(|c| !c.waivable)
    }

    /// Get conflicts by type
    pub fn conflicts_by_type(&self, conflict_type: &ConflictType) -> Vec<&ConflictDetails> {
        self.conflicts.iter()
            .filter(|c| &c.conflict_type == conflict_type)
            .collect()
    }
}

impl ConflictDetails {
    /// Create a new conflict detail
    pub fn new(
        conflict_type: ConflictType,
        conflicted_party: String,
        description: String,
        severity: ConflictSeverity,
        waivable: bool,
    ) -> Self {
        Self {
            conflict_id: Uuid::new_v4().to_string(),
            conflict_type,
            conflicted_party,
            description,
            related_case_id: None,
            related_matter: String::new(),
            relationship_start_date: None,
            relationship_end_date: None,
            severity,
            waivable,
            waiver_obtained: false,
            waiver_date: None,
            additional_details: None,
        }
    }

    /// Check if this conflict is currently active
    pub fn is_active(&self) -> bool {
        self.relationship_end_date.is_none() ||
        self.relationship_end_date.map(|end| end > Utc::now()).unwrap_or(false)
    }

    /// Check if a waiver is required and not yet obtained
    pub fn requires_waiver(&self) -> bool {
        self.waivable && !self.waiver_obtained && self.severity != ConflictSeverity::Informational
    }
}

/// Helper function to determine conflict severity ordering for comparison
fn conflict_severity_order(severity: &ConflictSeverity) -> u8 {
    match severity {
        ConflictSeverity::Informational => 0,
        ConflictSeverity::Low => 1,
        ConflictSeverity::Medium => 2,
        ConflictSeverity::High => 3,
        ConflictSeverity::Critical => 4,
    }
}

impl ConflictType {
    /// Check if this conflict type typically requires special handling
    pub fn requires_special_handling(&self) -> bool {
        matches!(self,
            ConflictType::DirectRepresentation |
            ConflictType::FormerClient |
            ConflictType::FirmConflict |
            ConflictType::PositionalConflict
        )
    }

    /// Get a human-readable description of the conflict type
    pub fn description(&self) -> &str {
        match self {
            ConflictType::DirectRepresentation => "Currently representing party or adverse party",
            ConflictType::FormerClient => "Previously represented party in substantially related matter",
            ConflictType::CoDefendant => "Representing co-defendant in same or related matter",
            ConflictType::PersonalInterest => "Personal interest in the outcome",
            ConflictType::FinancialInterest => "Financial interest in party or outcome",
            ConflictType::FamilyRelationship => "Family relationship with party",
            ConflictType::BusinessRelationship => "Business relationship with party",
            ConflictType::FirmConflict => "Law firm represents adverse party",
            ConflictType::PositionalConflict => "Taking inconsistent legal positions",
            ConflictType::IssueConflict => "Working on matters with conflicting interests",
            ConflictType::Other(_) => "Other type of conflict",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_check_request_creation() {
        let parties = vec!["John Doe".to_string(), "Jane Smith".to_string()];
        let adverse = vec!["ABC Corp".to_string()];
        let matter = "Personal injury lawsuit".to_string();

        let request = ConflictCheckRequest::new(parties.clone(), adverse.clone(), matter.clone());

        assert_eq!(request.parties_to_check, parties);
        assert_eq!(request.adverse_parties, adverse);
        assert_eq!(request.matter_description, matter);
        assert!(request.case_id.is_none());
        assert!(request.jurisdiction.is_none());
    }

    #[test]
    fn test_conflict_check_result_no_conflicts() {
        let attorney_id = "attorney-123".to_string();
        let request = ConflictCheckRequest::new(
            vec!["Test Party".to_string()],
            vec!["Adverse Party".to_string()],
            "Test matter".to_string(),
        );

        let result = ConflictCheckResult::no_conflicts(attorney_id.clone(), &request);

        assert_eq!(result.attorney_id, attorney_id);
        assert!(!result.has_conflicts);
        assert!(result.conflicts.is_empty());
        assert_eq!(result.recommendation, ConflictRecommendation::Proceed);
    }

    #[test]
    fn test_conflict_details_creation() {
        let conflict = ConflictDetails::new(
            ConflictType::FormerClient,
            "John Doe".to_string(),
            "Previously represented in similar matter".to_string(),
            ConflictSeverity::Medium,
            true,
        );

        assert_eq!(conflict.conflict_type, ConflictType::FormerClient);
        assert_eq!(conflict.conflicted_party, "John Doe");
        assert_eq!(conflict.severity, ConflictSeverity::Medium);
        assert!(conflict.waivable);
        assert!(!conflict.waiver_obtained);
    }

    #[test]
    fn test_conflict_severity_ordering() {
        assert!(conflict_severity_order(&ConflictSeverity::Critical) >
                conflict_severity_order(&ConflictSeverity::High));
        assert!(conflict_severity_order(&ConflictSeverity::High) >
                conflict_severity_order(&ConflictSeverity::Medium));
        assert!(conflict_severity_order(&ConflictSeverity::Medium) >
                conflict_severity_order(&ConflictSeverity::Low));
        assert!(conflict_severity_order(&ConflictSeverity::Low) >
                conflict_severity_order(&ConflictSeverity::Informational));
    }

    #[test]
    fn test_conflict_type_special_handling() {
        assert!(ConflictType::DirectRepresentation.requires_special_handling());
        assert!(ConflictType::FormerClient.requires_special_handling());
        assert!(!ConflictType::PersonalInterest.requires_special_handling());
    }
}