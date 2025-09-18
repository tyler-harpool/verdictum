//! Repository port for docket and calendar persistence
//!
//! This trait defines the contract for storing and retrieving docket entries,
//! calendar events, and related data in the federal court system.

use crate::domain::docket::{DocketEntry, CalendarEntry, SpeedyTrialClock, DocketEntryType, CalendarEventType, EventStatus};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Repository trait for docket entry persistence
pub trait DocketRepository {
    /// Save a docket entry
    fn save_entry(&self, entry: &DocketEntry) -> Result<()>;

    /// Find docket entry by ID
    fn find_entry_by_id(&self, id: Uuid) -> Result<Option<DocketEntry>>;

    /// Find all entries for a case
    fn find_entries_by_case(&self, case_id: Uuid) -> Result<Vec<DocketEntry>>;

    /// Find entries by type
    fn find_entries_by_type(&self, case_id: Uuid, entry_type: DocketEntryType) -> Result<Vec<DocketEntry>>;

    /// Get next entry number for a case
    fn get_next_entry_number(&self, case_id: Uuid) -> Result<u32>;

    /// Find sealed entries
    fn find_sealed_entries(&self, case_id: Uuid) -> Result<Vec<DocketEntry>>;

    /// Search entries by text
    fn search_entries(&self, case_id: Uuid, search_text: &str) -> Result<Vec<DocketEntry>>;

    /// Delete an entry (for administrative purposes)
    fn delete_entry(&self, id: Uuid) -> Result<bool>;
}

/// Repository trait for calendar persistence
pub trait CalendarRepository {
    /// Save a calendar entry
    fn save_event(&self, event: &CalendarEntry) -> Result<()>;

    /// Find event by ID
    fn find_event_by_id(&self, id: Uuid) -> Result<Option<CalendarEntry>>;

    /// Find events by case
    fn find_events_by_case(&self, case_id: Uuid) -> Result<Vec<CalendarEntry>>;

    /// Find events by judge
    fn find_events_by_judge(&self, judge_id: Uuid) -> Result<Vec<CalendarEntry>>;

    /// Find events by courtroom
    fn find_events_by_courtroom(&self, courtroom: &str, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<CalendarEntry>>;

    /// Find events in date range
    fn find_events_in_range(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<CalendarEntry>>;

    /// Find conflicting events
    fn find_conflicts(&self, judge_id: Uuid, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<CalendarEntry>>;

    /// Update event status
    fn update_event_status(&self, id: Uuid, status: EventStatus) -> Result<()>;

    /// Delete an event
    fn delete_event(&self, id: Uuid) -> Result<bool>;
}

/// Repository trait for Speedy Trial tracking
pub trait SpeedyTrialRepository {
    /// Save Speedy Trial clock
    fn save_clock(&self, clock: &SpeedyTrialClock) -> Result<()>;

    /// Find clock by case ID
    fn find_clock_by_case(&self, case_id: Uuid) -> Result<Option<SpeedyTrialClock>>;

    /// Find cases approaching deadline
    fn find_approaching_deadlines(&self, days_threshold: i64) -> Result<Vec<SpeedyTrialClock>>;

    /// Find violated deadlines
    fn find_violations(&self) -> Result<Vec<SpeedyTrialClock>>;

    /// Update clock status
    fn update_clock(&self, case_id: Uuid, clock: &SpeedyTrialClock) -> Result<()>;
}

/// Query parameters for searching docket entries
#[derive(Debug, Default)]
pub struct DocketQuery {
    pub case_id: Option<Uuid>,
    pub entry_type: Option<DocketEntryType>,
    pub filed_by: Option<String>,
    pub sealed_only: bool,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub search_text: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

/// Query parameters for calendar searches
#[derive(Debug, Default)]
pub struct CalendarQuery {
    pub judge_id: Option<Uuid>,
    pub courtroom: Option<String>,
    pub event_type: Option<CalendarEventType>,
    pub status: Option<EventStatus>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub offset: usize,
    pub limit: usize,
}

/// Extended repository with advanced query capabilities
pub trait DocketQueryRepository: DocketRepository {
    /// Search docket entries with filters
    fn search_docket(&self, query: DocketQuery) -> Result<(Vec<DocketEntry>, usize)>;

    /// Get filing statistics
    fn get_filing_statistics(&self, case_id: Uuid) -> Result<FilingStatistics>;

    /// Generate docket sheet
    fn generate_docket_sheet(&self, case_id: Uuid) -> Result<String>;
}

/// Extended calendar repository with scheduling features
pub trait CalendarSchedulingRepository: CalendarRepository {
    /// Search calendar with filters
    fn search_calendar(&self, query: CalendarQuery) -> Result<(Vec<CalendarEntry>, usize)>;

    /// Find next available slot
    fn find_available_slot(&self, judge_id: Uuid, duration_minutes: u32, earliest: DateTime<Utc>) -> Result<DateTime<Utc>>;

    /// Get judge's schedule
    fn get_judge_schedule(&self, judge_id: Uuid, date: DateTime<Utc>) -> Result<Vec<CalendarEntry>>;

    /// Get courtroom utilization
    fn get_courtroom_utilization(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<CourtroomUtilization>;
}

/// Filing statistics for a case
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FilingStatistics {
    pub total_entries: usize,
    pub motions_filed: usize,
    pub orders_entered: usize,
    pub sealed_entries: usize,
    pub days_since_filing: i64,
    pub last_activity: DateTime<Utc>,
    pub most_active_filer: String,
}

/// Courtroom utilization statistics
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CourtroomUtilization {
    pub total_courtrooms: usize,
    pub total_events: usize,
    pub average_utilization_percent: f32,
    pub busiest_courtroom: String,
    pub peak_hours: Vec<u32>,
}