//! Attorney and Party Management for Federal Court System
//!
//! This module handles attorney registration, CJA panel management,
//! pro hac vice admissions, and party representation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Attorney profile and credentials
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Attorney {
    pub id: String,
    pub bar_number: String,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub firm_name: Option<String>,
    pub email: String,
    pub phone: String,
    pub fax: Option<String>,
    pub address: Address,

    // Bar admissions
    pub bar_admissions: Vec<BarAdmission>,
    pub federal_admissions: Vec<FederalAdmission>,
    pub pro_hac_vice_admissions: Vec<ProHacViceAdmission>,

    // ECF privileges
    pub ecf_registration: Option<ECFRegistration>,

    // CJA Panel
    pub cja_panel_member: bool,
    pub cja_panel_districts: Vec<String>,
    pub cja_appointments: Vec<CJAAppointment>,

    // Practice areas
    pub practice_areas: Vec<PracticeArea>,
    pub languages_spoken: Vec<String>,

    // Status
    pub status: AttorneyStatus,
    pub discipline_history: Vec<DisciplinaryAction>,

    // Metrics
    pub cases_handled: i32,
    pub win_rate_percentage: Option<f64>,
    pub avg_case_duration_days: Option<i32>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Attorney status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum AttorneyStatus {
    Active,
    Inactive,
    Suspended,
    Disbarred,
    Retired,
    Deceased,
}

/// Physical/mailing address
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Address {
    pub street1: String,
    pub street2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

/// State bar admission
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BarAdmission {
    pub state: String,
    pub bar_number: String,
    pub admission_date: DateTime<Utc>,
    pub status: AdmissionStatus,
    pub expiration_date: Option<DateTime<Utc>>,
}

/// Federal court admission
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FederalAdmission {
    pub court: String, // e.g., "N.D. Cal.", "9th Cir."
    pub admission_date: DateTime<Utc>,
    pub sponsor_attorney: Option<String>,
    pub status: AdmissionStatus,
}

/// Pro hac vice (temporary) admission
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProHacViceAdmission {
    pub case_id: String,
    pub case_caption: String,
    pub court: String,
    pub admission_date: DateTime<Utc>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub local_counsel: String,
    pub local_counsel_bar_number: String,
    pub status: ProHacViceStatus,
    pub fee_paid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum AdmissionStatus {
    Active,
    Inactive,
    Suspended,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ProHacViceStatus {
    Pending,
    Granted,
    Denied,
    Withdrawn,
    Expired,
}

/// ECF (Electronic Case Filing) registration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ECFRegistration {
    pub login_id: String,
    pub registration_date: DateTime<Utc>,
    pub primary_email: String,
    pub secondary_emails: Vec<String>,
    pub filing_privileges: Vec<FilingPrivilege>,
    pub training_completed: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum FilingPrivilege {
    CivilFiling,
    CriminalFiling,
    BankruptcyFiling,
    AppellateFiling,
    EmergencyFiling,
    SealedFiling,
    ExParteFiling,
}

/// CJA (Criminal Justice Act) appointment
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CJAAppointment {
    pub id: String,
    pub case_id: String,
    pub case_caption: String,
    pub appointment_date: DateTime<Utc>,
    pub appointment_type: CJAAppointmentType,
    pub compensation_status: CompensationStatus,
    pub hours_claimed: f64,
    pub amount_approved: Option<f64>,
    pub voucher_status: VoucherStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CJAAppointmentType {
    TrialLevel,
    Appellate,
    Habeas,
    CapitalCase,
    ComplexLitigation,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CompensationStatus {
    Pending,
    Approved,
    PartiallyApproved,
    Denied,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VoucherStatus {
    NotSubmitted,
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Paid,
}

/// Practice areas
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum PracticeArea {
    CivilLitigation,
    CriminalDefense,
    CorporateLaw,
    IntellectualProperty,
    Immigration,
    FamilyLaw,
    Bankruptcy,
    TaxLaw,
    EmploymentLaw,
    EnvironmentalLaw,
    CivilRights,
    WhiteCollarDefense,
    Appellate,
    Other(String),
}

/// Disciplinary actions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisciplinaryAction {
    pub id: String,
    pub date: DateTime<Utc>,
    pub jurisdiction: String,
    pub action_type: DisciplineType,
    pub description: String,
    pub case_number: Option<String>,
    pub effective_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub public_record: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum DisciplineType {
    Warning,
    Reprimand,
    Probation,
    Suspension,
    Disbarment,
    Reinstatement,
    Other,
}

/// Status of disciplinary action
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum DisciplineStatus {
    Pending,
    Active,
    Completed,
    Appealed,
    Reversed,
    Expired,
}

/// Party in a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Party {
    pub id: String,
    pub case_id: String,
    pub party_type: PartyType,
    pub party_role: PartyRole,
    pub name: String,
    pub entity_type: EntityType,

    // Individual fields
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<DateTime<Utc>>,
    pub ssn_last_four: Option<String>,

    // Organization fields
    pub organization_name: Option<String>,
    pub ein: Option<String>,

    // Contact information
    pub address: Option<Address>,
    pub phone: Option<String>,
    pub email: Option<String>,

    // Representation
    pub represented: bool,
    pub pro_se: bool,
    pub attorneys: Vec<AttorneyRepresentation>,

    // Service information
    pub service_address: Option<Address>,
    pub service_email: Option<String>,
    pub service_method: ServiceMethod,

    // Status
    pub status: PartyStatus,
    pub joined_date: DateTime<Utc>,
    pub terminated_date: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum PartyType {
    Plaintiff,
    Defendant,
    Appellant,
    Appellee,
    Petitioner,
    Respondent,
    Intervenor,
    AmicusCuriae,
    ThirdParty,
    CounterClaimant,
    CrossClaimant,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum PartyRole {
    Principal,
    CoParty,
    Representative,
    Guardian,
    Trustee,
    Executor,
    Administrator,
    NextFriend,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum EntityType {
    Individual,
    Corporation,
    Partnership,
    LLC,
    Government,
    NonProfit,
    Estate,
    Trust,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum PartyStatus {
    Active,
    Terminated,
    Dismissed,
    Settled,
    Defaulted,
    InContempt,
}

/// Status of attorney representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum RepresentationStatus {
    Active,
    Withdrawn,
    Terminated,
    Suspended,
    Completed,
}

/// Representation record linking attorney and party
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Representation {
    pub id: String,
    pub attorney_id: String,
    pub party_id: String,
    pub case_id: String,
    pub status: RepresentationStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub representation_type: RepresentationType,
    pub notes: Option<String>,
}

/// Attorney representation record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttorneyRepresentation {
    pub id: String,
    pub attorney_id: String,
    pub party_id: String,
    pub case_id: String,
    pub representation_type: RepresentationType,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub lead_counsel: bool,
    pub local_counsel: bool,
    pub limited_appearance: bool,
    pub scope_of_representation: Option<String>,
    pub withdrawal_reason: Option<WithdrawalReason>,
    pub court_appointed: bool,
    pub cja_appointment_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum RepresentationType {
    General,
    Limited,
    ProHacVice,
    CJAAppointed,
    ProBono,
    PublicDefender,
    Standby,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum WithdrawalReason {
    ClientRequest,
    ConflictOfInterest,
    NonPayment,
    CompletedRepresentation,
    BreakdownInCommunication,
    HealthReasons,
    CourtOrder,
    Other,
}

/// Service of process record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceRecord {
    pub id: String,
    pub document_id: String,
    pub party_id: String,
    pub service_date: DateTime<Utc>,
    pub service_method: ServiceMethod,
    pub served_by: String,
    pub proof_of_service_filed: bool,
    pub certificate_of_service: Option<String>,
    pub successful: bool,
    pub attempts: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ServiceMethod {
    PersonalService,
    CertifiedMail,
    RegularMail,
    Email,
    ECF,
    Publication,
    Waiver,
    Other,
}

/// Attorney performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttorneyMetrics {
    pub attorney_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    // Case metrics
    pub total_cases: i32,
    pub active_cases: i32,
    pub completed_cases: i32,
    pub cases_won: i32,
    pub cases_lost: i32,
    pub cases_settled: i32,
    pub cases_dismissed: i32,

    // Filing metrics
    pub total_filings: i32,
    pub motions_filed: i32,
    pub motions_granted: i32,
    pub appeals_filed: i32,
    pub appeals_won: i32,

    // Time metrics
    pub avg_case_duration_days: f64,
    pub avg_response_time_hours: f64,

    // Financial metrics (CJA)
    pub cja_appointments: i32,
    pub cja_hours_billed: f64,
    pub cja_amount_approved: f64,

    // Compliance
    pub missed_deadlines: i32,
    pub sanctions_received: i32,
    pub rule_violations: i32,
}

/// Conflict check request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictCheck {
    pub id: String,
    pub attorney_id: String,
    pub check_date: DateTime<Utc>,
    pub case_id: Option<String>,
    pub party_names: Vec<String>,
    pub adverse_parties: Vec<String>,
    pub conflicts_found: Vec<ConflictResult>,
    pub cleared: bool,
    pub waiver_obtained: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictResult {
    pub party_name: String,
    pub conflict_type: ConflictType,
    pub related_case_id: Option<String>,
    pub related_matter: String,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ConflictType {
    CurrentClient,
    FormerClient,
    PersonalInterest,
    FinancialInterest,
    FamilyRelationship,
    BusinessRelationship,
    PriorInvolvement,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ConflictSeverity {
    None,
    Waivable,
    NonWaivable,
    PotentialFuture,
}

/// Individual conflict of interest record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Conflict {
    pub id: String,
    pub attorney_id: String,
    pub conflict_type: ConflictType,
    pub description: String,
    pub identified_date: DateTime<Utc>,
    pub resolved: bool,
    pub resolution_date: Option<DateTime<Utc>>,
    pub waiver_obtained: bool,
    pub related_case_id: Option<String>,
    pub severity: ConflictSeverity,
}

/// Service status for documents
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ServiceStatus {
    Pending,
    Served,
    Failed,
    Waived,
    Returned,
}

impl Attorney {
    /// Create a new attorney
    pub fn new(
        bar_number: String,
        first_name: String,
        last_name: String,
        email: String,
        phone: String,
        address: Address,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            bar_number,
            first_name,
            last_name,
            middle_name: None,
            firm_name: None,
            email,
            phone,
            fax: None,
            address,
            bar_admissions: Vec::new(),
            federal_admissions: Vec::new(),
            pro_hac_vice_admissions: Vec::new(),
            ecf_registration: None,
            cja_panel_member: false,
            cja_panel_districts: Vec::new(),
            cja_appointments: Vec::new(),
            practice_areas: Vec::new(),
            languages_spoken: vec!["English".to_string()],
            status: AttorneyStatus::Active,
            discipline_history: Vec::new(),
            cases_handled: 0,
            win_rate_percentage: None,
            avg_case_duration_days: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if attorney is in good standing
    pub fn is_in_good_standing(&self) -> bool {
        self.status == AttorneyStatus::Active
            && self.bar_admissions.iter()
                .any(|a| a.status == AdmissionStatus::Active)
    }

    /// Check if attorney can practice in federal court
    pub fn can_practice_federal(&self, court: &str) -> bool {
        self.federal_admissions.iter()
            .any(|a| a.court == court && a.status == AdmissionStatus::Active)
    }

    /// Check if attorney has ECF filing privileges
    pub fn has_ecf_privileges(&self) -> bool {
        self.ecf_registration.as_ref()
            .map(|ecf| ecf.active && ecf.training_completed)
            .unwrap_or(false)
    }

    /// Calculate win rate
    pub fn calculate_win_rate(&self, wins: i32, total: i32) -> Option<f64> {
        if total > 0 {
            Some((wins as f64 / total as f64) * 100.0)
        } else {
            None
        }
    }
}

impl Party {
    /// Create a new party
    pub fn new(
        case_id: String,
        party_type: PartyType,
        name: String,
        entity_type: EntityType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            party_type,
            party_role: PartyRole::Principal,
            name,
            entity_type,
            first_name: None,
            middle_name: None,
            last_name: None,
            date_of_birth: None,
            ssn_last_four: None,
            organization_name: None,
            ein: None,
            address: None,
            phone: None,
            email: None,
            represented: false,
            pro_se: false,
            attorneys: Vec::new(),
            service_address: None,
            service_email: None,
            service_method: ServiceMethod::ECF,
            status: PartyStatus::Active,
            joined_date: now,
            terminated_date: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if party needs service
    pub fn needs_service(&self) -> bool {
        self.status == PartyStatus::Active && !self.pro_se
    }

    /// Get lead counsel for party
    pub fn get_lead_counsel(&self) -> Option<&AttorneyRepresentation> {
        self.attorneys.iter()
            .find(|a| a.lead_counsel && a.end_date.is_none())
    }

    /// Check if party is represented
    pub fn is_represented(&self) -> bool {
        self.represented && self.attorneys.iter()
            .any(|a| a.end_date.is_none())
    }
}

/// Request DTO for creating a new attorney
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAttorneyRequest {
    pub bar_number: String,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub firm_name: Option<String>,
    pub email: String,
    pub phone: String,
    pub fax: Option<String>,
    pub address: Address,
}

/// Request DTO for creating a new party
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePartyRequest {
    pub case_id: String,
    pub party_type: PartyType,
    pub party_role: Option<PartyRole>,
    pub name: String,
    pub entity_type: EntityType,
    // Optional individual fields
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<DateTime<Utc>>,
    pub ssn_last_four: Option<String>,
    // Optional organization fields
    pub organization_name: Option<String>,
    pub ein: Option<String>,
    // Contact information
    pub address: Option<Address>,
    pub phone: Option<String>,
    pub email: Option<String>,
}