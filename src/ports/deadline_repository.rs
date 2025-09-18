//! Repository port for deadline tracking and compliance
//!
//! This trait defines the contract for storing and retrieving deadlines,
//! extensions, and compliance data in the federal court system.

use crate::domain::deadline::{Deadline, DeadlineType, DeadlineStatus, ExtensionRequest, DeadlineReminder};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Repository trait for deadline persistence
pub trait DeadlineRepository {
    /// Save a deadline
    fn save_deadline(&self, deadline: &Deadline) -> Result<()>;

    /// Find deadline by ID
    fn find_deadline_by_id(&self, id: Uuid) -> Result<Option<Deadline>>;

    /// Find all deadlines for a case
    fn find_deadlines_by_case(&self, case_id: Uuid) -> Result<Vec<Deadline>>;

    /// Find deadlines by type
    fn find_deadlines_by_type(&self, case_id: Uuid, deadline_type: DeadlineType) -> Result<Vec<Deadline>>;

    /// Find deadlines by status
    fn find_deadlines_by_status(&self, status: DeadlineStatus) -> Result<Vec<Deadline>>;

    /// Find deadlines for a party
    fn find_deadlines_by_party(&self, party_name: &str) -> Result<Vec<Deadline>>;

    /// Find upcoming deadlines
    fn find_upcoming_deadlines(&self, days_ahead: i64) -> Result<Vec<Deadline>>;

    /// Update deadline status
    fn update_deadline_status(&self, id: Uuid, status: DeadlineStatus) -> Result<()>;

    /// Mark deadline as completed
    fn complete_deadline(&self, id: Uuid, completion_date: DateTime<Utc>) -> Result<()>;

    /// Delete a deadline
    fn delete_deadline(&self, id: Uuid) -> Result<bool>;
}

/// Repository trait for extension request persistence
pub trait ExtensionRepository {
    /// Save an extension request
    fn save_extension(&self, deadline_id: Uuid, extension: &ExtensionRequest) -> Result<()>;

    /// Find extension by ID
    fn find_extension_by_id(&self, id: Uuid) -> Result<Option<ExtensionRequest>>;

    /// Find extensions for a deadline
    fn find_extensions_by_deadline(&self, deadline_id: Uuid) -> Result<Vec<ExtensionRequest>>;

    /// Find pending extensions
    fn find_pending_extensions(&self) -> Result<Vec<(Uuid, ExtensionRequest)>>;

    /// Update extension status
    fn update_extension_status(&self, id: Uuid, status: crate::domain::deadline::ExtensionStatus) -> Result<()>;
}

/// Repository trait for reminder tracking
pub trait ReminderRepository {
    /// Save reminders sent
    fn save_reminders(&self, reminders: &[DeadlineReminder]) -> Result<()>;

    /// Find reminders for a deadline
    fn find_reminders_by_deadline(&self, deadline_id: Uuid) -> Result<Vec<DeadlineReminder>>;

    /// Find reminders for a recipient
    fn find_reminders_by_recipient(&self, recipient: &str) -> Result<Vec<DeadlineReminder>>;

    /// Mark reminder as acknowledged
    fn acknowledge_reminder(&self, reminder_id: Uuid) -> Result<()>;

    /// Get unsent reminders
    fn get_pending_reminders(&self) -> Result<Vec<DeadlineReminder>>;
}

/// Query parameters for searching deadlines
#[derive(Debug, Default)]
pub struct DeadlineQuery {
    pub case_id: Option<Uuid>,
    pub deadline_type: Option<DeadlineType>,
    pub status: Option<DeadlineStatus>,
    pub responsible_party: Option<String>,
    pub is_jurisdictional: Option<bool>,
    pub due_date_from: Option<DateTime<Utc>>,
    pub due_date_to: Option<DateTime<Utc>>,
    pub offset: usize,
    pub limit: usize,
}

/// Extended repository with advanced query and compliance features
pub trait DeadlineComplianceRepository: DeadlineRepository {
    /// Search deadlines with filters
    fn search_deadlines(&self, query: DeadlineQuery) -> Result<(Vec<Deadline>, usize)>;

    /// Get compliance statistics
    fn get_compliance_statistics(&self, case_id: Option<Uuid>) -> Result<ComplianceStatistics>;

    /// Find missed jurisdictional deadlines
    fn find_missed_jurisdictional(&self) -> Result<Vec<Deadline>>;

    /// Generate compliance report
    fn generate_compliance_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<ComplianceReport>;

    /// Calculate deadline performance metrics
    fn get_performance_metrics(&self, party_name: Option<String>) -> Result<PerformanceMetrics>;
}

/// Compliance statistics
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ComplianceStatistics {
    pub total_deadlines: usize,
    pub completed_on_time: usize,
    pub completed_late: usize,
    pub pending: usize,
    pub overdue: usize,
    pub extended: usize,
    pub waived: usize,
    pub compliance_rate: f32,
    pub average_days_early: f32,
}

/// Compliance report for a period
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ComplianceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_cases: usize,
    pub deadlines_tracked: usize,
    pub compliance_rate: f32,
    pub jurisdictional_violations: usize,
    pub most_missed_type: String,
    pub parties_with_violations: Vec<String>,
}

/// Performance metrics for deadline compliance
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PerformanceMetrics {
    pub entity: String, // Party or "Overall"
    pub total_deadlines: usize,
    pub on_time_percentage: f32,
    pub average_response_days: f32,
    pub extension_requests: usize,
    pub violations: usize,
    pub trending: String, // "improving", "declining", "stable"
}