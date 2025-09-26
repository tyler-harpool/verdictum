//! Spin Key-Value Store implementation for judge repository
//!
//! This adapter implements the JudgeRepository traits using Spin's
//! built-in key-value store for persistence.

use crate::adapters::store_utils::open_validated_store;
use crate::domain::judge::{
    Judge, CaseAssignment, RecusalMotion, JudgeStatus,
    ConflictOfInterest, RecusalStatus
};
use crate::ports::judge_repository::{
    JudgeRepository, CaseAssignmentRepository, RecusalRepository,
    ConflictRepository, JudgeQuery, JudgeQueryRepository, WorkloadStatistics
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use spin_sdk::key_value::Store;
use uuid::Uuid;

const JUDGE_KEY_PREFIX: &str = "judge-";
const ASSIGNMENT_KEY_PREFIX: &str = "assignment-";
const RECUSAL_KEY_PREFIX: &str = "recusal-";
const CONFLICT_KEY_PREFIX: &str = "conflict-";
const INDEX_KEY_PREFIX: &str = "idx-";

/// Spin KV implementation of the JudgeRepository
pub struct SpinKvJudgeRepository {
    store: Store,
}

impl SpinKvJudgeRepository {
    /// Create repository with specific store name for multi-tenancy
    pub fn with_store(store_name: String) -> Self {
        let store = open_validated_store(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }

    fn build_judge_key(id: Uuid) -> String {
        format!("{}{}", JUDGE_KEY_PREFIX, id)
    }

    fn build_assignment_key(id: Uuid) -> String {
        format!("{}{}", ASSIGNMENT_KEY_PREFIX, id)
    }

    fn build_recusal_key(id: Uuid) -> String {
        format!("{}{}", RECUSAL_KEY_PREFIX, id)
    }

    fn build_conflict_key(judge_id: Uuid, conflict_id: Uuid) -> String {
        format!("{}{}-{}", CONFLICT_KEY_PREFIX, judge_id, conflict_id)
    }

    fn build_case_assignment_index_key(case_id: Uuid) -> String {
        format!("{}case-assignment-{}", INDEX_KEY_PREFIX, case_id)
    }

    fn build_judge_assignment_index_key(judge_id: Uuid) -> String {
        format!("{}judge-assignments-{}", INDEX_KEY_PREFIX, judge_id)
    }
}

impl JudgeRepository for SpinKvJudgeRepository {
    fn save_judge(&self, judge: &Judge) -> Result<()> {        let key = Self::build_judge_key(judge.id);
        self.store.set_json(&key, judge)?;
        Ok(())
    }

    fn find_judge_by_id(&self, id: Uuid) -> Result<Option<Judge>> {        let key = Self::build_judge_key(id);
        self.store.get_json::<Judge>(&key)
    }

    fn find_all_judges(&self) -> Result<Vec<Judge>> {
        let judges: Vec<Judge> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(JUDGE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<Judge>(key.as_str()).ok())
            .filter_map(|judge| judge)
            .collect();

        Ok(judges)
    }

    fn find_judges_by_status(&self, status: JudgeStatus) -> Result<Vec<Judge>> {
        let judges = self.find_all_judges()?;
        Ok(judges.into_iter().filter(|j| j.status == status).collect())
    }

    fn find_judges_by_district(&self, district: &str) -> Result<Vec<Judge>> {
        let judges = self.find_all_judges()?;
        Ok(judges
            .into_iter()
            .filter(|j| j.district.to_lowercase() == district.to_lowercase())
            .collect())
    }

    fn find_available_judges(&self) -> Result<Vec<Judge>> {
        let judges = self.find_all_judges()?;
        Ok(judges
            .into_iter()
            .filter(|j| j.can_accept_new_cases())
            .collect())
    }

    fn delete_judge(&self, id: Uuid) -> Result<bool> {        let key = Self::build_judge_key(id);

        let exists = self.store.exists(&key)?;
        if exists {
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CaseAssignmentRepository for SpinKvJudgeRepository {
    fn save_assignment(&self, assignment: &CaseAssignment) -> Result<()> {
        // Save the assignment
        let key = Self::build_assignment_key(assignment.id);
        self.store.set_json(&key, assignment)?;

        // Update case index
        let case_index_key = Self::build_case_assignment_index_key(assignment.case_id);
        self.store.set(&case_index_key, assignment.id.to_string().as_bytes())?;

        // Update judge assignments index
        let judge_index_key = Self::build_judge_assignment_index_key(assignment.judge_id);
        let mut assignments = self.find_assignments_by_judge(assignment.judge_id)?;
        assignments.push(assignment.clone());
        self.store.set_json(&judge_index_key, &assignments)?;

        Ok(())
    }

    fn find_assignment_by_case(&self, case_id: Uuid) -> Result<Option<CaseAssignment>> {        let index_key = Self::build_case_assignment_index_key(case_id);

        match self.store.get(&index_key)? {
            Some(id_bytes) => {
                let id_str = String::from_utf8(id_bytes)?;
                let assignment_id = Uuid::parse_str(&id_str)?;
                let assignment_key = Self::build_assignment_key(assignment_id);
                self.store.get_json::<CaseAssignment>(&assignment_key)
            }
            None => Ok(None),
        }
    }

    fn find_assignments_by_judge(&self, judge_id: Uuid) -> Result<Vec<CaseAssignment>> {        let index_key = Self::build_judge_assignment_index_key(judge_id);

        match self.store.get_json::<Vec<CaseAssignment>>(&index_key)? {
            Some(assignments) => Ok(assignments),
            None => Ok(Vec::new()),
        }
    }

    fn find_assignment_history(&self, case_id: Uuid) -> Result<Vec<CaseAssignment>> {
        let assignments: Vec<CaseAssignment> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(ASSIGNMENT_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<CaseAssignment>(key.as_str()).ok())
            .filter_map(|assignment| assignment)
            .filter(|a| a.case_id == case_id)
            .collect();

        Ok(assignments)
    }

    fn delete_assignment(&self, id: Uuid) -> Result<bool> {        let key = Self::build_assignment_key(id);

        let exists = self.store.exists(&key)?;
        if exists {
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl RecusalRepository for SpinKvJudgeRepository {
    fn save_recusal(&self, motion: &RecusalMotion) -> Result<()> {        let key = Self::build_recusal_key(motion.id);
        self.store.set_json(&key, motion)?;
        Ok(())
    }

    fn find_recusal_by_id(&self, id: Uuid) -> Result<Option<RecusalMotion>> {        let key = Self::build_recusal_key(id);
        self.store.get_json::<RecusalMotion>(&key)
    }

    fn find_recusals_by_case(&self, case_id: Uuid) -> Result<Vec<RecusalMotion>> {
        let recusals: Vec<RecusalMotion> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(RECUSAL_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<RecusalMotion>(key.as_str()).ok())
            .filter_map(|recusal| recusal)
            .filter(|r| r.case_id == case_id)
            .collect();

        Ok(recusals)
    }

    fn find_recusals_by_judge(&self, judge_id: Uuid) -> Result<Vec<RecusalMotion>> {
        let recusals: Vec<RecusalMotion> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(RECUSAL_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<RecusalMotion>(key.as_str()).ok())
            .filter_map(|recusal| recusal)
            .filter(|r| r.judge_id == judge_id)
            .collect();

        Ok(recusals)
    }

    fn find_pending_recusals(&self) -> Result<Vec<RecusalMotion>> {
        let recusals: Vec<RecusalMotion> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(RECUSAL_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<RecusalMotion>(key.as_str()).ok())
            .filter_map(|recusal| recusal)
            .filter(|r| r.status == RecusalStatus::Pending)
            .collect();

        Ok(recusals)
    }
}

impl ConflictRepository for SpinKvJudgeRepository {
    fn save_conflict(&self, judge_id: Uuid, conflict: &ConflictOfInterest) -> Result<()> {        let key = Self::build_conflict_key(judge_id, conflict.id);

        // Save the conflict
        self.store.set_json(&key, conflict)?;

        // Update judge's conflict list
        if let Ok(Some(mut judge)) = self.find_judge_by_id(judge_id) {
            judge.add_conflict(conflict.clone());
            self.save_judge(&judge)?;
        }

        Ok(())
    }

    fn find_conflicts_by_judge(&self, judge_id: Uuid) -> Result<Vec<ConflictOfInterest>> {
        if let Ok(Some(judge)) = self.find_judge_by_id(judge_id) {
            Ok(judge.conflicts_of_interest)
        } else {
            Ok(Vec::new())
        }
    }

    fn find_conflicts_by_party(&self, party_name: &str) -> Result<Vec<(Uuid, ConflictOfInterest)>> {
        let judges = self.find_all_judges()?;
        let mut conflicts = Vec::new();

        for judge in judges {
            for conflict in &judge.conflicts_of_interest {
                if let Some(ref name) = conflict.party_name {
                    if name.to_lowercase().contains(&party_name.to_lowercase()) {
                        conflicts.push((judge.id, conflict.clone()));
                    }
                }
            }
        }

        Ok(conflicts)
    }

    fn has_conflict(&self, judge_id: Uuid, party_name: &str) -> Result<bool> {
        if let Ok(Some(judge)) = self.find_judge_by_id(judge_id) {
            Ok(judge.has_conflict_with(party_name))
        } else {
            Ok(false)
        }
    }

    fn delete_conflict(&self, judge_id: Uuid, conflict_id: Uuid) -> Result<bool> {        let key = Self::build_conflict_key(judge_id, conflict_id);

        // Remove from judge's conflict list
        if let Ok(Some(mut judge)) = self.find_judge_by_id(judge_id) {
            judge.conflicts_of_interest.retain(|c| c.id != conflict_id);
            self.save_judge(&judge)?;
        }

        let exists = self.store.exists(&key)?;
        if exists {
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl JudgeQueryRepository for SpinKvJudgeRepository {
    fn search_judges(&self, query: JudgeQuery) -> Result<(Vec<Judge>, usize)> {
        let mut judges = self.find_all_judges()?;

        // Apply filters
        if let Some(status) = query.status {
            judges.retain(|j| j.status == status);
        }

        if let Some(title) = query.title {
            judges.retain(|j| matches!(j.title, ref t if *t == title));
        }

        if let Some(district) = query.district {
            judges.retain(|j| j.district.to_lowercase().contains(&district.to_lowercase()));
        }

        if let Some(accepts_criminal) = query.accepts_criminal {
            judges.retain(|j| j.availability.accepts_criminal_cases == accepts_criminal);
        }

        if let Some(accepts_civil) = query.accepts_civil {
            judges.retain(|j| j.availability.accepts_civil_cases == accepts_civil);
        }

        if let Some(max_percentage) = query.max_caseload_percentage {
            judges.retain(|j| {
                let percentage = (j.current_caseload as f32 / j.max_caseload as f32) * 100.0;
                percentage <= max_percentage
            });
        }

        // Sort by caseload (ascending)
        judges.sort_by_key(|j| j.current_caseload);

        // Get total count before pagination
        let total = judges.len();

        // Apply pagination
        let paginated: Vec<Judge> = judges
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok((paginated, total))
    }

    fn get_workload_statistics(&self) -> Result<WorkloadStatistics> {
        let judges = self.find_all_judges()?;

        let active_judges = judges.iter().filter(|j| j.status == JudgeStatus::Active).count();
        let senior_judges = judges.iter().filter(|j| j.status == JudgeStatus::Senior).count();

        let total_cases: u32 = judges.iter().map(|j| j.current_caseload).sum();
        let average_caseload = if !judges.is_empty() {
            total_cases as f32 / judges.len() as f32
        } else {
            0.0
        };

        let overloaded_judges = judges.iter().filter(|j| {
            (j.current_caseload as f32 / j.max_caseload as f32) > 0.9
        }).count();

        let available_capacity: u32 = judges
            .iter()
            .map(|j| j.max_caseload.saturating_sub(j.current_caseload))
            .sum();

        Ok(WorkloadStatistics {
            total_judges: judges.len(),
            active_judges,
            senior_judges,
            average_caseload,
            total_cases: total_cases as usize,
            overloaded_judges,
            available_capacity: available_capacity as usize,
        })
    }

    fn find_judges_on_vacation(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<Judge>> {
        let judges = self.find_all_judges()?;

        Ok(judges.into_iter().filter(|j| {
            j.availability.vacation_dates.iter().any(|vacation| {
                vacation.start <= end_date && vacation.end >= start_date
            })
        }).collect())
    }
}