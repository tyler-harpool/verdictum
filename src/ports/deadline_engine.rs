//! Port trait for FRCP Rule 6 deadline computation engine
//!
//! Defines the contract for computing court deadlines according to
//! the Federal Rules of Civil Procedure, Rule 6. The engine handles
//! period counting (short vs. long), weekend/holiday exclusions,
//! and service method adjustments.

use chrono::NaiveDate;
use crate::domain::deadline_calc::{DeadlineComputeRequest, DeadlineResult, FederalHoliday};
use crate::error::ApiError;

/// Port trait for FRCP Rule 6 deadline computation
pub trait DeadlineEngine {
    /// Compute a deadline following the FRCP Rule 6 algorithm.
    ///
    /// The algorithm excludes the trigger date, adds service method days,
    /// counts business-only days for short periods (< 11 days) or
    /// calendar days for long periods (>= 11 days), and extends
    /// any landing day that falls on a weekend or holiday.
    fn compute_deadline(&self, request: &DeadlineComputeRequest) -> Result<DeadlineResult, ApiError>;

    /// Check if a date is a federal holiday (including observed dates)
    fn is_federal_holiday(&self, date: NaiveDate) -> bool;

    /// Check if a date is a weekend (Saturday or Sunday)
    fn is_weekend(&self, date: NaiveDate) -> bool;

    /// Get the next business day, skipping weekends and federal holidays
    fn next_business_day(&self, date: NaiveDate) -> NaiveDate;

    /// Get all federal holidays for a given year, including observed dates
    fn get_federal_holidays(&self, year: i32) -> Vec<FederalHoliday>;
}
