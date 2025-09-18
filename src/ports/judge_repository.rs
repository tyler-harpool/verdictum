//! Repository port for judge persistence
//!
//! This trait defines the contract for storing and retrieving judges,
//! assignments, and recusals in the federal court system.

use crate::domain::judge::{Judge, CaseAssignment, RecusalMotion, JudgeStatus, JudgeTitle, ConflictOfInterest};
use anyhow::Result;
use uuid::Uuid;

/// Repository trait for judge persistence
pub trait JudgeRepository {
    /// Save a judge (create or update)
    fn save_judge(&self, judge: &Judge) -> Result<()>;

    /// Find a judge by ID
    fn find_judge_by_id(&self, id: Uuid) -> Result<Option<Judge>>;

    /// Find all judges
    fn find_all_judges(&self) -> Result<Vec<Judge>>;

    /// Find judges by status
    fn find_judges_by_status(&self, status: JudgeStatus) -> Result<Vec<Judge>>;

    /// Find judges by district
    fn find_judges_by_district(&self, district: &str) -> Result<Vec<Judge>>;

    /// Find available judges for assignment
    fn find_available_judges(&self) -> Result<Vec<Judge>>;

    /// Delete a judge
    fn delete_judge(&self, id: Uuid) -> Result<bool>;
}

/// Repository trait for case assignment persistence
pub trait CaseAssignmentRepository {
    /// Save a case assignment
    fn save_assignment(&self, assignment: &CaseAssignment) -> Result<()>;

    /// Find assignment by case ID
    fn find_assignment_by_case(&self, case_id: Uuid) -> Result<Option<CaseAssignment>>;

    /// Find all assignments for a judge
    fn find_assignments_by_judge(&self, judge_id: Uuid) -> Result<Vec<CaseAssignment>>;

    /// Find assignment history for a case
    fn find_assignment_history(&self, case_id: Uuid) -> Result<Vec<CaseAssignment>>;

    /// Delete an assignment
    fn delete_assignment(&self, id: Uuid) -> Result<bool>;
}

/// Repository trait for recusal motion persistence
pub trait RecusalRepository {
    /// Save a recusal motion
    fn save_recusal(&self, motion: &RecusalMotion) -> Result<()>;

    /// Find recusal by ID
    fn find_recusal_by_id(&self, id: Uuid) -> Result<Option<RecusalMotion>>;

    /// Find recusals by case
    fn find_recusals_by_case(&self, case_id: Uuid) -> Result<Vec<RecusalMotion>>;

    /// Find recusals by judge
    fn find_recusals_by_judge(&self, judge_id: Uuid) -> Result<Vec<RecusalMotion>>;

    /// Find pending recusals
    fn find_pending_recusals(&self) -> Result<Vec<RecusalMotion>>;
}

/// Repository trait for conflict of interest tracking
pub trait ConflictRepository {
    /// Save a conflict of interest
    fn save_conflict(&self, judge_id: Uuid, conflict: &ConflictOfInterest) -> Result<()>;

    /// Find conflicts for a judge
    fn find_conflicts_by_judge(&self, judge_id: Uuid) -> Result<Vec<ConflictOfInterest>>;

    /// Find conflicts by party name
    fn find_conflicts_by_party(&self, party_name: &str) -> Result<Vec<(Uuid, ConflictOfInterest)>>;

    /// Check if judge has conflict with party
    fn has_conflict(&self, judge_id: Uuid, party_name: &str) -> Result<bool>;

    /// Delete a conflict
    fn delete_conflict(&self, judge_id: Uuid, conflict_id: Uuid) -> Result<bool>;
}

/// Query parameters for searching judges
#[derive(Debug, Default)]
pub struct JudgeQuery {
    pub status: Option<JudgeStatus>,
    pub title: Option<JudgeTitle>,
    pub district: Option<String>,
    pub accepts_criminal: Option<bool>,
    pub accepts_civil: Option<bool>,
    pub max_caseload_percentage: Option<f32>,
    pub offset: usize,
    pub limit: usize,
}

/// Extended repository with advanced query capabilities
pub trait JudgeQueryRepository: JudgeRepository {
    /// Search judges with filters and pagination
    fn search_judges(&self, query: JudgeQuery) -> Result<(Vec<Judge>, usize)>;

    /// Get workload statistics
    fn get_workload_statistics(&self) -> Result<WorkloadStatistics>;

    /// Find judges with upcoming vacations
    fn find_judges_on_vacation(&self, start_date: chrono::DateTime<chrono::Utc>, end_date: chrono::DateTime<chrono::Utc>) -> Result<Vec<Judge>>;
}

/// Workload statistics for judges
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct WorkloadStatistics {
    pub total_judges: usize,
    pub active_judges: usize,
    pub senior_judges: usize,
    pub average_caseload: f32,
    pub total_cases: usize,
    pub overloaded_judges: usize, // Judges at >90% capacity
    pub available_capacity: usize, // Total available case slots
}