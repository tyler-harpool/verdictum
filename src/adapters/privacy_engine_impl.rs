//! FRCP 5.2 Privacy Engine implementation
//!
//! Scans court documents for personally identifiable information (PII)
//! using string-based pattern matching. Implements false positive filtering
//! for case numbers, phone numbers, and procedural date references.

use crate::domain::privacy::{PiiMatch, PiiType, PrivacyScanResult, RestrictedDocType};
use crate::error::ApiError;
use crate::ports::privacy_engine::PrivacyEngine;

/// FRCP 5.2 privacy engine using string-based PII detection
pub struct FrcpPrivacyEngine;

impl FrcpPrivacyEngine {
    pub fn new() -> Self {
        Self
    }

    /// Check if a character sequence at the given position looks like a
    /// federal case number (e.g., "5:24-cv-05001"), which should NOT
    /// be flagged as an SSN.
    fn is_case_number_context(&self, text: &str, start: usize) -> bool {
        // Look backwards from start for a digit-colon pattern like "5:"
        // Federal case numbers: d:dd-xx-ddddd
        let before = if start >= 5 { &text[start - 5..start] } else { &text[..start] };
        // Check if there's a "d:" pattern before, indicating case number format
        for (i, ch) in before.char_indices().rev() {
            if ch == ':' {
                // Check the char before colon is a digit
                let abs_pos = if start >= 5 { start - 5 + i } else { i };
                if abs_pos > 0 {
                    let prev_char = text.as_bytes().get(abs_pos - 1);
                    if let Some(&c) = prev_char {
                        if c.is_ascii_digit() {
                            return true;
                        }
                    }
                }
            }
        }

        // Also check if the text around this position contains common case
        // number patterns: look for "-cv-", "-cr-", "-mc-", "-mj-" nearby
        let window_start = start.saturating_sub(20);
        let window_end = (start + 30).min(text.len());
        let window = &text[window_start..window_end].to_lowercase();
        window.contains("-cv-")
            || window.contains("-cr-")
            || window.contains("-mc-")
            || window.contains("-mj-")
            || window.contains("-po-")
    }

    /// Check if a matched SSN is already properly redacted (XXX-XX-dddd)
    fn is_already_redacted(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.starts_with("xxx-xx-") || lower.starts_with("xxx xx ")
    }

    /// Check if a sequence of digits is part of a phone number pattern
    fn is_phone_number_context(&self, text: &str, start: usize, end: usize) -> bool {
        // Look for parenthesized area code before the digits: (ddd) nearby
        let window_start = start.saturating_sub(10);
        let before = &text[window_start..start];

        // Check for "(ddd)" or "(ddd) " pattern
        if let Some(paren_pos) = before.rfind('(') {
            let after_paren = &before[paren_pos..];
            let digits_in_paren = after_paren
                .chars()
                .filter(|c| c.is_ascii_digit())
                .count();
            if digits_in_paren == 3 && after_paren.contains(')') {
                return true;
            }
        }

        // Check for common phone separators within the matched text
        let matched = &text[start..end];
        if matched.contains('(') && matched.contains(')') {
            return true;
        }

        // Check for "phone", "tel", "fax", "call" context
        let context_start = start.saturating_sub(30);
        let context = text[context_start..start].to_lowercase();
        context.contains("phone")
            || context.contains("tel")
            || context.contains("fax")
            || context.contains("call")
            || context.contains("mobile")
            || context.contains("cell")
    }

    /// Scan for Social Security Numbers: ddd-dd-dddd or ddd dd dddd
    fn scan_ssn(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let mut i = 0;
        while i < len {
            // Need at least 9 digits + 2 separators = 11 chars, or 9 consecutive digits
            if chars[i].is_ascii_digit() {
                if let Some(m) = self.try_match_ssn(&chars, i, text) {
                    let byte_start = self.char_to_byte_pos(text, i);
                    let byte_end = self.char_to_byte_pos(text, i + m.char_len);

                    // Filter: skip if already redacted
                    if self.is_already_redacted(&m.matched_text) {
                        i += m.char_len;
                        continue;
                    }

                    // Filter: skip if it looks like a case number
                    if self.is_case_number_context(text, byte_start) {
                        i += m.char_len;
                        continue;
                    }

                    // Extract last 4 digits for redacted format
                    let digits: String = m.matched_text.chars().filter(|c| c.is_ascii_digit()).collect();
                    let last_four = &digits[digits.len() - 4..];

                    matches.push(PiiMatch {
                        pii_type: PiiType::Ssn,
                        start_position: byte_start,
                        end_position: byte_end,
                        original_text: m.matched_text,
                        required_format: format!("XXX-XX-{}", last_four),
                    });

                    i += m.char_len;
                    continue;
                }
            }
            i += 1;
        }

        matches
    }

    /// Try to match an SSN pattern starting at position i in the char array.
    /// SSN format: 3 digits, separator, 2 digits, separator, 4 digits
    /// Separators can be '-' or ' '
    fn try_match_ssn(&self, chars: &[char], start: usize, _text: &str) -> Option<SsnMatch> {
        let len = chars.len();

        // Check we have at least enough chars for ddd-dd-dddd (11)
        // or ddddddddd (9)
        if start + 9 > len {
            return None;
        }

        // Ensure the character before start is not a digit (word boundary)
        if start > 0 && chars[start - 1].is_ascii_digit() {
            return None;
        }

        // Try format: ddd[- ]dd[- ]dddd
        let d1 = chars[start].is_ascii_digit();
        let d2 = chars[start + 1].is_ascii_digit();
        let d3 = chars[start + 2].is_ascii_digit();

        if !(d1 && d2 && d3) {
            return None;
        }

        // Check for separator after first group
        let sep1_pos = start + 3;
        if sep1_pos >= len {
            return None;
        }

        let sep1 = chars[sep1_pos];
        if sep1 == '-' || sep1 == ' ' {
            // Format with separators: ddd-dd-dddd
            if start + 10 >= len {
                return None;
            }

            let d4 = chars[sep1_pos + 1].is_ascii_digit();
            let d5 = chars[sep1_pos + 2].is_ascii_digit();
            let sep2 = chars[sep1_pos + 3];
            let d6 = chars[sep1_pos + 4].is_ascii_digit();
            let d7 = chars[sep1_pos + 5].is_ascii_digit();
            let d8 = chars[sep1_pos + 6].is_ascii_digit();
            let d9 = chars[sep1_pos + 7].is_ascii_digit();

            if d4 && d5 && (sep2 == '-' || sep2 == ' ') && d6 && d7 && d8 && d9 {
                let char_len = 11; // ddd-dd-dddd
                // Ensure next char is not a digit (word boundary)
                if start + char_len < len && chars[start + char_len].is_ascii_digit() {
                    return None;
                }
                let matched: String = chars[start..start + char_len].iter().collect();
                return Some(SsnMatch {
                    matched_text: matched,
                    char_len,
                });
            }
        }

        // Try format without separators: ddddddddd (9 consecutive digits)
        // But NOT more than 9 digits (that's a financial account)
        let mut digit_count = 0;
        let mut pos = start;
        while pos < len && chars[pos].is_ascii_digit() {
            digit_count += 1;
            pos += 1;
        }

        if digit_count == 9 {
            let matched: String = chars[start..start + 9].iter().collect();
            return Some(SsnMatch {
                matched_text: matched,
                char_len: 9,
            });
        }

        None
    }

    /// Scan for Taxpayer ID Numbers: dd-ddddddd
    fn scan_taxpayer_id(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let mut i = 0;
        while i < len {
            if chars[i].is_ascii_digit() {
                if let Some(m) = self.try_match_taxpayer_id(&chars, i) {
                    let byte_start = self.char_to_byte_pos(text, i);
                    let byte_end = self.char_to_byte_pos(text, i + m.char_len);

                    // Filter: skip case numbers
                    if self.is_case_number_context(text, byte_start) {
                        i += m.char_len;
                        continue;
                    }

                    // Look for "EIN", "TIN", "taxpayer" context to reduce false positives
                    let context_start = byte_start.saturating_sub(40);
                    let context = text[context_start..byte_start].to_lowercase();
                    let has_tax_context = context.contains("ein")
                        || context.contains("tin")
                        || context.contains("taxpayer")
                        || context.contains("tax id")
                        || context.contains("employer identification");

                    if !has_tax_context {
                        i += m.char_len;
                        continue;
                    }

                    let digits: String = m.matched_text.chars().filter(|c| c.is_ascii_digit()).collect();
                    let last_four = &digits[digits.len() - 4..];

                    matches.push(PiiMatch {
                        pii_type: PiiType::TaxpayerId,
                        start_position: byte_start,
                        end_position: byte_end,
                        original_text: m.matched_text,
                        required_format: format!("XX-XXX{}", last_four),
                    });

                    i += m.char_len;
                    continue;
                }
            }
            i += 1;
        }

        matches
    }

    /// Try to match a Taxpayer ID: dd[-]ddddddd
    fn try_match_taxpayer_id(&self, chars: &[char], start: usize) -> Option<SsnMatch> {
        let len = chars.len();
        if start + 9 > len {
            return None;
        }

        // Word boundary check
        if start > 0 && chars[start - 1].is_ascii_digit() {
            return None;
        }

        let d1 = chars[start].is_ascii_digit();
        let d2 = chars[start + 1].is_ascii_digit();

        if !(d1 && d2) {
            return None;
        }

        // Format: dd-ddddddd
        if chars[start + 2] == '-' {
            if start + 10 > len {
                return None;
            }
            let all_digits = (3..10).all(|offset| chars[start + offset].is_ascii_digit());
            if all_digits {
                let char_len = 10;
                if start + char_len < len && chars[start + char_len].is_ascii_digit() {
                    return None;
                }
                let matched: String = chars[start..start + char_len].iter().collect();
                return Some(SsnMatch {
                    matched_text: matched,
                    char_len,
                });
            }
        }

        // Format: dd ddddddd (with space separator)
        if chars[start + 2] == ' ' {
            if start + 10 > len {
                return None;
            }
            let all_digits = (3..10).all(|offset| chars[start + offset].is_ascii_digit());
            if all_digits {
                let char_len = 10;
                if start + char_len < len && chars[start + char_len].is_ascii_digit() {
                    return None;
                }
                let matched: String = chars[start..start + char_len].iter().collect();
                return Some(SsnMatch {
                    matched_text: matched,
                    char_len,
                });
            }
        }

        None
    }

    /// Scan for Date of Birth patterns preceded by DOB keywords
    fn scan_dob(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();
        let lower = text.to_lowercase();

        // DOB keywords to search for
        let keywords = ["date of birth", "dob", "born"];

        for keyword in &keywords {
            let mut search_from = 0;
            while let Some(kw_pos) = lower[search_from..].find(keyword) {
                let abs_kw_pos = search_from + kw_pos;
                let after_keyword = abs_kw_pos + keyword.len();

                // Skip optional separator characters: whitespace, ':', '.'
                let mut scan_pos = after_keyword;
                while scan_pos < text.len() {
                    let ch = text.as_bytes()[scan_pos];
                    if ch == b' ' || ch == b':' || ch == b'.' || ch == b'\t' {
                        scan_pos += 1;
                    } else {
                        break;
                    }
                }

                // Try to find a date pattern after the keyword
                if let Some(date_match) = self.try_match_date(&text[scan_pos..]) {
                    let byte_start = scan_pos;
                    let byte_end = scan_pos + date_match.char_len;

                    // Extract year from the date for redacted format
                    let year = self.extract_year_from_date(&date_match.matched_text);

                    matches.push(PiiMatch {
                        pii_type: PiiType::DateOfBirth,
                        start_position: byte_start,
                        end_position: byte_end,
                        original_text: date_match.matched_text,
                        required_format: format!("Year only: {}", year),
                    });
                }

                search_from = abs_kw_pos + keyword.len();
            }
        }

        matches
    }

    /// Try to match common date formats: MM/DD/YYYY, MM-DD-YYYY,
    /// Month DD, YYYY, YYYY-MM-DD
    fn try_match_date(&self, text: &str) -> Option<SsnMatch> {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 6 {
            return None;
        }

        // Format: MM/DD/YYYY or MM-DD-YYYY
        if len >= 10 && chars[0].is_ascii_digit() && chars[1].is_ascii_digit() {
            let sep = chars[2];
            if (sep == '/' || sep == '-')
                && chars[3].is_ascii_digit()
                && chars[4].is_ascii_digit()
                && chars[5] == sep
                && chars[6].is_ascii_digit()
                && chars[7].is_ascii_digit()
                && chars[8].is_ascii_digit()
                && chars[9].is_ascii_digit()
            {
                let matched: String = chars[..10].iter().collect();
                return Some(SsnMatch {
                    matched_text: matched,
                    char_len: 10,
                });
            }
        }

        // Format: YYYY-MM-DD
        if len >= 10
            && chars[0].is_ascii_digit()
            && chars[1].is_ascii_digit()
            && chars[2].is_ascii_digit()
            && chars[3].is_ascii_digit()
            && chars[4] == '-'
            && chars[5].is_ascii_digit()
            && chars[6].is_ascii_digit()
            && chars[7] == '-'
            && chars[8].is_ascii_digit()
            && chars[9].is_ascii_digit()
        {
            let matched: String = chars[..10].iter().collect();
            return Some(SsnMatch {
                matched_text: matched,
                char_len: 10,
            });
        }

        // Format: Month DD, YYYY (e.g., "January 15, 1990")
        let months = [
            "january", "february", "march", "april", "may", "june",
            "july", "august", "september", "october", "november", "december",
        ];
        let text_lower = text.to_lowercase();
        for month in &months {
            if text_lower.starts_with(month) {
                let after_month = month.len();
                // Skip space
                let mut pos = after_month;
                while pos < len && chars[pos] == ' ' {
                    pos += 1;
                }
                // Read day digits
                let day_start = pos;
                while pos < len && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
                if pos == day_start {
                    continue;
                }
                // Skip comma and space
                if pos < len && chars[pos] == ',' {
                    pos += 1;
                }
                while pos < len && chars[pos] == ' ' {
                    pos += 1;
                }
                // Read year (4 digits)
                let year_start = pos;
                while pos < len && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
                if pos - year_start == 4 {
                    let matched: String = chars[..pos].iter().collect();
                    return Some(SsnMatch {
                        matched_text: matched,
                        char_len: pos,
                    });
                }
            }
        }

        None
    }

    /// Extract the year from a date string
    fn extract_year_from_date(&self, date_text: &str) -> String {
        // Try to find a 4-digit year
        let chars: Vec<char> = date_text.chars().collect();
        let mut i = 0;
        while i + 3 < chars.len() {
            if chars[i].is_ascii_digit()
                && chars[i + 1].is_ascii_digit()
                && chars[i + 2].is_ascii_digit()
                && chars[i + 3].is_ascii_digit()
            {
                let year: String = chars[i..i + 4].iter().collect();
                // Validate reasonable year range
                if let Ok(y) = year.parse::<u32>() {
                    if (1900..=2100).contains(&y) {
                        return year;
                    }
                }
            }
            i += 1;
        }
        "REDACTED".to_string()
    }

    /// Scan for financial account numbers (8+ consecutive digits in
    /// financial context)
    fn scan_financial_accounts(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let mut i = 0;
        while i < len {
            if chars[i].is_ascii_digit() {
                // Count consecutive digits
                let start = i;
                while i < len && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let digit_count = i - start;

                // Only flag sequences of 8 or more digits
                if digit_count >= 8 {
                    let byte_start = self.char_to_byte_pos(text, start);
                    let byte_end = self.char_to_byte_pos(text, i);

                    // Filter: skip phone numbers
                    if self.is_phone_number_context(text, byte_start, byte_end) {
                        continue;
                    }

                    // Filter: skip case numbers
                    if self.is_case_number_context(text, byte_start) {
                        continue;
                    }

                    // Only flag in financial context
                    let context_start = byte_start.saturating_sub(50);
                    let context_end = (byte_end + 50).min(text.len());
                    let context = text[context_start..context_end].to_lowercase();

                    let has_financial_context = context.contains("account")
                        || context.contains("routing")
                        || context.contains("bank")
                        || context.contains("acct")
                        || context.contains("deposit")
                        || context.contains("wire")
                        || context.contains("swift")
                        || context.contains("iban")
                        || context.contains("checking")
                        || context.contains("savings")
                        || context.contains("credit card");

                    if has_financial_context {
                        let matched: String = chars[start..i].iter().collect();
                        let last_four = matched[matched.len() - 4..].to_string();
                        let required_fmt = format!("XXXX{}", last_four);

                        matches.push(PiiMatch {
                            pii_type: PiiType::FinancialAccount,
                            start_position: byte_start,
                            end_position: byte_end,
                            original_text: matched,
                            required_format: required_fmt,
                        });
                    }
                }
                continue;
            }
            i += 1;
        }

        matches
    }

    /// Convert a character index to byte position in the string
    fn char_to_byte_pos(&self, text: &str, char_idx: usize) -> usize {
        text.char_indices()
            .nth(char_idx)
            .map(|(byte_pos, _)| byte_pos)
            .unwrap_or(text.len())
    }

    /// Determine the restricted document type from a document type string
    fn classify_restricted_type(&self, doc_type: &str) -> Option<(RestrictedDocType, String)> {
        let lower = doc_type.to_lowercase();

        if lower.contains("warrant") || lower.contains("summons") {
            Some((
                RestrictedDocType::UnexecutedWarrant,
                "Unexecuted warrants/summons restricted under FRCP 5.2(b)".to_string(),
            ))
        } else if lower.contains("presentence")
            || lower == "psi"
            || lower.contains("presentence investigation")
        {
            Some((
                RestrictedDocType::PresentenceReport,
                "Presentence investigation reports restricted under FRCP 5.2(b)".to_string(),
            ))
        } else if lower.contains("statement of reasons") || lower == "sor" {
            Some((
                RestrictedDocType::StatementOfReasons,
                "Statement of reasons restricted under FRCP 5.2(b)".to_string(),
            ))
        } else if lower.contains("cja affidavit") || lower.contains("cja financial") {
            Some((
                RestrictedDocType::CjaFinancialAffidavit,
                "CJA financial affidavits restricted under FRCP 5.2(b)".to_string(),
            ))
        } else if lower.contains("juvenile") {
            Some((
                RestrictedDocType::JuvenileRecord,
                "Juvenile records restricted under FRCP 5.2(b)".to_string(),
            ))
        } else if lower.contains("juror") {
            Some((
                RestrictedDocType::JurorInfo,
                "Juror information restricted under FRCP 5.2(b)".to_string(),
            ))
        } else if lower.contains("sealed") {
            Some((
                RestrictedDocType::SealedDocument,
                "Sealed documents restricted under court order".to_string(),
            ))
        } else {
            None
        }
    }
}

/// Internal helper struct for pattern match results
struct SsnMatch {
    matched_text: String,
    char_len: usize,
}

impl PrivacyEngine for FrcpPrivacyEngine {
    fn scan(&self, document_text: &str, case_type: &str) -> Result<PrivacyScanResult, ApiError> {
        // Step 1: Check if document type is auto-restricted
        let (restricted, restriction_reason) =
            if let Some((_, reason)) = self.classify_restricted_type(case_type) {
                (true, Some(reason))
            } else {
                (false, None)
            };

        // Step 2: Run all PII detection patterns
        let mut violations = Vec::new();
        violations.extend(self.scan_ssn(document_text));
        violations.extend(self.scan_taxpayer_id(document_text));
        violations.extend(self.scan_dob(document_text));
        violations.extend(self.scan_financial_accounts(document_text));

        // Step 3: Sort violations by position for consistent output
        violations.sort_by_key(|v| v.start_position);

        // Step 4: Build result
        let clean = violations.is_empty() && !restricted;

        Ok(PrivacyScanResult {
            clean,
            violations,
            restricted,
            restriction_reason,
        })
    }

    fn is_restricted_document_type(&self, doc_type: &str) -> bool {
        self.classify_restricted_type(doc_type).is_some()
    }

    fn get_restricted_types(&self) -> Vec<RestrictedDocType> {
        vec![
            RestrictedDocType::UnexecutedWarrant,
            RestrictedDocType::PresentenceReport,
            RestrictedDocType::StatementOfReasons,
            RestrictedDocType::CjaFinancialAffidavit,
            RestrictedDocType::JuvenileRecord,
            RestrictedDocType::JurorInfo,
            RestrictedDocType::SealedDocument,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::privacy::PiiType;

    fn engine() -> FrcpPrivacyEngine {
        FrcpPrivacyEngine::new()
    }

    // --- SSN Detection Tests ---

    #[test]
    fn test_unredacted_ssn_detected() {
        let e = engine();
        let result = e.scan("John's SSN is 123-45-6789", "motion").unwrap();

        assert!(!result.clean, "Document with unredacted SSN should not be clean");
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pii_type, PiiType::Ssn);
        assert_eq!(result.violations[0].original_text, "123-45-6789");
        assert_eq!(result.violations[0].required_format, "XXX-XX-6789");
    }

    #[test]
    fn test_ssn_with_spaces_detected() {
        let e = engine();
        let result = e.scan("SSN: 123 45 6789 on file", "motion").unwrap();

        assert!(!result.clean);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pii_type, PiiType::Ssn);
        assert_eq!(result.violations[0].original_text, "123 45 6789");
    }

    #[test]
    fn test_properly_redacted_ssn_passes() {
        let e = engine();
        let result = e.scan("SSN: XXX-XX-6789 is on file", "motion").unwrap();

        assert!(result.clean, "Properly redacted SSN should pass");
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_case_number_not_false_positive() {
        let e = engine();
        let result = e.scan("Case 5:24-cv-05001 filed in district court", "motion").unwrap();

        assert!(
            result.clean,
            "Case number 5:24-cv-05001 should not be flagged as SSN"
        );
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_criminal_case_number_not_false_positive() {
        let e = engine();
        let result = e.scan("Case 3:23-cr-00123 pending", "motion").unwrap();

        assert!(
            result.clean,
            "Criminal case number should not be flagged"
        );
    }

    // --- Date of Birth Detection Tests ---

    #[test]
    fn test_dob_with_born_keyword_detected() {
        let e = engine();
        let result = e.scan("Defendant was born January 15, 1990 in Arkansas", "motion").unwrap();

        assert!(!result.clean, "DOB after 'born' keyword should be flagged");
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pii_type, PiiType::DateOfBirth);
        assert!(result.violations[0].required_format.contains("1990"));
    }

    #[test]
    fn test_dob_with_dob_keyword_detected() {
        let e = engine();
        let result = e.scan("DOB: 03/15/1985 listed on form", "motion").unwrap();

        assert!(!result.clean);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pii_type, PiiType::DateOfBirth);
        assert!(result.violations[0].required_format.contains("1985"));
    }

    #[test]
    fn test_dob_with_date_of_birth_keyword_detected() {
        let e = engine();
        let result = e.scan("Date of birth: 1990-06-20 confirmed", "motion").unwrap();

        assert!(!result.clean);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pii_type, PiiType::DateOfBirth);
    }

    #[test]
    fn test_date_without_born_not_flagged() {
        let e = engine();
        let result = e.scan("Filed on January 15, 1990 in the Eastern District", "motion").unwrap();

        assert!(
            result.clean,
            "Dates without DOB keywords should not be flagged"
        );
    }

    #[test]
    fn test_procedural_date_not_flagged() {
        let e = engine();
        let result = e.scan("The hearing was scheduled for 01/15/2025 at 9:00 AM", "motion").unwrap();

        assert!(result.clean, "Procedural dates should not be flagged as DOB");
    }

    // --- Financial Account Detection Tests ---

    #[test]
    fn test_financial_account_in_context_detected() {
        let e = engine();
        let result = e.scan("Bank account number 12345678901234 was garnished", "motion").unwrap();

        assert!(!result.clean, "Financial account number should be flagged");
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pii_type, PiiType::FinancialAccount);
        assert!(result.violations[0].required_format.ends_with("1234"));
    }

    #[test]
    fn test_phone_number_not_account() {
        let e = engine();
        let result = e.scan("Contact phone (479) 555-1234 for details", "motion").unwrap();

        assert!(
            result.clean,
            "Phone number should not be flagged as financial account"
        );
    }

    #[test]
    fn test_digits_without_financial_context_not_flagged() {
        let e = engine();
        let result = e.scan("Reference number 12345678 in the order", "motion").unwrap();

        assert!(
            result.clean,
            "Digit sequences without financial context should not be flagged"
        );
    }

    // --- Taxpayer ID Detection Tests ---

    #[test]
    fn test_taxpayer_id_with_ein_context_detected() {
        let e = engine();
        let result = e.scan("Company EIN: 12-3456789 on tax return", "motion").unwrap();

        assert!(!result.clean, "Taxpayer ID with EIN context should be flagged");
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].pii_type, PiiType::TaxpayerId);
    }

    #[test]
    fn test_taxpayer_id_without_context_not_flagged() {
        let e = engine();
        let result = e.scan("Document reference 12-3456789 approved", "motion").unwrap();

        assert!(
            result.clean,
            "Number matching TIN format without tax context should not be flagged"
        );
    }

    // --- Restricted Document Type Tests ---

    #[test]
    fn test_psi_report_auto_restricted() {
        let e = engine();
        let result = e.scan("This is clean text with no PII", "presentence investigation").unwrap();

        assert!(!result.clean, "PSI reports should not be clean even without PII");
        assert!(result.restricted);
        assert!(result.restriction_reason.is_some());
        assert!(
            result.restriction_reason.unwrap().contains("Presentence"),
            "Restriction reason should mention presentence"
        );
    }

    #[test]
    fn test_sealed_document_restricted() {
        let e = engine();
        let result = e.scan("Clean text", "sealed order").unwrap();

        assert!(result.restricted);
        assert!(!result.clean);
    }

    #[test]
    fn test_warrant_restricted() {
        let e = engine();
        assert!(e.is_restricted_document_type("arrest warrant"));
        assert!(e.is_restricted_document_type("search warrant"));
        assert!(e.is_restricted_document_type("summons"));
    }

    #[test]
    fn test_cja_affidavit_restricted() {
        let e = engine();
        assert!(e.is_restricted_document_type("CJA Financial Affidavit"));
        assert!(e.is_restricted_document_type("cja affidavit"));
    }

    #[test]
    fn test_juvenile_restricted() {
        let e = engine();
        assert!(e.is_restricted_document_type("juvenile proceeding"));
    }

    #[test]
    fn test_juror_restricted() {
        let e = engine();
        assert!(e.is_restricted_document_type("juror questionnaire"));
    }

    #[test]
    fn test_statement_of_reasons_restricted() {
        let e = engine();
        assert!(e.is_restricted_document_type("statement of reasons"));
        assert!(e.is_restricted_document_type("SOR"));
    }

    #[test]
    fn test_regular_motion_not_restricted() {
        let e = engine();
        assert!(!e.is_restricted_document_type("motion to dismiss"));
        assert!(!e.is_restricted_document_type("brief"));
        assert!(!e.is_restricted_document_type("complaint"));
    }

    #[test]
    fn test_get_restricted_types_returns_all_seven() {
        let e = engine();
        let types = e.get_restricted_types();
        assert_eq!(types.len(), 7);
    }

    // --- Combined / Multiple PII Tests ---

    #[test]
    fn test_multiple_pii_types_all_reported() {
        let e = engine();
        let text = "Defendant born 03/15/1990 has SSN 123-45-6789 on record";
        let result = e.scan(text, "motion").unwrap();

        assert!(!result.clean);
        assert_eq!(result.violations.len(), 2, "Should detect both SSN and DOB");

        let has_ssn = result.violations.iter().any(|v| v.pii_type == PiiType::Ssn);
        let has_dob = result.violations.iter().any(|v| v.pii_type == PiiType::DateOfBirth);

        assert!(has_ssn, "Should detect SSN violation");
        assert!(has_dob, "Should detect DOB violation");
    }

    #[test]
    fn test_multiple_ssns_all_detected() {
        let e = engine();
        let text = "Plaintiff SSN 111-22-3333 and defendant SSN 444-55-6666";
        let result = e.scan(text, "motion").unwrap();

        assert_eq!(result.violations.len(), 2, "Should detect both SSNs");
    }

    #[test]
    fn test_clean_document_passes() {
        let e = engine();
        let text = "This motion requests summary judgment based on the evidence presented.";
        let result = e.scan(text, "motion").unwrap();

        assert!(result.clean);
        assert!(result.violations.is_empty());
        assert!(!result.restricted);
        assert!(result.restriction_reason.is_none());
    }

    #[test]
    fn test_restricted_with_pii_reports_both() {
        let e = engine();
        let text = "Defendant SSN is 123-45-6789";
        let result = e.scan(text, "presentence investigation").unwrap();

        assert!(!result.clean);
        assert!(result.restricted, "Should be restricted");
        assert!(!result.violations.is_empty(), "Should also report PII violations");
    }

    // --- Violation position tracking ---

    #[test]
    fn test_violation_positions_are_accurate() {
        let e = engine();
        let text = "SSN is 123-45-6789 here";
        let result = e.scan(text, "motion").unwrap();

        assert_eq!(result.violations.len(), 1);
        let v = &result.violations[0];

        // Verify the position points to the actual SSN in the text
        let extracted = &text[v.start_position..v.end_position];
        assert_eq!(extracted, "123-45-6789");
    }
}
