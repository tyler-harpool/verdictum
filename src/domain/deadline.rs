//! Deadline tracking and compliance for federal court rules
//!
//! This module handles FRCP/FRCrP deadlines, local rules, and automated compliance checking

use chrono::{DateTime, Duration, Utc, Weekday, NaiveDate, Datelike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

/// Deadline tracking for a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Deadline {
    pub id: Uuid,
    pub case_id: Uuid,
    pub deadline_type: DeadlineType,
    pub due_date: DateTime<Utc>,
    pub triggering_event: String,
    pub triggering_date: DateTime<Utc>,
    pub applicable_rule: String,
    pub description: String,
    pub responsible_party: String,
    pub is_jurisdictional: bool,
    pub is_extendable: bool,
    pub status: DeadlineStatus,
    pub completion_date: Option<DateTime<Utc>>,
    pub extension_requests: Vec<ExtensionRequest>,
    pub reminders_sent: Vec<DateTime<Utc>>,
}

/// Types of deadlines in federal court
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineType {
    // Filing Deadlines
    Answer,
    Reply,
    Motion,
    Response,
    Discovery,

    // Appeal Deadlines
    NoticeOfAppeal,
    AppellateBrief,
    ReplyBrief,
    PetitionForRehearing,

    // Criminal Deadlines
    SpeedyTrial,
    Sentencing,
    PretrialMotions,

    // Discovery Deadlines
    InitialDisclosures,
    ExpertDisclosures,
    DiscoveryCompletion,

    // Trial Deadlines
    WitnessList,
    ExhibitList,
    JuryInstructions,
    PretrialStatement,

    // Administrative
    ServiceOfProcess,
    RemovalNotice,
    StatusReport,

    Other,
}

/// Status of a deadline
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineStatus {
    Pending,
    Approaching, // Within warning period
    Due, // Due today
    Completed,
    Overdue,
    Extended,
    Waived,
    Moot,
}

/// Extension request for a deadline
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExtensionRequest {
    pub id: Uuid,
    pub requested_date: DateTime<Utc>,
    pub requested_by: String,
    pub new_due_date: DateTime<Utc>,
    pub reason: String,
    pub opposed_by: Vec<String>,
    pub status: ExtensionStatus,
    pub ruling_date: Option<DateTime<Utc>>,
    pub order_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionStatus {
    Pending,
    Granted,
    Denied,
    PartiallyGranted,
    Withdrawn,
}

/// Federal Rules of Civil/Criminal Procedure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FederalRule {
    pub rule_number: String,
    pub title: String,
    pub days_to_respond: i64,
    pub is_calendar_days: bool, // vs court days
    pub includes_weekends: bool,
    pub service_adds_days: i64, // Additional days for service method
}

/// Service for calculating deadlines
pub struct DeadlineCalculator;

impl DeadlineCalculator {
    /// Calculate a deadline based on federal rules
    pub fn calculate_deadline(
        triggering_date: DateTime<Utc>,
        days: i64,
        count_weekends: bool,
        add_service_days: i64,
    ) -> DateTime<Utc> {
        let mut deadline = if count_weekends {
            triggering_date + Duration::days(days + add_service_days)
        } else {
            Self::add_court_days(triggering_date, days + add_service_days)
        };

        // If deadline falls on weekend/holiday, move to next court day
        deadline = Self::next_court_day(deadline);

        deadline
    }

    /// Add court days (excluding weekends and holidays)
    fn add_court_days(start: DateTime<Utc>, days: i64) -> DateTime<Utc> {
        let mut current = start;
        let mut days_remaining = days;

        while days_remaining > 0 {
            current = current + Duration::days(1);

            // Skip weekends
            if current.weekday() != Weekday::Sat &&
               current.weekday() != Weekday::Sun &&
               !Self::is_federal_holiday(&current) {
                days_remaining -= 1;
            }
        }

        current
    }

    /// Get next court day if date falls on weekend/holiday
    fn next_court_day(date: DateTime<Utc>) -> DateTime<Utc> {
        let mut result = date;

        while result.weekday() == Weekday::Sat ||
              result.weekday() == Weekday::Sun ||
              Self::is_federal_holiday(&result) {
            result = result + Duration::days(1);
        }

        result
    }

    /// Check if date is a federal holiday
    fn is_federal_holiday(date: &DateTime<Utc>) -> bool {
        let year = date.year();
        let holidays = Self::get_federal_holidays(year);

        holidays.iter().any(|holiday| {
            *holiday == date.date_naive()
        })
    }

    /// Get federal holidays for a year
    fn get_federal_holidays(year: i32) -> Vec<NaiveDate> {
        vec![
            // New Year's Day
            NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            // MLK Day - 3rd Monday in January
            Self::nth_weekday_of_month(year, 1, Weekday::Mon, 3),
            // Presidents Day - 3rd Monday in February
            Self::nth_weekday_of_month(year, 2, Weekday::Mon, 3),
            // Memorial Day - Last Monday in May
            Self::last_weekday_of_month(year, 5, Weekday::Mon),
            // Independence Day
            NaiveDate::from_ymd_opt(year, 7, 4).unwrap(),
            // Labor Day - 1st Monday in September
            Self::nth_weekday_of_month(year, 9, Weekday::Mon, 1),
            // Columbus Day - 2nd Monday in October
            Self::nth_weekday_of_month(year, 10, Weekday::Mon, 2),
            // Veterans Day
            NaiveDate::from_ymd_opt(year, 11, 11).unwrap(),
            // Thanksgiving - 4th Thursday in November
            Self::nth_weekday_of_month(year, 11, Weekday::Thu, 4),
            // Christmas
            NaiveDate::from_ymd_opt(year, 12, 25).unwrap(),
        ]
    }

    fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
        let mut date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let mut count = 0;

        while count < n {
            if date.weekday() == weekday {
                count += 1;
                if count == n {
                    return date;
                }
            }
            date = date.succ_opt().unwrap();
        }

        date
    }

    fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
        let mut date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };

        date = date.pred_opt().unwrap();

        while date.weekday() != weekday {
            date = date.pred_opt().unwrap();
        }

        date
    }

    /// Calculate common FRCP deadlines
    pub fn calculate_frcp_deadlines(triggering_event: &str, triggering_date: DateTime<Utc>) -> Vec<Deadline> {
        let mut deadlines = Vec::new();

        match triggering_event {
            "complaint_filed" => {
                // 21 days to answer (FRCP 12)
                deadlines.push(Deadline {
                    id: Uuid::new_v4(),
                    case_id: Uuid::new_v4(),
                    deadline_type: DeadlineType::Answer,
                    due_date: Self::calculate_deadline(triggering_date, 21, false, 0),
                    triggering_event: triggering_event.to_string(),
                    triggering_date,
                    applicable_rule: "FRCP 12(a)(1)(A)".to_string(),
                    description: "Deadline to file answer to complaint".to_string(),
                    responsible_party: "Defendant".to_string(),
                    is_jurisdictional: false,
                    is_extendable: true,
                    status: DeadlineStatus::Pending,
                    completion_date: None,
                    extension_requests: Vec::new(),
                    reminders_sent: Vec::new(),
                });
            }
            "scheduling_order" => {
                // Various discovery deadlines
                deadlines.push(Deadline {
                    id: Uuid::new_v4(),
                    case_id: Uuid::new_v4(),
                    deadline_type: DeadlineType::InitialDisclosures,
                    due_date: Self::calculate_deadline(triggering_date, 14, false, 0),
                    triggering_event: triggering_event.to_string(),
                    triggering_date,
                    applicable_rule: "FRCP 26(a)(1)".to_string(),
                    description: "Initial disclosures due".to_string(),
                    responsible_party: "All parties".to_string(),
                    is_jurisdictional: false,
                    is_extendable: true,
                    status: DeadlineStatus::Pending,
                    completion_date: None,
                    extension_requests: Vec::new(),
                    reminders_sent: Vec::new(),
                });
            }
            "judgment_entered" => {
                // 30 days to appeal (FRAP 4)
                deadlines.push(Deadline {
                    id: Uuid::new_v4(),
                    case_id: Uuid::new_v4(),
                    deadline_type: DeadlineType::NoticeOfAppeal,
                    due_date: Self::calculate_deadline(triggering_date, 30, false, 0),
                    triggering_event: triggering_event.to_string(),
                    triggering_date,
                    applicable_rule: "FRAP 4(a)(1)(A)".to_string(),
                    description: "Deadline to file notice of appeal".to_string(),
                    responsible_party: "Appellant".to_string(),
                    is_jurisdictional: true,
                    is_extendable: false,
                    status: DeadlineStatus::Pending,
                    completion_date: None,
                    extension_requests: Vec::new(),
                    reminders_sent: Vec::new(),
                });
            }
            _ => {}
        }

        deadlines
    }
}

/// Service for monitoring and alerting on deadlines
pub struct DeadlineMonitor;

impl DeadlineMonitor {
    /// Check all deadlines and update statuses
    pub fn update_deadline_statuses(deadlines: &mut [Deadline], current_date: DateTime<Utc>) {
        for deadline in deadlines {
            if deadline.status == DeadlineStatus::Completed ||
               deadline.status == DeadlineStatus::Waived ||
               deadline.status == DeadlineStatus::Moot {
                continue;
            }

            let days_until = (deadline.due_date - current_date).num_days();

            deadline.status = match days_until {
                d if d < 0 => DeadlineStatus::Overdue,
                0 => DeadlineStatus::Due,
                1..=7 => DeadlineStatus::Approaching,
                _ => DeadlineStatus::Pending,
            };
        }
    }

    /// Get deadlines requiring immediate attention
    pub fn get_urgent_deadlines(deadlines: &[Deadline]) -> Vec<&Deadline> {
        deadlines
            .iter()
            .filter(|d| matches!(
                d.status,
                DeadlineStatus::Overdue | DeadlineStatus::Due | DeadlineStatus::Approaching
            ))
            .collect()
    }

    /// Generate reminder notifications
    pub fn generate_reminders(deadlines: &[Deadline], current_date: DateTime<Utc>) -> Vec<DeadlineReminder> {
        let mut reminders = Vec::new();

        for deadline in deadlines {
            if deadline.status == DeadlineStatus::Completed ||
               deadline.status == DeadlineStatus::Waived {
                continue;
            }

            let days_until = (deadline.due_date - current_date).num_days();

            // Generate reminders at 14, 7, 3, and 1 days before
            let should_remind = match days_until {
                14 | 7 | 3 | 1 => true,
                d if d < 0 => true, // Overdue
                _ => false,
            };

            if should_remind {
                // Check if reminder already sent today
                let already_sent = deadline.reminders_sent
                    .iter()
                    .any(|sent| sent.date_naive() == current_date.date_naive());

                if !already_sent {
                    reminders.push(DeadlineReminder {
                        deadline_id: deadline.id,
                        case_id: deadline.case_id,
                        recipient: deadline.responsible_party.clone(),
                        deadline_type: deadline.deadline_type.clone(),
                        due_date: deadline.due_date,
                        days_until,
                        is_jurisdictional: deadline.is_jurisdictional,
                        message: Self::format_reminder_message(deadline, days_until),
                    });
                }
            }
        }

        reminders
    }

    fn format_reminder_message(deadline: &Deadline, days_until: i64) -> String {
        if days_until < 0 {
            format!(
                "OVERDUE: {} was due on {}. {} days overdue.",
                deadline.description,
                deadline.due_date.format("%m/%d/%Y"),
                -days_until
            )
        } else if days_until == 0 {
            format!(
                "DUE TODAY: {}. {}",
                deadline.description,
                if deadline.is_jurisdictional {
                    "This is a JURISDICTIONAL deadline."
                } else {
                    ""
                }
            )
        } else {
            format!(
                "REMINDER: {} due in {} days ({}). Rule: {}",
                deadline.description,
                days_until,
                deadline.due_date.format("%m/%d/%Y"),
                deadline.applicable_rule
            )
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeadlineReminder {
    pub deadline_id: Uuid,
    pub case_id: Uuid,
    pub recipient: String,
    pub deadline_type: DeadlineType,
    pub due_date: DateTime<Utc>,
    pub days_until: i64,
    pub is_jurisdictional: bool,
    pub message: String,
}