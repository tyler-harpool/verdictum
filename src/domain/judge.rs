//! Judge domain model for federal court system
//!
//! This module handles judge assignments, recusals, and administrative functions
//! following the hexagonal architecture pattern.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

/// Federal judge entity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Judge {
    pub id: Uuid,
    pub name: String,
    pub title: JudgeTitle,
    pub district: String,
    pub appointed_date: DateTime<Utc>,
    pub status: JudgeStatus,
    pub senior_status_date: Option<DateTime<Utc>>,
    pub courtroom: String,
    pub current_caseload: u32,
    pub max_caseload: u32,
    pub specializations: Vec<CaseSpecialization>,
    pub conflicts_of_interest: Vec<ConflictOfInterest>,
    pub availability: JudgeAvailability,
}

/// Judge titles in federal court
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JudgeTitle {
    ChiefJudge,
    DistrictJudge,
    SeniorJudge,
    MagistrateJudge,
    BankruptcyJudge,
    VisitingJudge,
}

/// Judge status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JudgeStatus {
    Active,
    Senior,
    Visiting,
    OnLeave,
    Retired,
    Recused,
}

/// Case specialization areas
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseSpecialization {
    CriminalLaw,
    CivilRights,
    IntellectualProperty,
    Bankruptcy,
    Immigration,
    TaxLaw,
    SecuritiesFraud,
    Antitrust,
    EnvironmentalLaw,
    ComplexLitigation,
}

/// Conflict of interest tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictOfInterest {
    pub id: Uuid,
    pub party_name: Option<String>,
    pub law_firm: Option<String>,
    pub corporation: Option<String>,
    pub conflict_type: JudgeConflictType,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JudgeConflictType {
    FinancialInterest,
    PriorRepresentation,
    FamilyRelationship,
    PreviousEmployment,
    StockOwnership,
    Other,
}

/// Judge availability for case assignments
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JudgeAvailability {
    pub accepts_criminal_cases: bool,
    pub accepts_civil_cases: bool,
    pub accepts_emergency_matters: bool,
    pub vacation_dates: Vec<DateRange>,
    pub blocked_dates: Vec<DateTime<Utc>>,
    pub preferred_hearing_days: Vec<Weekday>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

/// Case assignment record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CaseAssignment {
    pub id: Uuid,
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub assignment_type: AssignmentType,
    pub assigned_date: DateTime<Utc>,
    pub reason: String,
    pub previous_judge_id: Option<Uuid>,
    pub reassignment_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentType {
    Random,
    Direct,
    Related,
    Reassignment,
    Visiting,
    Emergency,
}

/// Recusal motion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecusalMotion {
    pub id: Uuid,
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub filed_by: String,
    pub filed_date: DateTime<Utc>,
    pub reason: RecusalReason,
    pub detailed_grounds: String,
    pub status: RecusalStatus,
    pub ruling_date: Option<DateTime<Utc>>,
    pub replacement_judge_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecusalReason {
    ConflictOfInterest,
    PersonalBias,
    PriorInvolvement,
    FinancialInterest,
    ExParte,
    AppearanceOfImpropriety,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecusalStatus {
    Pending,
    Granted,
    Denied,
    Withdrawn,
}

impl Judge {
    /// Create a new judge
    pub fn new(name: String, title: JudgeTitle, district: String, courtroom: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            title,
            district,
            courtroom,
            appointed_date: Utc::now(),
            status: JudgeStatus::Active,
            senior_status_date: None,
            current_caseload: 0,
            max_caseload: match title {
                JudgeTitle::MagistrateJudge => 150,
                JudgeTitle::BankruptcyJudge => 200,
                _ => 100,
            },
            specializations: Vec::new(),
            conflicts_of_interest: Vec::new(),
            availability: JudgeAvailability {
                accepts_criminal_cases: true,
                accepts_civil_cases: true,
                accepts_emergency_matters: true,
                vacation_dates: Vec::new(),
                blocked_dates: Vec::new(),
                preferred_hearing_days: vec![
                    Weekday::Monday,
                    Weekday::Tuesday,
                    Weekday::Wednesday,
                    Weekday::Thursday,
                    Weekday::Friday,
                ],
            },
        }
    }

    /// Check if judge can accept new cases
    pub fn can_accept_new_cases(&self) -> bool {
        self.status == JudgeStatus::Active &&
        self.current_caseload < self.max_caseload
    }

    /// Check if judge is available on a specific date
    pub fn is_available_on(&self, date: &DateTime<Utc>) -> bool {
        // Check blocked dates
        if self.availability.blocked_dates.contains(date) {
            return false;
        }

        // Check vacation dates
        for range in &self.availability.vacation_dates {
            if date >= &range.start && date <= &range.end {
                return false;
            }
        }

        true
    }

    /// Add a conflict of interest
    pub fn add_conflict(&mut self, conflict: ConflictOfInterest) {
        self.conflicts_of_interest.push(conflict);
    }

    /// Check for conflicts with a specific party
    pub fn has_conflict_with(&self, party_name: &str) -> bool {
        self.conflicts_of_interest.iter().any(|c| {
            c.party_name.as_ref().map_or(false, |name| {
                name.to_lowercase().contains(&party_name.to_lowercase())
            }) && c.end_date.is_none()
        })
    }

    /// Update judge status
    pub fn update_status(&mut self, status: JudgeStatus) {
        self.status = status;
        if matches!(self.status, JudgeStatus::Senior) {
            self.senior_status_date = Some(Utc::now());
        }
    }

    /// Increment caseload
    pub fn assign_case(&mut self) -> Result<(), String> {
        if !self.can_accept_new_cases() {
            return Err("Judge cannot accept new cases".to_string());
        }
        self.current_caseload += 1;
        Ok(())
    }

    /// Decrement caseload
    pub fn unassign_case(&mut self) {
        if self.current_caseload > 0 {
            self.current_caseload -= 1;
        }
    }
}

/// Service for random judge assignment
pub struct JudgeAssignmentService;

impl JudgeAssignmentService {
    /// Randomly assign a judge to a case, considering conflicts and availability
    pub fn assign_judge(
        available_judges: &[Judge],
        case_type: CaseType,
        parties: &[String],
        preferred_date: Option<DateTime<Utc>>,
    ) -> Result<Uuid, String> {
        let mut eligible_judges: Vec<&Judge> = available_judges
            .iter()
            .filter(|j| j.can_accept_new_cases())
            .filter(|j| match case_type {
                CaseType::Criminal => j.availability.accepts_criminal_cases,
                CaseType::Civil => j.availability.accepts_civil_cases,
            })
            .filter(|j| {
                // Check for conflicts with any party
                !parties.iter().any(|party| j.has_conflict_with(party))
            })
            .filter(|j| {
                // Check availability on preferred date
                preferred_date.map_or(true, |date| j.is_available_on(&date))
            })
            .collect();

        if eligible_judges.is_empty() {
            return Err("No eligible judges available".to_string());
        }

        // Sort by caseload (ascending) to balance workload
        eligible_judges.sort_by_key(|j| j.current_caseload);

        // Select judge with lowest caseload (with some randomization for ties)
        let min_caseload = eligible_judges[0].current_caseload;
        let candidates: Vec<&Judge> = eligible_judges
            .into_iter()
            .filter(|j| j.current_caseload == min_caseload)
            .collect();

        // Randomly select from candidates with minimum caseload
        let selected_index = (rand::random::<usize>()) % candidates.len();
        Ok(candidates[selected_index].id)
    }

    /// Create a case assignment record
    pub fn create_assignment(
        case_id: Uuid,
        judge_id: Uuid,
        assignment_type: AssignmentType,
        reason: String,
    ) -> CaseAssignment {
        CaseAssignment {
            id: Uuid::new_v4(),
            case_id,
            judge_id,
            assignment_type,
            assigned_date: Utc::now(),
            reason,
            previous_judge_id: None,
            reassignment_reason: None,
        }
    }

    /// Process a recusal motion
    pub fn process_recusal(
        motion: &RecusalMotion,
        available_judges: &[Judge],
        parties: &[String],
    ) -> Result<Uuid, String> {
        // Find a replacement judge
        let replacement_judges: Vec<&Judge> = available_judges
            .iter()
            .filter(|j| j.id != motion.judge_id)
            .filter(|j| j.can_accept_new_cases())
            .filter(|j| !parties.iter().any(|party| j.has_conflict_with(party)))
            .collect();

        if replacement_judges.is_empty() {
            return Err("No replacement judge available".to_string());
        }

        // Select replacement with lowest caseload
        let replacement = replacement_judges
            .iter()
            .min_by_key(|j| j.current_caseload)
            .unwrap();

        Ok(replacement.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseType {
    Criminal,
    Civil,
}