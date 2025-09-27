//! Attorney-Case relationship domain model
//!
//! This module manages the relationships between attorneys and criminal cases,
//! tracking representation history and roles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Role of an attorney in a case
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepresentationRole {
    /// Primary defense counsel
    LeadCounsel,
    /// Assistant defense counsel
    CoCounsel,
    /// Federal prosecutor
    Prosecutor,
    /// Assistant prosecutor
    AssistantProsecutor,
    /// Court-appointed public defender
    PublicDefender,
    /// Standby counsel
    StandbyCounsel,
    /// Appellate counsel
    AppellateCounsel,
}

/// Represents an attorney's assignment to a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttorneyCaseAssignment {
    /// Unique assignment ID
    pub id: String,
    /// Attorney ID
    pub attorney_id: String,
    /// Case ID
    pub case_id: String,
    /// Role in the case
    pub role: RepresentationRole,
    /// Date assigned to the case
    pub assigned_date: DateTime<Utc>,
    /// Date removed from the case (if applicable)
    pub removed_date: Option<DateTime<Utc>>,
    /// Whether this is the current assignment
    pub is_active: bool,
    /// Notes about the assignment
    pub notes: Option<String>,
}

/// Request to assign an attorney to a case
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AssignAttorneyRequest {
    /// Attorney ID
    pub attorney_id: String,
    /// Case ID
    pub case_id: String,
    /// Role in the case
    pub role: RepresentationRole,
    /// Optional notes
    pub notes: Option<String>,
}

/// Request to remove an attorney from a case
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RemoveAttorneyRequest {
    /// Reason for removal
    pub reason: Option<String>,
}

/// Summary of an attorney's case load
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttorneyCaseLoad {
    /// Attorney ID
    pub attorney_id: String,
    /// Attorney name
    pub attorney_name: String,
    /// Number of active cases
    pub active_cases: usize,
    /// Number of completed cases
    pub completed_cases: usize,
    /// Active case assignments
    pub active_assignments: Vec<CaseSummary>,
}

/// Brief case information for listings
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CaseSummary {
    /// Case ID
    pub case_id: String,
    /// Case number
    pub case_number: String,
    /// Defendant name
    pub defendant_name: String,
    /// Attorney's role
    pub role: RepresentationRole,
    /// Date assigned
    pub assigned_date: DateTime<Utc>,
}

/// Attorney's complete representation history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttorneyRepresentationHistory {
    /// Attorney ID
    pub attorney_id: String,
    /// Attorney name
    pub attorney_name: String,
    /// All case assignments (current and historical)
    pub assignments: Vec<RepresentationHistoryEntry>,
    /// Summary statistics
    pub summary: RepresentationSummary,
}

/// Individual entry in representation history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RepresentationHistoryEntry {
    /// Assignment ID
    pub assignment_id: String,
    /// Case ID
    pub case_id: String,
    /// Case number
    pub case_number: String,
    /// Defendant name
    pub defendant_name: String,
    /// Attorney's role
    pub role: RepresentationRole,
    /// Date assigned to the case
    pub assigned_date: DateTime<Utc>,
    /// Date removed from the case (if applicable)
    pub removed_date: Option<DateTime<Utc>>,
    /// Whether this is currently active
    pub is_active: bool,
    /// Case outcome (if completed)
    pub case_outcome: Option<String>,
    /// Role changes over time
    pub role_changes: Vec<RoleChange>,
    /// Notes about the assignment
    pub notes: Option<String>,
}

/// Role change history within a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleChange {
    /// Previous role
    pub from_role: RepresentationRole,
    /// New role
    pub to_role: RepresentationRole,
    /// Date of role change
    pub change_date: DateTime<Utc>,
    /// Reason for change
    pub reason: Option<String>,
}

/// Summary statistics for representation history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RepresentationSummary {
    /// Total number of cases represented
    pub total_cases: usize,
    /// Number of currently active cases
    pub active_cases: usize,
    /// Number of completed cases
    pub completed_cases: usize,
    /// Most common role
    pub primary_role: RepresentationRole,
    /// Date range of representation
    pub date_range: DateRange,
    /// Cases by outcome
    pub outcomes: std::collections::HashMap<String, usize>,
}

/// Date range for filtering
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DateRange {
    /// Start date
    pub start_date: DateTime<Utc>,
    /// End date
    pub end_date: DateTime<Utc>,
}

/// Query parameters for representation history
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RepresentationHistoryQuery {
    /// Start date for filtering (ISO 8601)
    pub start_date: Option<String>,
    /// End date for filtering (ISO 8601)
    pub end_date: Option<String>,
    /// Filter by case status
    pub status: Option<String>,
    /// Filter by role
    pub role: Option<RepresentationRole>,
    /// Include only active assignments
    pub active_only: Option<bool>,
    /// Page number for pagination
    pub page: Option<usize>,
    /// Page size for pagination
    pub page_size: Option<usize>,
}

impl AttorneyCaseAssignment {
    /// Create a new assignment
    pub fn new(
        attorney_id: String,
        case_id: String,
        role: RepresentationRole,
        notes: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            attorney_id,
            case_id,
            role,
            assigned_date: Utc::now(),
            removed_date: None,
            is_active: true,
            notes,
        }
    }

    /// Mark assignment as inactive
    pub fn deactivate(&mut self, reason: Option<String>) {
        self.is_active = false;
        self.removed_date = Some(Utc::now());
        if let Some(reason) = reason {
            self.notes = Some(format!(
                "{}\nRemoval reason: {}",
                self.notes.as_deref().unwrap_or(""),
                reason
            ));
        }
    }
}