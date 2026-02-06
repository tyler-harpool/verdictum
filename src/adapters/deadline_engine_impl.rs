//! FRCP Rule 6 deadline computation engine implementation
//!
//! Implements the federal court deadline computation algorithm per
//! Federal Rules of Civil Procedure, Rule 6(a). Handles short period
//! (< 11 days) business-day counting vs. long period calendar-day
//! counting, service method adjustments per Rule 6(d), and federal
//! holiday awareness with proper Saturday/Sunday observation rules.

use chrono::{Datelike, NaiveDate, Weekday};
use crate::domain::deadline_calc::{DeadlineComputeRequest, DeadlineResult, FederalHoliday};
use crate::error::ApiError;
use crate::ports::deadline_engine::DeadlineEngine;

/// Threshold below which a period is considered "short" and
/// weekends/holidays are excluded from the day count.
const SHORT_PERIOD_THRESHOLD: i32 = 11;

/// FRCP Rule 6 deadline computation engine
///
/// Computes deadlines according to the Federal Rules of Civil
/// Procedure, Rule 6(a), with proper handling of short vs. long
/// periods, service method adjustments, and federal holidays.
pub struct FrcpDeadlineEngine;

impl FrcpDeadlineEngine {
    pub fn new() -> Self {
        Self
    }
}

impl DeadlineEngine for FrcpDeadlineEngine {
    fn compute_deadline(&self, request: &DeadlineComputeRequest) -> Result<DeadlineResult, ApiError> {
        if request.period_days < 0 {
            return Err(ApiError::InvalidInput(
                "Period days cannot be negative".to_string(),
            ));
        }

        let service_additional = request.service_method.additional_days();
        let total_period = request.period_days + service_additional;
        let is_short = total_period < SHORT_PERIOD_THRESHOLD;

        // Step 1: Exclude trigger date — start from the next day
        let start_date = request.trigger_date
            .succ_opt()
            .ok_or_else(|| ApiError::InvalidInput("Trigger date overflow".to_string()))?;

        // Step 2: Count the period
        let raw_due_date = if is_short {
            // Short period: count only business days (exclude weekends and holidays)
            count_business_days(self, start_date, total_period)
        } else {
            // Long period: count all calendar days
            count_calendar_days(start_date, total_period)?
        };

        // Step 3: Landing day check — if due date falls on weekend or holiday, extend
        let due_date = self.next_business_day(raw_due_date);

        // Build computation notes
        let mut notes = Vec::new();
        notes.push(format!(
            "Trigger date: {}; counting begins {}",
            request.trigger_date, start_date
        ));

        if service_additional > 0 {
            notes.push(format!(
                "Service method ({:?}): +{} days added to base period of {} days",
                request.service_method, service_additional, request.period_days
            ));
        }

        notes.push(format!(
            "Total period: {} days ({})",
            total_period,
            if is_short {
                "short period — weekends/holidays excluded from count"
            } else {
                "long period — calendar days counted"
            }
        ));

        if due_date != raw_due_date {
            notes.push(format!(
                "Landing day {} falls on weekend/holiday; extended to next business day {}",
                raw_due_date, due_date
            ));
        }

        notes.push(format!("Due date: {}", due_date));

        Ok(DeadlineResult {
            due_date,
            description: request.description.clone(),
            rule_citation: request.rule_citation.clone(),
            computation_notes: notes.join("; "),
            is_short_period: is_short,
        })
    }

    fn is_federal_holiday(&self, date: NaiveDate) -> bool {
        let holidays = self.get_federal_holidays(date.year());
        holidays.iter().any(|h| h.date == date)
    }

    fn is_weekend(&self, date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    fn next_business_day(&self, date: NaiveDate) -> NaiveDate {
        let mut current = date;
        while self.is_weekend(current) || self.is_federal_holiday(current) {
            current = current.succ_opt().unwrap_or(current);
        }
        current
    }

    fn get_federal_holidays(&self, year: i32) -> Vec<FederalHoliday> {
        let mut holidays = Vec::new();

        // New Year's Day: January 1
        add_observed_holiday(&mut holidays, year, 1, 1, "New Year's Day");

        // Martin Luther King Jr. Day: 3rd Monday of January
        let mlk_date = nth_weekday_of_month(year, 1, Weekday::Mon, 3);
        holidays.push(FederalHoliday {
            date: mlk_date,
            name: "Martin Luther King Jr. Day".to_string(),
        });

        // Presidents' Day: 3rd Monday of February
        let presidents_date = nth_weekday_of_month(year, 2, Weekday::Mon, 3);
        holidays.push(FederalHoliday {
            date: presidents_date,
            name: "Presidents' Day".to_string(),
        });

        // Memorial Day: Last Monday of May
        let memorial_date = last_weekday_of_month(year, 5, Weekday::Mon);
        holidays.push(FederalHoliday {
            date: memorial_date,
            name: "Memorial Day".to_string(),
        });

        // Juneteenth: June 19
        add_observed_holiday(&mut holidays, year, 6, 19, "Juneteenth National Independence Day");

        // Independence Day: July 4
        add_observed_holiday(&mut holidays, year, 7, 4, "Independence Day");

        // Labor Day: 1st Monday of September
        let labor_date = nth_weekday_of_month(year, 9, Weekday::Mon, 1);
        holidays.push(FederalHoliday {
            date: labor_date,
            name: "Labor Day".to_string(),
        });

        // Columbus Day: 2nd Monday of October
        let columbus_date = nth_weekday_of_month(year, 10, Weekday::Mon, 2);
        holidays.push(FederalHoliday {
            date: columbus_date,
            name: "Columbus Day".to_string(),
        });

        // Veterans Day: November 11
        add_observed_holiday(&mut holidays, year, 11, 11, "Veterans Day");

        // Thanksgiving: 4th Thursday of November
        let thanksgiving_date = nth_weekday_of_month(year, 11, Weekday::Thu, 4);
        holidays.push(FederalHoliday {
            date: thanksgiving_date,
            name: "Thanksgiving Day".to_string(),
        });

        // Christmas Day: December 25
        add_observed_holiday(&mut holidays, year, 12, 25, "Christmas Day");

        // Sort by date for consistent ordering
        holidays.sort_by_key(|h| h.date);
        holidays
    }
}

/// Count business days from a start date, skipping weekends and holidays.
/// Used for short periods (< 11 days).
fn count_business_days(engine: &FrcpDeadlineEngine, start: NaiveDate, days: i32) -> NaiveDate {
    if days <= 0 {
        return start;
    }

    let mut counted = 0;
    let mut current = start;

    while counted < days {
        if !engine.is_weekend(current) && !engine.is_federal_holiday(current) {
            counted += 1;
        }
        if counted < days {
            current = current.succ_opt().unwrap_or(current);
        }
    }

    current
}

/// Count calendar days from a start date. Used for long periods (>= 11 days).
fn count_calendar_days(start: NaiveDate, days: i32) -> Result<NaiveDate, ApiError> {
    if days <= 0 {
        return Ok(start);
    }

    // Subtract 1 because start itself is day 1
    start
        .checked_add_signed(chrono::Duration::days((days - 1) as i64))
        .ok_or_else(|| ApiError::InvalidInput("Date overflow during calendar day count".to_string()))
}

/// Compute the nth occurrence of a given weekday in a month.
///
/// For example, `nth_weekday_of_month(2025, 1, Monday, 3)` returns
/// the 3rd Monday of January 2025 (Martin Luther King Jr. Day).
fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
    let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let first_weekday = first_of_month.weekday();

    // Days until the first occurrence of the target weekday
    let days_ahead = (weekday.num_days_from_monday() as i32
        - first_weekday.num_days_from_monday() as i32
        + 7) % 7;

    // Offset to the nth occurrence (n is 1-based)
    let day = 1 + days_ahead as u32 + (n - 1) * 7;

    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// Compute the last occurrence of a given weekday in a month.
///
/// For example, `last_weekday_of_month(2025, 5, Monday)` returns
/// the last Monday of May 2025 (Memorial Day).
fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    // Start from the last day of the month
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    let last_of_month = last_day.pred_opt().unwrap();

    let last_weekday = last_of_month.weekday();
    let days_back = (last_weekday.num_days_from_monday() as i32
        - weekday.num_days_from_monday() as i32
        + 7) % 7;

    NaiveDate::from_ymd_opt(year, month, last_of_month.day() - days_back as u32).unwrap()
}

/// Apply the federal holiday observation rule:
/// - Saturday holidays are observed on the preceding Friday
/// - Sunday holidays are observed on the following Monday
///
/// The observed date is what gets added to the holidays list.
fn observed_date(date: NaiveDate) -> NaiveDate {
    match date.weekday() {
        Weekday::Sat => date.pred_opt().unwrap(), // Friday
        Weekday::Sun => date.succ_opt().unwrap(), // Monday
        _ => date,
    }
}

/// Add a fixed-date holiday with proper observation rules.
/// If the actual date falls on a weekend, the observed date is used instead.
fn add_observed_holiday(
    holidays: &mut Vec<FederalHoliday>,
    year: i32,
    month: u32,
    day: u32,
    name: &str,
) {
    let actual = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    let obs = observed_date(actual);
    holidays.push(FederalHoliday {
        date: obs,
        name: name.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::deadline_calc::ServiceMethod;

    fn engine() -> FrcpDeadlineEngine {
        FrcpDeadlineEngine::new()
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn make_request(
        trigger: NaiveDate,
        period: i32,
        service: ServiceMethod,
    ) -> DeadlineComputeRequest {
        DeadlineComputeRequest {
            trigger_date: trigger,
            period_days: period,
            service_method: service,
            jurisdiction: "TEST".to_string(),
            description: "Test deadline".to_string(),
            rule_citation: "FRCP 12(a)".to_string(),
        }
    }

    // --- Holiday computation tests ---

    #[test]
    fn mlk_day_2025() {
        // 3rd Monday of January 2025 = Jan 20
        assert_eq!(nth_weekday_of_month(2025, 1, Weekday::Mon, 3), date(2025, 1, 20));
    }

    #[test]
    fn presidents_day_2025() {
        // 3rd Monday of February 2025 = Feb 17
        assert_eq!(nth_weekday_of_month(2025, 2, Weekday::Mon, 3), date(2025, 2, 17));
    }

    #[test]
    fn memorial_day_2025() {
        // Last Monday of May 2025 = May 26
        assert_eq!(last_weekday_of_month(2025, 5, Weekday::Mon), date(2025, 5, 26));
    }

    #[test]
    fn labor_day_2025() {
        // 1st Monday of September 2025 = Sep 1
        assert_eq!(nth_weekday_of_month(2025, 9, Weekday::Mon, 1), date(2025, 9, 1));
    }

    #[test]
    fn thanksgiving_2025() {
        // 4th Thursday of November 2025 = Nov 27
        assert_eq!(nth_weekday_of_month(2025, 11, Weekday::Thu, 4), date(2025, 11, 27));
    }

    #[test]
    fn observation_saturday_to_friday() {
        // July 4, 2026 is Saturday -> observed Friday July 3
        assert_eq!(observed_date(date(2026, 7, 4)), date(2026, 7, 3));
    }

    #[test]
    fn observation_sunday_to_monday() {
        // Jan 1, 2028 is Saturday -> observed Dec 31, 2027
        // Actually let's pick a known Sunday: July 4, 2027 is Sunday
        assert_eq!(observed_date(date(2027, 7, 4)), date(2027, 7, 5));
    }

    #[test]
    fn observation_weekday_unchanged() {
        // July 4, 2025 is Friday -> no change
        assert_eq!(observed_date(date(2025, 7, 4)), date(2025, 7, 4));
    }

    #[test]
    fn holiday_list_has_eleven_entries() {
        let e = engine();
        let holidays = e.get_federal_holidays(2025);
        assert_eq!(holidays.len(), 11);
    }

    #[test]
    fn christmas_2027_observed_on_friday() {
        // Dec 25, 2027 is Saturday -> observed Dec 24 (Friday)
        let e = engine();
        let holidays = e.get_federal_holidays(2027);
        let xmas = holidays.iter().find(|h| h.name == "Christmas Day").unwrap();
        assert_eq!(xmas.date, date(2027, 12, 24));
    }

    // --- Weekend check tests ---

    #[test]
    fn saturday_is_weekend() {
        let e = engine();
        assert!(e.is_weekend(date(2025, 10, 4))); // Saturday
    }

    #[test]
    fn sunday_is_weekend() {
        let e = engine();
        assert!(e.is_weekend(date(2025, 10, 5))); // Sunday
    }

    #[test]
    fn monday_is_not_weekend() {
        let e = engine();
        assert!(!e.is_weekend(date(2025, 10, 6))); // Monday
    }

    // --- Next business day tests ---

    #[test]
    fn next_business_day_on_weekday() {
        let e = engine();
        // Wednesday Oct 8, 2025 is a regular weekday
        assert_eq!(e.next_business_day(date(2025, 10, 8)), date(2025, 10, 8));
    }

    #[test]
    fn next_business_day_on_saturday() {
        let e = engine();
        // Saturday Oct 4, 2025 -> Monday Oct 6
        assert_eq!(e.next_business_day(date(2025, 10, 4)), date(2025, 10, 6));
    }

    #[test]
    fn next_business_day_on_holiday() {
        let e = engine();
        // Christmas 2025 is Thursday Dec 25 -> next business day is Friday Dec 26
        assert_eq!(e.next_business_day(date(2025, 12, 25)), date(2025, 12, 26));
    }

    // --- Short period counting tests ---

    #[test]
    fn short_period_five_business_days() {
        let e = engine();
        // 5-day period served electronically from Mon Oct 6, 2025
        // Trigger: Mon Oct 6; counting starts Tue Oct 7
        // Day 1: Tue Oct 7, Day 2: Wed Oct 8, Day 3: Thu Oct 9,
        // Day 4: Fri Oct 10, Day 5: Mon Oct 13 (skip weekend)
        let req = make_request(date(2025, 10, 6), 5, ServiceMethod::Electronic);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 10, 13));
        assert!(result.is_short_period);
    }

    #[test]
    fn short_period_with_mail_service() {
        let e = engine();
        // 5-day base + 3 mail days = 8 total (short)
        // Trigger: Mon Oct 6, 2025; counting starts Tue Oct 7
        // 8 business days: Tue-Fri (4) + Mon-Fri next week (4) = Oct 16 (Thu)
        let req = make_request(date(2025, 10, 6), 5, ServiceMethod::Mail);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 10, 16));
        assert!(result.is_short_period);
    }

    #[test]
    fn short_period_with_mail_crosses_eleven_becomes_long() {
        let e = engine();
        // 10-day base + 3 mail = 13 total (long period, >= 11)
        // Trigger: Mon Oct 6, 2025; counting starts Tue Oct 7
        // 12 calendar days from Oct 7 = Oct 18 (Saturday)
        // Landing day is Saturday -> next business day = Mon Oct 20
        let req = make_request(date(2025, 10, 6), 10, ServiceMethod::Mail);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 10, 20));
        assert!(!result.is_short_period);
    }

    // --- Long period counting tests ---

    #[test]
    fn long_period_thirty_days() {
        let e = engine();
        // 30-day period from Tue Oct 7, 2025
        // Trigger: Oct 7; counting starts Oct 8
        // 29 calendar days from Oct 8 = Nov 5 (Wednesday)
        // Nov 5 is a weekday, no holiday -> due Nov 5
        let req = make_request(date(2025, 10, 7), 30, ServiceMethod::Electronic);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 11, 5));
        assert!(!result.is_short_period);
    }

    #[test]
    fn long_period_landing_on_weekend() {
        let e = engine();
        // 14-day period from Mon Oct 6, 2025
        // Trigger: Oct 6; counting starts Oct 7
        // 13 calendar days from Oct 7 = Oct 19 (Sunday)
        // Landing day is Sunday -> next business day = Mon Oct 20
        let req = make_request(date(2025, 10, 6), 14, ServiceMethod::Electronic);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 10, 20));
        assert!(!result.is_short_period);
    }

    #[test]
    fn long_period_landing_on_holiday() {
        let e = engine();
        // Craft a request that lands on Christmas 2025 (Thu Dec 25)
        // Trigger: Nov 25, 2025; counting starts Nov 26
        // 30-day period: 29 calendar days from Nov 26 = Dec 24 (Wed)
        // Dec 24 is not a holiday -> due Dec 24
        // Let's try 31 days: 30 cal days from Nov 26 = Dec 25 (Thu, Christmas)
        // Landing on holiday -> next business day = Dec 26 (Fri)
        let req = make_request(date(2025, 11, 25), 31, ServiceMethod::Electronic);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 12, 26));
    }

    // --- Edge cases ---

    #[test]
    fn zero_day_period() {
        let e = engine();
        // 0-day period from Mon Oct 6, 2025
        // Start is Oct 7, 0 days means due on start = Oct 7
        let req = make_request(date(2025, 10, 6), 0, ServiceMethod::Electronic);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 10, 7));
    }

    #[test]
    fn negative_period_is_error() {
        let e = engine();
        let req = make_request(date(2025, 10, 6), -1, ServiceMethod::Electronic);
        assert!(e.compute_deadline(&req).is_err());
    }

    #[test]
    fn one_day_period_on_friday() {
        let e = engine();
        // 1-day period, trigger Friday Oct 10, 2025
        // Start: Sat Oct 11, which is weekend, skip to Mon Oct 13 counting
        // Day 1 is Mon Oct 13 (business day)
        let req = make_request(date(2025, 10, 10), 1, ServiceMethod::Electronic);
        let result = e.compute_deadline(&req).unwrap();
        assert_eq!(result.due_date, date(2025, 10, 13));
        assert!(result.is_short_period);
    }

    #[test]
    fn is_federal_holiday_positive() {
        let e = engine();
        // Christmas 2025 is Thursday Dec 25
        assert!(e.is_federal_holiday(date(2025, 12, 25)));
    }

    #[test]
    fn is_federal_holiday_negative() {
        let e = engine();
        // Dec 24, 2025 is NOT a federal holiday
        assert!(!e.is_federal_holiday(date(2025, 12, 24)));
    }

    #[test]
    fn is_federal_holiday_observed_date() {
        let e = engine();
        // July 4, 2026 is Saturday -> observed Friday July 3
        assert!(e.is_federal_holiday(date(2026, 7, 3)));
        assert!(!e.is_federal_holiday(date(2026, 7, 4)));
    }
}
