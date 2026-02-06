//! Repository port for criminal case persistence
//!
//! This trait defines the contract for storing and retrieving criminal cases,
//! allowing the domain to be independent of the storage implementation.

use crate::domain::criminal_case::{CaseStatus, CasePriority, CriminalCase};
use anyhow::Result;
use uuid::Uuid;

/// Repository trait for criminal case persistence
///
/// This port defines all operations needed for case management.
/// Implementations could use Spin KV, PostgreSQL, MongoDB, etc.
pub trait CaseRepository {
    /// Save a case (create or update)
    fn save(&self, case: &CriminalCase) -> Result<()>;

    /// Find a case by its ID
    fn find_by_id(&self, id: Uuid) -> Result<Option<CriminalCase>>;

    /// Find a case by its case number
    fn find_by_case_number(&self, case_number: &str) -> Result<Option<CriminalCase>>;

    /// Find all cases
    fn find_all_cases(&self) -> Result<Vec<CriminalCase>>;

    /// Find cases by status
    fn find_by_status(&self, status: CaseStatus) -> Result<Vec<CriminalCase>>;

    /// Find cases by assigned judge ID
    fn find_by_judge(&self, judge_id: Uuid) -> Result<Vec<CriminalCase>>;

    /// Delete a case (returns true if case existed)
    fn delete(&self, id: Uuid) -> Result<bool>;

    /// Count cases by status
    fn count_by_status(&self, status: CaseStatus) -> Result<usize>;
}

/// Query parameters for searching cases
#[derive(Debug, Default)]
pub struct CaseQuery {
    pub status: Option<CaseStatus>,
    pub priority: Option<CasePriority>,
    pub judge_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub offset: usize,
    pub limit: usize,
}

/// Extended repository with advanced query capabilities
pub trait CaseQueryRepository: CaseRepository {
    /// Search cases with filters and pagination
    fn search(&self, query: CaseQuery) -> Result<(Vec<CriminalCase>, usize)>;

    /// Get case statistics
    fn get_statistics(&self) -> Result<CaseStatistics>;
}

/// Statistics about cases in the system
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CaseStatistics {
    pub total_cases: usize,
    pub open_cases: usize,
    pub under_investigation: usize,
    pub closed_cases: usize,
    pub cold_cases: usize,
    pub critical_priority: usize,
    pub high_priority: usize,
}
