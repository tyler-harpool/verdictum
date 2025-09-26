//! Spin Key-Value Store implementation for criminal case repository
//!
//! This adapter implements the CaseRepository trait using Spin's
//! built-in key-value store for persistence.

use crate::adapters::store_utils::open_validated_store;
use crate::domain::criminal_case::{CaseStatus, CasePriority, CriminalCase};
use crate::ports::case_repository::{CaseRepository, CaseQuery, CaseQueryRepository, CaseStatistics};
use anyhow::Result;
use spin_sdk::key_value::Store;
use uuid::Uuid;

const CASE_KEY_PREFIX: &str = "case-";
const CASE_INDEX_PREFIX: &str = "case-idx-";

/// Spin KV implementation of the CaseRepository
pub struct SpinKvCaseRepository {
    store: Store,
}

impl SpinKvCaseRepository {
    /// Create repository with specific store name for multi-tenancy
    pub fn with_store(store_name: String) -> Self {
        let store = open_validated_store(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }

    /// Build a storage key for a case
    fn build_case_key(id: Uuid) -> String {
        format!("{}{}", CASE_KEY_PREFIX, id)
    }

    /// Build an index key for case number lookup
    fn build_case_number_key(case_number: &str) -> String {
        format!("{}case-num-{}", CASE_INDEX_PREFIX, case_number)
    }

}

impl CaseRepository for SpinKvCaseRepository {
    fn save(&self, case: &CriminalCase) -> Result<()> {

        // Save the case
        let case_key = Self::build_case_key(case.id);
        self.store.set_json(&case_key, case)?;

        // Save case number index
        let case_number_key = Self::build_case_number_key(&case.case_number);
        self.store.set(&case_number_key, case.id.to_string().as_bytes())?;

        Ok(())
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<CriminalCase>> {
        let key = Self::build_case_key(id);
        self.store.get_json::<CriminalCase>(&key)
    }

    fn find_by_case_number(&self, case_number: &str) -> Result<Option<CriminalCase>> {
        let case_number_key = Self::build_case_number_key(case_number);

        // Look up the case ID from the case number index
        match self.store.get(&case_number_key)? {
            Some(id_bytes) => {
                let id_str = String::from_utf8(id_bytes)?;
                let id = Uuid::parse_str(&id_str)?;
                self.find_by_id(id)
            }
            None => Ok(None),
        }
    }

    fn find_all_cases(&self) -> Result<Vec<CriminalCase>> {

        let cases: Vec<CriminalCase> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(CASE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<CriminalCase>(key.as_str()).ok())
            .filter_map(|case| case)
            .collect();

        Ok(cases)
    }

    fn find_by_status(&self, status: CaseStatus) -> Result<Vec<CriminalCase>> {
        let cases = self.find_all_cases()?;
        Ok(cases.into_iter().filter(|c| c.status == status).collect())
    }

    fn find_by_judge(&self, judge: &str) -> Result<Vec<CriminalCase>> {
        let cases = self.find_all_cases()?;
        Ok(cases
            .into_iter()
            .filter(|c| c.assigned_judge.to_lowercase() == judge.to_lowercase())
            .collect())
    }

    fn delete(&self, id: Uuid) -> Result<bool> {
        let key = Self::build_case_key(id);

        // Check if case exists
        let exists = self.store.exists(&key)?;

        if exists {
            // Get the case to find its case number
            if let Ok(Some(case)) = self.store.get_json::<CriminalCase>(&key) {
                // Delete the case number index
                let case_number_key = Self::build_case_number_key(&case.case_number);
                let _ = self.store.delete(&case_number_key);
            }

            // Delete the case
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn count_by_status(&self, status: CaseStatus) -> Result<usize> {
        let cases = self.find_by_status(status)?;
        Ok(cases.len())
    }
}

impl CaseQueryRepository for SpinKvCaseRepository {
    fn search(&self, query: CaseQuery) -> Result<(Vec<CriminalCase>, usize)> {
        let mut cases = self.find_all_cases()?;

        // Apply filters
        if let Some(status) = query.status {
            cases.retain(|c| c.status == status);
        }

        if let Some(priority) = query.priority {
            cases.retain(|c| c.priority == priority);
        }

        if let Some(judge) = query.judge {
            cases.retain(|c| c.assigned_judge.to_lowercase().contains(&judge.to_lowercase()));
        }

        if let Some(is_active) = query.is_active {
            cases.retain(|c| c.is_active() == is_active);
        }

        // Sort by updated_at (most recent first)
        cases.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        // Get total count before pagination
        let total = cases.len();

        // Apply pagination
        let paginated: Vec<CriminalCase> = cases
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok((paginated, total))
    }

    fn get_statistics(&self) -> Result<CaseStatistics> {
        let cases = self.find_all_cases()?;

        let stats = CaseStatistics {
            total_cases: cases.len(),
            open_cases: cases.iter().filter(|c| matches!(c.status, CaseStatus::Filed | CaseStatus::Arraigned)).count(),
            under_investigation: cases.iter().filter(|c| matches!(c.status, CaseStatus::Discovery | CaseStatus::PretrialMotions)).count(),
            closed_cases: cases.iter().filter(|c| matches!(c.status, CaseStatus::Sentenced | CaseStatus::Dismissed)).count(),
            cold_cases: 0, // No longer applicable in federal court context
            critical_priority: cases.iter().filter(|c| c.priority == CasePriority::Critical).count(),
            high_priority: cases.iter().filter(|c| c.priority == CasePriority::High).count(),
        };

        Ok(stats)
    }
}