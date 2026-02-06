//! Shared domain types used across multiple domain modules
//!
//! This module contains canonical definitions for types that are shared
//! between attorney, order, document, and case domains. Each type is
//! defined once here to eliminate duplication and ensure consistency.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Types of conflicts of interest that can be identified
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum ConflictType {
    /// Currently representing the party or an adverse party
    DirectRepresentation,
    /// Previously represented the party and matter is substantially related
    FormerClient,
    /// Representing a co-defendant in same or related matter
    CoDefendant,
    /// Personal interest in the outcome of the matter
    PersonalInterest,
    /// Financial interest in a party or the outcome
    FinancialInterest,
    /// Family relationship with a party
    FamilyRelationship,
    /// Business relationship with a party
    BusinessRelationship,
    /// Attorney's law firm represents an adverse party
    FirmConflict,
    /// Positional conflict - taking inconsistent legal positions
    PositionalConflict,
    /// Issue conflict - working on matters with conflicting interests
    IssueConflict,
    /// Prior involvement in the matter
    PriorInvolvement,
    /// Other type of conflict
    Other(String),
}

impl ConflictType {
    /// Check if this conflict type typically requires special handling
    pub fn requires_special_handling(&self) -> bool {
        matches!(
            self,
            ConflictType::DirectRepresentation
                | ConflictType::FormerClient
                | ConflictType::FirmConflict
                | ConflictType::PositionalConflict
        )
    }

    /// Get a human-readable description of the conflict type
    pub fn description(&self) -> &str {
        match self {
            ConflictType::DirectRepresentation => "Currently representing party or adverse party",
            ConflictType::FormerClient => {
                "Previously represented party in substantially related matter"
            }
            ConflictType::CoDefendant => {
                "Representing co-defendant in same or related matter"
            }
            ConflictType::PersonalInterest => "Personal interest in the outcome",
            ConflictType::FinancialInterest => "Financial interest in party or outcome",
            ConflictType::FamilyRelationship => "Family relationship with party",
            ConflictType::BusinessRelationship => "Business relationship with party",
            ConflictType::FirmConflict => "Law firm represents adverse party",
            ConflictType::PositionalConflict => "Taking inconsistent legal positions",
            ConflictType::IssueConflict => "Working on matters with conflicting interests",
            ConflictType::PriorInvolvement => "Prior involvement in the matter",
            ConflictType::Other(_) => "Other type of conflict",
        }
    }
}

/// Severity levels for conflicts of interest (5-level scale)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum ConflictSeverity {
    /// No actual conflict, but flagged for review
    Informational,
    /// Minor conflict that can typically be waived
    Low,
    /// Moderate conflict requiring careful consideration
    Medium,
    /// Serious conflict that should generally be avoided
    High,
    /// Severe conflict that cannot be waived
    Critical,
}

impl ConflictSeverity {
    /// Get the numeric ordering value for comparison
    pub fn order(&self) -> u8 {
        match self {
            ConflictSeverity::Informational => 0,
            ConflictSeverity::Low => 1,
            ConflictSeverity::Medium => 2,
            ConflictSeverity::High => 3,
            ConflictSeverity::Critical => 4,
        }
    }
}

/// Electronic signature for court documents
///
/// Unified signature struct supporting both judge signatures (on orders)
/// and general document signatures, with all fields from both use cases.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ElectronicSignature {
    /// ID of the signer (judge_id for orders, general signer_id for documents)
    pub signer_id: Option<String>,
    /// Name of the person who signed
    pub signer_name: String,
    /// When the signature was applied
    pub signed_at: DateTime<Utc>,
    /// Cryptographic hash of the signature
    pub signature_hash: String,
    /// Digital certificate ID used for signing
    pub certificate_id: Option<String>,
    /// Verification code for the document
    pub verification_code: Option<String>,
    /// IP address from which the document was signed
    pub ip_address: Option<String>,
}

impl ElectronicSignature {
    /// Create a new electronic signature for a document
    pub fn new(signer_name: String, signature_data: &str) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(signature_data.as_bytes());
        hasher.update(signer_name.as_bytes());
        let timestamp = Utc::now();
        hasher.update(timestamp.to_string().as_bytes());

        let hash = format!("{:x}", hasher.finalize());
        let verification_code = format!("DOC-{}", &hash[..8].to_uppercase());

        Self {
            signer_id: None,
            signer_name,
            signed_at: timestamp,
            signature_hash: hash,
            certificate_id: None,
            verification_code: Some(verification_code),
            ip_address: None,
        }
    }

    /// Create an electronic signature for a judge signing an order
    pub fn for_judge(
        judge_id: String,
        judge_name: String,
        signature_hash: String,
        certificate_id: String,
        ip_address: String,
    ) -> Self {
        Self {
            signer_id: Some(judge_id),
            signer_name: judge_name,
            signed_at: Utc::now(),
            signature_hash,
            certificate_id: Some(certificate_id),
            verification_code: None,
            ip_address: Some(ip_address),
        }
    }
}

/// Method of service for legal documents
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMethod {
    /// Service via the court's electronic filing system
    ElectronicFiling,
    /// In-person delivery of documents
    PersonalService,
    /// Service via USPS certified mail
    CertifiedMail,
    /// Service via regular first-class mail
    RegularMail,
    /// Service via email
    Email,
    /// Electronic Court Filing system
    ECF,
    /// Service by publication in a newspaper
    Publication,
    /// Waiver of formal service
    Waiver,
    /// Other method of service
    Other(String),
}

/// Status of service for a document
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    /// Service has not yet been attempted
    Pending,
    /// Service was successfully completed
    Served,
    /// Service attempt failed
    Failed,
    /// Service was returned (e.g., undeliverable mail)
    Returned,
    /// Formal service was waived
    Waived,
}

/// Status of a motion filed in a case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MotionStatus {
    /// Motion is pending a ruling
    Pending,
    /// Motion was granted
    Granted,
    /// Motion was denied
    Denied,
    /// Motion is moot (no longer relevant)
    Moot,
    /// Motion was withdrawn by the filing party
    Withdrawn,
    /// Motion was granted in part
    GrantedInPart,
    /// Motion was denied without prejudice (can be refiled)
    DeniedWithoutPrejudice,
}

impl std::fmt::Display for MotionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MotionStatus::Pending => write!(f, "pending"),
            MotionStatus::Granted => write!(f, "granted"),
            MotionStatus::Denied => write!(f, "denied"),
            MotionStatus::Moot => write!(f, "moot"),
            MotionStatus::Withdrawn => write!(f, "withdrawn"),
            MotionStatus::GrantedInPart => write!(f, "granted_in_part"),
            MotionStatus::DeniedWithoutPrejudice => write!(f, "denied_without_prejudice"),
        }
    }
}
