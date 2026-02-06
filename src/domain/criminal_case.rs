//! Criminal case domain model
//!
//! This module defines the core business logic for criminal case management
//! in the federal court system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use super::common::MotionStatus;
use super::defendant::{CreateDefendantRequest, Defendant, PleaType};
use super::docket::{DocketEntry, DocketEntryType, DocketService, SpeedyTrialClock, ExcludableDelay};

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
    /// Federal case number (e.g., "SDNY:26-CR-00123-JMS")
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
    /// Assigned judge ID (references judge entity)
    pub assigned_judge_id: Option<Uuid>,
    /// Federal district code (e.g., "SDNY", "CDCA")
    pub district_code: String,
    /// Location where the crime occurred
    pub location: String,
    /// When the case was opened
    pub opened_at: DateTime<Utc>,
    /// When the case was last updated
    pub updated_at: DateTime<Utc>,
    /// When the case was closed (if applicable)
    pub closed_at: Option<DateTime<Utc>>,
    /// Defendants in this case
    pub defendants: Vec<Defendant>,
    /// Evidence items
    #[serde(deserialize_with = "deserialize_evidence")]
    pub evidence: Vec<Evidence>,
    /// Court events scheduled
    pub court_events: Vec<CourtEvent>,
    /// Motions filed in the case
    pub motions: Vec<Motion>,
    /// Case notes
    pub notes: Vec<CaseNote>,
    /// Docket entries filed in the case
    #[serde(default)]
    pub docket_entries: Vec<DocketEntry>,
    /// Whether this case is sealed
    #[serde(default)]
    pub is_sealed: bool,
    /// Date the case was sealed
    #[serde(default)]
    pub sealed_date: Option<DateTime<Utc>>,
    /// Who sealed the case
    #[serde(default)]
    pub sealed_by: Option<String>,
    /// Reason for sealing
    #[serde(default)]
    pub seal_reason: Option<String>,
    /// Speedy Trial Act clock
    #[serde(default)]
    pub speedy_trial: Option<SpeedyTrialClock>,
    /// CVRA victims associated with this case
    #[serde(default)]
    pub victims: Vec<super::victim::Victim>,
}

/// A note added to a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CaseNote {
    pub id: Uuid,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
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

/// Type of evidence
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Physical,
    Digital,
    Documentary,
    Testimonial,
    Forensic,
    PhotoVideo,
    Audio,
    Financial,
    Other(String),
}

/// Condition of evidence during custody transfer
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCondition {
    Excellent,
    Good,
    Fair,
    Poor,
    Damaged,
    Tampered,
    Unknown,
}

/// A transfer in the evidence chain of custody
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustodyTransfer {
    pub id: Uuid,
    pub transferred_from: String,
    pub transferred_to: String,
    pub date: DateTime<Utc>,
    pub location: String,
    pub condition: EvidenceCondition,
    pub notes: Option<String>,
}

/// An evidence item with chain of custody tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Evidence {
    pub id: Uuid,
    pub description: String,
    pub evidence_type: EvidenceType,
    pub seized_date: Option<DateTime<Utc>>,
    pub seized_by: Option<String>,
    pub location: String,
    pub chain_of_custody: Vec<CustodyTransfer>,
    pub is_sealed: bool,
    pub created_at: DateTime<Utc>,
}

/// Custom deserializer that handles both legacy Vec<String> and new Vec<Evidence>
fn deserialize_evidence<'de, D>(deserializer: D) -> Result<Vec<Evidence>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum EvidenceItem {
        Legacy(String),
        Full(Evidence),
    }

    let items: Vec<EvidenceItem> = Vec::deserialize(deserializer)?;

    Ok(items.into_iter().map(|item| match item {
        EvidenceItem::Legacy(description) => Evidence {
            id: Uuid::new_v4(),
            description,
            evidence_type: EvidenceType::Other("legacy".to_string()),
            seized_date: None,
            seized_by: None,
            location: String::new(),
            chain_of_custody: Vec::new(),
            is_sealed: false,
            created_at: Utc::now(),
        },
        EvidenceItem::Full(evidence) => evidence,
    }).collect())
}

/// A motion filed in the case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Motion {
    pub id: Uuid,
    pub motion_type: MotionType,
    pub filed_by: String,
    pub description: String,
    pub filed_date: DateTime<Utc>,
    pub status: MotionStatus,
    pub ruling_date: Option<DateTime<Utc>>,
}

impl CriminalCase {
    /// Create a new criminal case
    pub fn new(
        title: String,
        description: String,
        crime_type: CrimeType,
        district_code: String,
        assigned_judge_id: Option<Uuid>,
        judge_initials: &str,
        location: String,
    ) -> Self {
        let now = Utc::now();
        let case_number = Self::generate_case_number(&district_code, "CR", judge_initials);

        Self {
            id: Uuid::new_v4(),
            case_number,
            title,
            description,
            crime_type,
            status: CaseStatus::Filed,
            priority: CasePriority::Medium,
            assigned_judge_id,
            district_code,
            location,
            opened_at: now,
            updated_at: now,
            closed_at: None,
            defendants: Vec::new(),
            evidence: Vec::new(),
            court_events: Vec::new(),
            motions: Vec::new(),
            notes: Vec::new(),
            docket_entries: Vec::new(),
            is_sealed: false,
            sealed_date: None,
            sealed_by: None,
            seal_reason: None,
            speedy_trial: None,
            victims: Vec::new(),
        }
    }

    /// Generate a federal case number
    ///
    /// Format: `{district}:{yy}-{type}-{seq}-{judge_initials}`
    /// Example: `SDNY:26-CR-00123-JMS`
    fn generate_case_number(district_code: &str, case_type: &str, judge_initials: &str) -> String {
        let year = Utc::now().format("%y");
        let seq: u32 = rand::random::<u32>() % 100000;
        format!("{}:{}-{}-{:05}-{}", district_code, year, case_type, seq, judge_initials)
    }

    /// Add a defendant to the case from a request
    pub fn add_defendant(&mut self, request: CreateDefendantRequest) -> Uuid {
        let defendant = Defendant::from_request(self.id, request);
        let id = defendant.id;
        self.defendants.push(defendant);
        self.updated_at = Utc::now();
        id
    }

    /// Find a defendant by ID
    pub fn find_defendant(&self, defendant_id: Uuid) -> Option<&Defendant> {
        self.defendants.iter().find(|d| d.id == defendant_id)
    }

    /// Enter a plea for a specific count of a specific defendant
    pub fn enter_plea(
        &mut self,
        defendant_id: Uuid,
        count_number: u32,
        plea: PleaType,
    ) -> Result<(), String> {
        match self.defendants.iter_mut().find(|d| d.id == defendant_id) {
            Some(defendant) => {
                defendant.enter_plea(count_number, plea)?;
                self.updated_at = Utc::now();
                Ok(())
            }
            None => Err(format!("Defendant {} not found in case {}", defendant_id, self.id)),
        }
    }

    /// Add evidence to the case
    pub fn add_evidence(&mut self, description: String, evidence_type: Option<EvidenceType>) -> Uuid {
        let evidence = Evidence {
            id: Uuid::new_v4(),
            description,
            evidence_type: evidence_type.unwrap_or(EvidenceType::Other("unspecified".to_string())),
            seized_date: None,
            seized_by: None,
            location: String::new(),
            chain_of_custody: Vec::new(),
            is_sealed: false,
            created_at: Utc::now(),
        };
        let id = evidence.id;
        self.evidence.push(evidence);
        self.updated_at = Utc::now();
        id
    }

    /// Add a custody transfer to an evidence item
    pub fn add_custody_transfer(
        &mut self,
        evidence_id: Uuid,
        transferred_from: String,
        transferred_to: String,
        location: String,
        condition: EvidenceCondition,
        notes: Option<String>,
    ) -> Result<Uuid, String> {
        let evidence = self.evidence.iter_mut()
            .find(|e| e.id == evidence_id)
            .ok_or_else(|| format!("Evidence {} not found", evidence_id))?;

        let transfer = CustodyTransfer {
            id: Uuid::new_v4(),
            transferred_from,
            transferred_to,
            date: Utc::now(),
            location,
            condition,
            notes,
        };
        let id = transfer.id;
        evidence.chain_of_custody.push(transfer);
        self.updated_at = Utc::now();
        Ok(id)
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
            status: MotionStatus::Pending,
            ruling_date: None,
        };
        self.motions.push(motion);
        self.updated_at = Utc::now();
    }

    /// Rule on a motion
    pub fn rule_on_motion(&mut self, motion_id: Uuid, ruling: MotionStatus) {
        if let Some(motion) = self.motions.iter_mut().find(|m| m.id == motion_id) {
            motion.status = ruling;
            motion.ruling_date = Some(Utc::now());
            self.updated_at = Utc::now();
        }
    }

    // ========================================================================
    // Phase 1: Docket Entry Methods
    // ========================================================================

    /// Add a docket entry to the case
    pub fn add_docket_entry(
        &mut self,
        entry_type: DocketEntryType,
        description: String,
        filed_by: Option<String>,
    ) -> Uuid {
        let next_number = self.docket_entries.iter()
            .map(|e| e.entry_number)
            .max()
            .unwrap_or(0) + 1;

        let mut entry = DocketService::create_entry(self.id, entry_type, description, filed_by);
        entry.entry_number = next_number;

        let id = entry.id;
        self.docket_entries.push(entry);
        self.updated_at = Utc::now();
        id
    }

    /// Get docket entries sorted by entry number
    pub fn get_docket_entries(&self) -> Vec<&DocketEntry> {
        let mut entries: Vec<&DocketEntry> = self.docket_entries.iter().collect();
        entries.sort_by_key(|e| e.entry_number);
        entries
    }

    // ========================================================================
    // Phase 3: Sealed Case Methods
    // ========================================================================

    /// Seal the case
    pub fn seal(&mut self, reason: String, sealed_by: String) -> Result<(), String> {
        if self.is_sealed {
            return Err("Case is already sealed".to_string());
        }
        self.is_sealed = true;
        self.sealed_date = Some(Utc::now());
        self.sealed_by = Some(sealed_by);
        self.seal_reason = Some(reason);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Unseal the case
    pub fn unseal(&mut self, reason: String, unsealed_by: String, court_order_id: Option<String>) -> Result<(), String> {
        if !self.is_sealed {
            return Err("Case is not sealed".to_string());
        }
        let note_content = format!(
            "Case unsealed by {}. Reason: {}.{}",
            unsealed_by,
            reason,
            court_order_id.map(|id| format!(" Court Order: {}", id)).unwrap_or_default()
        );
        self.is_sealed = false;
        self.sealed_date = None;
        self.sealed_by = None;
        self.seal_reason = None;
        self.add_note(note_content, unsealed_by);
        Ok(())
    }

    // ========================================================================
    // Phase 4: Speedy Trial Clock Methods
    // ========================================================================

    /// Start the Speedy Trial Act clock (70-day clock per § 3161)
    pub fn start_speedy_trial(
        &mut self,
        arrest_date: Option<DateTime<Utc>>,
        indictment_date: Option<DateTime<Utc>>,
        arraignment_date: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        if self.speedy_trial.is_some() {
            return Err("Speedy trial clock already initialized".to_string());
        }

        let now = Utc::now();
        let arraignment = arraignment_date.unwrap_or(now);
        let indictment = indictment_date.unwrap_or(now);

        let deadline = super::docket::SpeedyTrialService::calculate_deadline(indictment, arraignment);
        let clock = SpeedyTrialClock {
            case_id: self.id,
            arrest_date,
            indictment_date: Some(indictment),
            arraignment_date: Some(arraignment),
            trial_start_deadline: deadline,
            excludable_delays: Vec::new(),
            days_elapsed: 0,
            days_remaining: 70,
            is_tolled: false,
            waived: false,
        };

        self.speedy_trial = Some(clock);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Add an excludable delay to the speedy trial clock
    pub fn add_excludable_delay(&mut self, delay: ExcludableDelay) -> Result<(), String> {
        match self.speedy_trial.as_mut() {
            Some(clock) => {
                clock.excludable_delays.push(delay.clone());
                let days_remaining = super::docket::SpeedyTrialService::calculate_days_remaining(clock, Utc::now());
                clock.days_remaining = days_remaining;
                clock.days_elapsed = 70 - days_remaining;
                self.updated_at = Utc::now();
                Ok(())
            }
            None => Err("Speedy trial clock not initialized".to_string()),
        }
    }

    // ========================================================================
    // Phase 5: CVRA Victim Methods
    // ========================================================================

    /// Add a victim to the case
    pub fn add_victim(&mut self, request: super::victim::CreateVictimRequest) -> Uuid {
        let now = Utc::now();
        let victim = super::victim::Victim {
            id: Uuid::new_v4(),
            case_id: self.id,
            name: request.name,
            victim_type: request.victim_type,
            notification_preferences: super::victim::NotificationPreferences {
                preferred_method: request.preferred_method,
                email: request.email,
                phone: request.phone,
                mailing_address: request.mailing_address,
                victim_advocate: request.victim_advocate,
                opt_out: false,
            },
            notifications: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        let id = victim.id;
        self.victims.push(victim);
        self.updated_at = Utc::now();
        id
    }

    /// Send a notification to a victim
    pub fn send_victim_notification(
        &mut self,
        victim_id: Uuid,
        request: super::victim::SendNotificationRequest,
    ) -> Result<Uuid, String> {
        let victim = self.victims.iter_mut()
            .find(|v| v.id == victim_id)
            .ok_or_else(|| format!("Victim {} not found", victim_id))?;

        if victim.notification_preferences.opt_out {
            return Err("Victim has opted out of notifications".to_string());
        }

        let notification = super::victim::VictimNotification {
            id: Uuid::new_v4(),
            notification_type: request.notification_type,
            sent_at: Utc::now(),
            method: victim.notification_preferences.preferred_method.clone(),
            content_summary: request.content_summary,
            acknowledged: false,
            acknowledged_at: None,
        };
        let id = notification.id;
        victim.notifications.push(notification);
        victim.updated_at = Utc::now();
        self.updated_at = Utc::now();
        Ok(id)
    }
}
