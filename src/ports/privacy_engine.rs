//! Port trait for FRCP 5.2 privacy scanning
//!
//! This trait defines the contract for scanning court documents for
//! personally identifiable information (PII) violations under Federal
//! Rule of Civil Procedure 5.2 and Federal Rule of Criminal Procedure 49.1.

use crate::domain::privacy::{PrivacyScanResult, RestrictedDocType};
use crate::error::ApiError;

/// Port trait for FRCP 5.2 privacy scanning
pub trait PrivacyEngine {
    /// Scan document text for PII violations
    fn scan(&self, document_text: &str, case_type: &str) -> Result<PrivacyScanResult, ApiError>;

    /// Check if a document type is auto-restricted under FRCP 5.2(b)
    fn is_restricted_document_type(&self, doc_type: &str) -> bool;

    /// Get the list of all restricted document types
    fn get_restricted_types(&self) -> Vec<RestrictedDocType>;
}
