//! Criminal case domain model
//!
//! This module defines the core business logic for criminal case management,
//! demonstrating how hexagonal architecture can handle a different domain
//! alongside the existing ToDo functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

/// Status of a federal criminal case
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    /// Case filed, awaiting initial appearance
    Filed,
    /// Initial appearance/arraignment completed
    Arraigned,
    /// Discovery phase - exchange of evidence
    Discovery,
    /// Pretrial motions being heard
    PretrialMotions,
    /// Plea negotiations in progress
    PleaNegotiations,
    /// Ready for trial
    TrialReady,
    /// Trial in progress
    InTrial,
    /// Awaiting sentencing
    AwaitingSentencing,
    /// Case closed - sentenced
    Sentenced,
    /// Case dismissed
    Dismissed,
    /// On appeal
    OnAppeal,
}

/// Priority level for a case
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CasePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Type of federal crime charge
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CrimeType {
    /// Wire fraud, mail fraud, securities fraud
    Fraud,
    /// Drug trafficking, possession with intent
    DrugOffense,
    /// Racketeering, RICO violations
    Racketeering,
    /// Computer crimes, identity theft
    Cybercrime,
    /// Tax evasion, tax fraud
    TaxOffense,
    /// Money laundering
    MoneyLaundering,
    /// Immigration violations
    Immigration,
    /// Weapons charges
    Firearms,
    /// Other federal offenses
    Other(String),
}

/// Core criminal case domain entity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CriminalCase {
    /// Unique case identifier
    pub id: Uuid,
    /// Case number (e.g., "2024-001234")
    pub case_number: String,
    /// Case title/description
    pub title: String,
    /// Detailed description of the case
    pub description: String,
    /// Type of crime
    pub crime_type: CrimeType,
    /// Current case status
    pub status: CaseStatus,
    /// Case priority
    pub priority: CasePriority,
    /// Name of the assigned judge
    pub assigned_judge: String,
    /// Location where the crime occurred
    pub location: String,
    /// When the case was opened
    pub opened_at: DateTime<Utc>,
    /// When the case was last updated
    pub updated_at: DateTime<Utc>,
    /// When the case was closed (if applicable)
    pub closed_at: Option<DateTime<Utc>>,
    /// Defendant names (if any)
    pub defendants: Vec<String>,
    /// Plea entered by each defendant
    pub pleas: Vec<(String, PleaType)>, // (defendant_name, plea)
    /// Evidence items
    pub evidence: Vec<String>,
    /// Court events scheduled
    pub court_events: Vec<CourtEvent>,
    /// Motions filed in the case
    pub motions: Vec<Motion>,
    /// Case notes
    pub notes: Vec<CaseNote>,
}

/// A note added to a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CaseNote {
    pub id: Uuid,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

/// Type of plea entered by defendant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PleaType {
    Guilty,
    NotGuilty,
    NoloContendere,
    NotEntered,
}

/// Court event types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    InitialAppearance,
    Arraignment,
    StatusConference,
    PretrialConference,
    MotionHearing,
    TrialDate,
    SentencingHearing,
    Other(String),
}

/// A scheduled court event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CourtEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub scheduled_date: DateTime<Utc>,
    pub description: String,
    pub location: String,
    pub created_at: DateTime<Utc>,
}

/// Motion types in federal court
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MotionType {
    Dismiss,
    SuppressEvidence,
    ChangeOfVenue,
    Discovery,
    Continuance,
    BailModification,
    Other(String),
}

/// A motion filed in the case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Motion {
    pub id: Uuid,
    pub motion_type: MotionType,
    pub filed_by: String,
    pub description: String,
    pub filed_date: DateTime<Utc>,
    pub status: String, // pending, granted, denied
    pub ruling_date: Option<DateTime<Utc>>,
}

impl CriminalCase {
    /// Create a new criminal case
    pub fn new(
        title: String,
        description: String,
        crime_type: CrimeType,
        assigned_judge: String,
        location: String,
    ) -> Self {
        let now = Utc::now();
        let case_number = Self::generate_case_number(&now);

        Self {
            id: Uuid::new_v4(),
            case_number,
            title,
            description,
            crime_type,
            status: CaseStatus::Filed,
            priority: CasePriority::Medium,
            assigned_judge,
            location,
            opened_at: now,
            updated_at: now,
            closed_at: None,
            defendants: Vec::new(),
            pleas: Vec::new(),
            evidence: Vec::new(),
            court_events: Vec::new(),
            motions: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Generate a case number based on current date and a random component
    fn generate_case_number(date: &DateTime<Utc>) -> String {
        let year = date.format("%Y");
        let random: u32 = rand::random::<u32>() % 1000000;
        format!("{}-{:06}", year, random)
    }

    /// Add a defendant to the case
    pub fn add_defendant(&mut self, name: String) {
        if !self.defendants.contains(&name) {
            self.defendants.push(name);
            self.updated_at = Utc::now();
        }
    }

    /// Add evidence to the case
    pub fn add_evidence(&mut self, evidence: String) {
        self.evidence.push(evidence);
        self.updated_at = Utc::now();
    }

    /// Add a note to the case
    pub fn add_note(&mut self, content: String, author: String) {
        let note = CaseNote {
            id: Uuid::new_v4(),
            content,
            author,
            created_at: Utc::now(),
        };
        self.notes.push(note);
        self.updated_at = Utc::now();
    }

    /// Update case status
    pub fn update_status(&mut self, status: CaseStatus) {
        self.status = status.clone();
        self.updated_at = Utc::now();

        // Mark case as closed for terminal statuses
        if matches!(status, CaseStatus::Sentenced | CaseStatus::Dismissed) {
            self.closed_at = Some(Utc::now());
        }
    }

    /// Update case priority
    pub fn update_priority(&mut self, priority: CasePriority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }

    /// Check if case is active (not closed)
    pub fn is_active(&self) -> bool {
        !matches!(self.status, CaseStatus::Sentenced | CaseStatus::Dismissed)
    }

    /// Enter a plea for a defendant
    pub fn enter_plea(&mut self, defendant_name: String, plea: PleaType) {
        // Remove any existing plea for this defendant
        self.pleas.retain(|(name, _)| name != &defendant_name);
        // Add new plea
        self.pleas.push((defendant_name, plea));
        self.updated_at = Utc::now();
    }

    /// Schedule a court event
    pub fn schedule_event(&mut self, event_type: EventType, scheduled_date: DateTime<Utc>, description: String, location: String) {
        let event = CourtEvent {
            id: Uuid::new_v4(),
            event_type,
            scheduled_date,
            description,
            location,
            created_at: Utc::now(),
        };
        self.court_events.push(event);
        self.updated_at = Utc::now();
    }

    /// File a motion
    pub fn file_motion(&mut self, motion_type: MotionType, filed_by: String, description: String) {
        let motion = Motion {
            id: Uuid::new_v4(),
            motion_type,
            filed_by,
            description,
            filed_date: Utc::now(),
            status: "pending".to_string(),
            ruling_date: None,
        };
        self.motions.push(motion);
        self.updated_at = Utc::now();
    }

    /// Rule on a motion
    pub fn rule_on_motion(&mut self, motion_id: Uuid, ruling: String) {
        if let Some(motion) = self.motions.iter_mut().find(|m| m.id == motion_id) {
            motion.status = ruling;
            motion.ruling_date = Some(Utc::now());
            self.updated_at = Utc::now();
        }
    }
}