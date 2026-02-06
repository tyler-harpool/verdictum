//! Privacy protection domain model for federal court system
//!
//! This module defines types for detecting and redacting personally
//! identifiable information (PII) in court filings, as required by
//! Federal Rule of Civil Procedure 5.2 and Federal Rule of Criminal
//! Procedure 49.1.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Categories of personally identifiable information subject to redaction
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PiiType {
    /// Social Security number (show only last 4 digits)
    Ssn,
    /// Taxpayer identification number
    TaxpayerId,
    /// Date of birth (show only year)
    DateOfBirth,
    /// Name of a minor (use initials only)
    MinorName,
    /// Financial account number (show only last 4 digits)
    FinancialAccount,
    /// Home address (show only city and state in criminal cases)
    HomeAddress,
}

/// A match of PII found during a privacy scan
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PiiMatch {
    /// The type of PII detected
    pub pii_type: PiiType,
    /// Start character position in the scanned text
    pub start_position: usize,
    /// End character position in the scanned text
    pub end_position: usize,
    /// The original unredacted text that was matched
    pub original_text: String,
    /// The required redacted format (e.g., "XXX-XX-1234")
    pub required_format: String,
}

/// Result of scanning a document for PII violations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PrivacyScanResult {
    /// Whether the document is clean of PII violations
    pub clean: bool,
    /// List of PII violations found
    pub violations: Vec<PiiMatch>,
    /// Whether the document is restricted from public access
    pub restricted: bool,
    /// Reason for restricting the document, if applicable
    pub restriction_reason: Option<String>,
}

/// Document types that are restricted from public access per policy
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RestrictedDocType {
    /// Unexecuted arrest or search warrants
    UnexecutedWarrant,
    /// Presentence investigation reports
    PresentenceReport,
    /// Statement of reasons in sentencing
    StatementOfReasons,
    /// Criminal Justice Act financial affidavit
    CjaFinancialAffidavit,
    /// Juvenile records and proceedings
    JuvenileRecord,
    /// Juror questionnaires and identifying information
    JurorInfo,
    /// Court-sealed documents
    SealedDocument,
}
