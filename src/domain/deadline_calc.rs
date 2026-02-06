//! Deadline computation domain model for federal court system
//!
//! This module defines types for computing court deadlines according to
//! the Federal Rules of Civil and Criminal Procedure, including service
//! method adjustments and federal holiday awareness.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Method of service for deadline computation purposes
///
/// Each method carries additional days that are added to the base
/// deadline period per FRCP Rule 6(d) and FRCrP Rule 45(c).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMethod {
    /// Electronic service (no additional days)
    Electronic,
    /// In-person delivery (no additional days)
    PersonalDelivery,
    /// Service via U.S. mail (3 additional days)
    Mail,
    /// Leaving documents with the clerk of court (3 additional days)
    LeavingWithClerk,
    /// Other method of service (3 additional days)
    Other,
}

impl ServiceMethod {
    /// Returns the number of additional days added to a deadline
    /// based on the service method, per FRCP Rule 6(d).
    ///
    /// Electronic and personal delivery add 0 days.
    /// Mail, leaving with clerk, and other methods add 3 days.
    pub fn additional_days(&self) -> i32 {
        match self {
            ServiceMethod::Electronic | ServiceMethod::PersonalDelivery => 0,
            ServiceMethod::Mail | ServiceMethod::LeavingWithClerk | ServiceMethod::Other => 3,
        }
    }
}

/// Request to compute a deadline based on a triggering event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeadlineComputeRequest {
    /// The date of the triggering event (e.g., date of service)
    pub trigger_date: NaiveDate,
    /// Number of calendar days in the deadline period
    pub period_days: i32,
    /// Method by which the document was served
    pub service_method: ServiceMethod,
    /// Jurisdiction code (e.g., "CACD" for Central District of California)
    pub jurisdiction: String,
    /// Human-readable description of the deadline
    pub description: String,
    /// Citation to the governing rule (e.g., "FRCP 12(a)(1)(A)(i)")
    pub rule_citation: String,
}

/// Result of a deadline computation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeadlineResult {
    /// The computed due date
    pub due_date: NaiveDate,
    /// Human-readable description of the deadline
    pub description: String,
    /// Citation to the governing rule
    pub rule_citation: String,
    /// Notes explaining how the deadline was computed
    pub computation_notes: String,
    /// Whether this is a "short" period (14 days or fewer per FRCP 6(a)(2))
    pub is_short_period: bool,
}

/// A federal holiday used for deadline computation
///
/// Federal courts are closed on these days, and deadlines that fall
/// on a holiday are extended to the next business day per FRCP 6(a)(6).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FederalHoliday {
    /// The date of the holiday
    pub date: NaiveDate,
    /// The name of the holiday (e.g., "Independence Day")
    pub name: String,
}
