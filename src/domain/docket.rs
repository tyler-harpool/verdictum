//! Docket and calendar management for federal court system
//!
//! This module handles docket entries, court calendar, and scheduling
//! following Lexodus conventions.

use chrono::{DateTime, Duration, Utc, Weekday, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

/// Docket entry in a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocketEntry {
    pub id: Uuid,
    pub case_id: Uuid,
    pub entry_number: u32,
    pub date_filed: DateTime<Utc>,
    pub date_entered: DateTime<Utc>,
    pub filed_by: Option<String>,
    pub entry_type: DocketEntryType,
    pub description: String,
    pub document_id: Option<Uuid>,
    pub is_sealed: bool,
    pub is_ex_parte: bool,
    pub page_count: Option<u32>,
    pub attachments: Vec<DocketAttachment>,
    pub related_entries: Vec<u32>,
    pub service_list: Vec<String>,
}

/// Types of docket entries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocketEntryType {
    // Initial Filings
    Complaint,
    Indictment,
    Information,
    CriminalComplaint,

    // Pleadings
    Answer,
    Motion,
    Response,
    Reply,
    Notice,

    // Orders
    Order,
    MinuteOrder,
    SchedulingOrder,
    ProtectiveOrder,
    SealingOrder,

    // Discovery
    DiscoveryRequest,
    DiscoveryResponse,
    Deposition,
    Interrogatories,

    // Evidence
    Exhibit,
    WitnessList,
    ExpertReport,

    // Hearings
    HearingNotice,
    HearingMinutes,
    Transcript,

    // Judgments
    Judgment,
    Verdict,
    Sentence,

    // Administrative
    Summons,
    Subpoena,
    ServiceReturn,
    Appearance,
    Withdrawal,
    Substitution,

    // Appeals
    NoticeOfAppeal,
    AppealBrief,
    AppellateOrder,

    // Other
    Letter,
    Status,
    Other,
}

/// Attachment to a docket entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocketAttachment {
    pub id: Uuid,
    pub attachment_number: u32,
    pub description: String,
    pub page_count: u32,
    pub file_size_bytes: u64,
}

/// Court calendar entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CalendarEntry {
    pub id: Uuid,
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub event_type: CalendarEventType,
    pub scheduled_date: DateTime<Utc>,
    pub duration_minutes: u32,
    pub courtroom: String,
    pub description: String,
    pub participants: Vec<String>,
    pub court_reporter: Option<String>,
    pub is_public: bool,
    pub call_time: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub status: EventStatus,
    pub notes: String,
}

/// Types of calendar events
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CalendarEventType {
    // Criminal
    InitialAppearance,
    Arraignment,
    BailHearing,
    PleaHearing,
    TrialDate,
    Sentencing,
    ViolationHearing,

    // Civil & Criminal
    StatusConference,
    SchedulingConference,
    SettlementConference,
    PretrialConference,
    MotionHearing,
    EvidentiaryHearing,

    // Trial Events
    JurySelection,
    JuryTrial,
    BenchTrial,

    // Other
    ShowCauseHearing,
    ContemptHearing,
    EmergencyHearing,
    Telephonic,
    VideoConference,
}

/// Event status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Scheduled,
    Confirmed,
    InProgress,
    Completed,
    Cancelled,
    Postponed,
    Recessed,
    Continued,
}

/// Speedy Trial Act tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpeedyTrialClock {
    pub case_id: Uuid,
    pub arrest_date: Option<DateTime<Utc>>,
    pub indictment_date: Option<DateTime<Utc>>,
    pub arraignment_date: Option<DateTime<Utc>>,
    pub trial_start_deadline: DateTime<Utc>,
    pub excludable_delays: Vec<ExcludableDelay>,
    pub days_elapsed: i64,
    pub days_remaining: i64,
    pub is_tolled: bool,
    pub waived: bool,
}

/// Excludable delay periods under Speedy Trial Act
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExcludableDelay {
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub reason: DelayReason,
    pub statutory_reference: String,
    pub days_excluded: i64,
    pub order_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelayReason {
    CompetencyProceedings,
    InterlocutoryAppeal,
    PretrialMotions,
    CoDefendantProceedings,
    DefendantUnavailable,
    ContinuanceInInterestOfJustice,
    DeferredProsecution,
    Other,
}

/// Service for managing docket entries
pub struct DocketService;

impl DocketService {
    /// Create a new docket entry
    pub fn create_entry(
        case_id: Uuid,
        entry_type: DocketEntryType,
        description: String,
        filed_by: Option<String>,
    ) -> DocketEntry {
        DocketEntry {
            id: Uuid::new_v4(),
            case_id,
            entry_number: 0, // Should be set by repository based on sequence
            date_filed: Utc::now(),
            date_entered: Utc::now(),
            filed_by,
            entry_type,
            description,
            document_id: None,
            is_sealed: false,
            is_ex_parte: false,
            page_count: None,
            attachments: Vec::new(),
            related_entries: Vec::new(),
            service_list: Vec::new(),
        }
    }

    /// Generate automatic minute entry for an event
    pub fn generate_minute_entry(event: &CalendarEntry) -> DocketEntry {
        let description = format!(
            "Minute Entry for proceedings held before Judge on {}. {}",
            event.scheduled_date.format("%m/%d/%Y"),
            event.description
        );

        DocketEntry {
            id: Uuid::new_v4(),
            case_id: event.case_id,
            entry_number: 0,
            date_filed: event.actual_end.unwrap_or(Utc::now()),
            date_entered: Utc::now(),
            filed_by: Some("Court".to_string()),
            entry_type: DocketEntryType::MinuteOrder,
            description,
            document_id: None,
            is_sealed: !event.is_public,
            is_ex_parte: false,
            page_count: None,
            attachments: Vec::new(),
            related_entries: Vec::new(),
            service_list: event.participants.clone(),
        }
    }

    /// Check if a document requires immediate service
    pub fn requires_immediate_service(entry_type: &DocketEntryType) -> bool {
        matches!(
            entry_type,
            DocketEntryType::Order |
            DocketEntryType::SchedulingOrder |
            DocketEntryType::Summons |
            DocketEntryType::Subpoena |
            DocketEntryType::HearingNotice
        )
    }
}

/// Service for calendar management
pub struct CalendarService;

impl CalendarService {
    /// Schedule a court event
    pub fn schedule_event(
        case_id: Uuid,
        judge_id: Uuid,
        event_type: CalendarEventType,
        preferred_date: DateTime<Utc>,
        duration_minutes: u32,
        courtroom: String,
    ) -> CalendarEntry {
        CalendarEntry {
            id: Uuid::new_v4(),
            case_id,
            judge_id,
            event_type,
            scheduled_date: preferred_date,
            duration_minutes,
            courtroom,
            description: String::new(),
            participants: Vec::new(),
            court_reporter: None,
            is_public: true,
            call_time: Some(preferred_date - Duration::minutes(15)),
            actual_start: None,
            actual_end: None,
            status: EventStatus::Scheduled,
            notes: String::new(),
        }
    }

    /// Check for scheduling conflicts
    pub fn check_conflicts(
        existing_events: &[CalendarEntry],
        new_event: &CalendarEntry,
    ) -> Vec<CalendarEntry> {
        let new_start = new_event.scheduled_date;
        let new_end = new_start + Duration::minutes(new_event.duration_minutes as i64);

        existing_events
            .iter()
            .filter(|event| {
                // Same courtroom or same judge
                (event.courtroom == new_event.courtroom || event.judge_id == new_event.judge_id)
                    && event.status != EventStatus::Cancelled
            })
            .filter(|event| {
                let event_start = event.scheduled_date;
                let event_end = event_start + Duration::minutes(event.duration_minutes as i64);

                // Check for time overlap
                (new_start >= event_start && new_start < event_end) ||
                (new_end > event_start && new_end <= event_end) ||
                (new_start <= event_start && new_end >= event_end)
            })
            .cloned()
            .collect()
    }

    /// Calculate next available slot
    pub fn find_next_available_slot(
        existing_events: &[CalendarEntry],
        judge_id: Uuid,
        duration_minutes: u32,
        earliest_date: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let mut candidate_date = earliest_date;

        // Skip weekends
        while candidate_date.weekday() == Weekday::Sat || candidate_date.weekday() == Weekday::Sun {
            candidate_date = candidate_date + Duration::days(1);
        }

        // Set to 9 AM if before business hours
        if candidate_date.hour() < 9 {
            candidate_date = candidate_date.date_naive().and_hms_opt(9, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap();
        }

        // Find available slot
        loop {
            let test_event = CalendarEntry {
                id: Uuid::new_v4(),
                case_id: Uuid::new_v4(),
                judge_id,
                event_type: CalendarEventType::MotionHearing,
                scheduled_date: candidate_date,
                duration_minutes,
                courtroom: String::new(),
                description: String::new(),
                participants: Vec::new(),
                court_reporter: None,
                is_public: true,
                call_time: None,
                actual_start: None,
                actual_end: None,
                status: EventStatus::Scheduled,
                notes: String::new(),
            };

            let conflicts = Self::check_conflicts(existing_events, &test_event);

            if conflicts.is_empty() {
                return candidate_date;
            }

            // Try next hour
            candidate_date = candidate_date + Duration::hours(1);

            // If past 4 PM, move to next day 9 AM
            if candidate_date.hour() >= 16 {
                candidate_date = (candidate_date + Duration::days(1))
                    .date_naive()
                    .and_hms_opt(9, 0, 0)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap();

                // Skip weekends
                while candidate_date.weekday() == Weekday::Sat || candidate_date.weekday() == Weekday::Sun {
                    candidate_date = candidate_date + Duration::days(1);
                }
            }
        }
    }
}

/// Service for Speedy Trial Act compliance
pub struct SpeedyTrialService;

impl SpeedyTrialService {
    /// Calculate Speedy Trial Act deadline
    pub fn calculate_deadline(
        indictment_date: DateTime<Utc>,
        arraignment_date: DateTime<Utc>,
    ) -> DateTime<Utc> {
        // 70 days from indictment/information or arraignment, whichever is later
        let start_date = if arraignment_date > indictment_date {
            arraignment_date
        } else {
            indictment_date
        };

        start_date + Duration::days(70)
    }

    /// Calculate days remaining
    pub fn calculate_days_remaining(
        clock: &SpeedyTrialClock,
        as_of_date: DateTime<Utc>,
    ) -> i64 {
        let total_days_elapsed = (as_of_date - clock.arraignment_date.unwrap_or(as_of_date))
            .num_days();

        let excludable_days: i64 = clock.excludable_delays
            .iter()
            .map(|delay| delay.days_excluded)
            .sum();

        let net_days_elapsed = total_days_elapsed - excludable_days;

        70 - net_days_elapsed
    }

    /// Check if deadline is approaching (within 14 days)
    pub fn is_deadline_approaching(clock: &SpeedyTrialClock) -> bool {
        clock.days_remaining <= 14 && clock.days_remaining > 0 && !clock.waived
    }

    /// Check if deadline has passed
    pub fn is_deadline_violated(clock: &SpeedyTrialClock) -> bool {
        clock.days_remaining < 0 && !clock.waived
    }
}