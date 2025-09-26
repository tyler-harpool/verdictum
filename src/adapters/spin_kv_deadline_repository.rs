//! Spin Key-Value Store implementation for deadline repository
//!
//! This adapter implements the DeadlineRepository traits using Spin's
//! built-in key-value store for persistence.

use crate::adapters::store_utils::open_validated_store;
use crate::domain::deadline::{
    Deadline, DeadlineType, DeadlineStatus, ExtensionRequest,
    ExtensionStatus, DeadlineReminder, DeadlineMonitor
};
use crate::ports::deadline_repository::{
    DeadlineRepository, ExtensionRepository, ReminderRepository,
    DeadlineQuery, DeadlineComplianceRepository, ComplianceStatistics,
    ComplianceReport, PerformanceMetrics
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use spin_sdk::key_value::Store;
use uuid::Uuid;
use std::collections::HashMap;

const DEADLINE_KEY_PREFIX: &str = "deadline-";
const EXTENSION_KEY_PREFIX: &str = "extension-";
const REMINDER_KEY_PREFIX: &str = "reminder-";
const INDEX_KEY_PREFIX: &str = "idx-";

/// Spin KV implementation of the DeadlineRepository
pub struct SpinKvDeadlineRepository {
    store: Store,
}

impl SpinKvDeadlineRepository {
    /// Create repository with specific store name for multi-tenancy
    pub fn with_store(store_name: String) -> Self {
        let store = open_validated_store(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }

    fn build_deadline_key(id: Uuid) -> String {
        format!("{}{}", DEADLINE_KEY_PREFIX, id)
    }

    fn build_extension_key(deadline_id: Uuid, extension_id: Uuid) -> String {
        format!("{}{}-{}", EXTENSION_KEY_PREFIX, deadline_id, extension_id)
    }

    fn build_reminder_key(id: Uuid) -> String {
        format!("{}{}", REMINDER_KEY_PREFIX, id)
    }

    fn build_case_deadline_index_key(case_id: Uuid) -> String {
        format!("{}case-deadline-{}", INDEX_KEY_PREFIX, case_id)
    }
}

impl DeadlineRepository for SpinKvDeadlineRepository {
    fn save_deadline(&self, deadline: &Deadline) -> Result<()> {        let key = Self::build_deadline_key(deadline.id);
        self.store.set_json(&key, deadline)?;

        // Update case deadline index
        let index_key = Self::build_case_deadline_index_key(deadline.case_id);
        let mut deadlines = self.find_deadlines_by_case(deadline.case_id)?;
        if !deadlines.iter().any(|d| d.id == deadline.id) {
            deadlines.push(deadline.clone());
        }
        self.store.set_json(&index_key, &deadlines)?;

        Ok(())
    }

    fn find_deadline_by_id(&self, id: Uuid) -> Result<Option<Deadline>> {        let key = Self::build_deadline_key(id);
        self.store.get_json::<Deadline>(&key)
    }

    fn find_deadlines_by_case(&self, case_id: Uuid) -> Result<Vec<Deadline>> {        let index_key = Self::build_case_deadline_index_key(case_id);

        match self.store.get_json::<Vec<Deadline>>(&index_key)? {
            Some(deadlines) => Ok(deadlines),
            None => Ok(Vec::new()),
        }
    }

    fn find_deadlines_by_type(&self, case_id: Uuid, deadline_type: DeadlineType) -> Result<Vec<Deadline>> {
        let deadlines = self.find_deadlines_by_case(case_id)?;
        Ok(deadlines.into_iter().filter(|d| {
            std::mem::discriminant(&d.deadline_type) == std::mem::discriminant(&deadline_type)
        }).collect())
    }

    fn find_deadlines_by_status(&self, status: DeadlineStatus) -> Result<Vec<Deadline>> {
        let deadlines: Vec<Deadline> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
            .filter_map(|deadline| deadline)
            .filter(|d| d.status == status)
            .collect();

        Ok(deadlines)
    }

    fn find_deadlines_by_party(&self, party_name: &str) -> Result<Vec<Deadline>> {        let party_lower = party_name.to_lowercase();

        let deadlines: Vec<Deadline> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
            .filter_map(|deadline| deadline)
            .filter(|d| d.responsible_party.to_lowercase().contains(&party_lower))
            .collect();

        Ok(deadlines)
    }

    fn find_upcoming_deadlines(&self, days_ahead: i64) -> Result<Vec<Deadline>> {        let cutoff_date = Utc::now() + chrono::Duration::days(days_ahead);

        let deadlines: Vec<Deadline> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
            .filter_map(|deadline| deadline)
            .filter(|d| d.due_date <= cutoff_date && d.due_date >= Utc::now())
            .collect();

        Ok(deadlines)
    }

    fn update_deadline_status(&self, id: Uuid, status: DeadlineStatus) -> Result<()> {
        if let Some(mut deadline) = self.find_deadline_by_id(id)? {
            deadline.status = status;
            self.save_deadline(&deadline)?;
        }
        Ok(())
    }

    fn complete_deadline(&self, id: Uuid, completion_date: DateTime<Utc>) -> Result<()> {
        if let Some(mut deadline) = self.find_deadline_by_id(id)? {
            deadline.status = DeadlineStatus::Completed;
            deadline.completion_date = Some(completion_date);
            self.save_deadline(&deadline)?;
        }
        Ok(())
    }

    fn delete_deadline(&self, id: Uuid) -> Result<bool> {        let key = Self::build_deadline_key(id);

        let exists = self.store.exists(&key)?;
        if exists {
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl ExtensionRepository for SpinKvDeadlineRepository {
    fn save_extension(&self, deadline_id: Uuid, extension: &ExtensionRequest) -> Result<()> {        let key = Self::build_extension_key(deadline_id, extension.id);
        self.store.set_json(&key, extension)?;

        // Update deadline with extension
        if let Some(mut deadline) = self.find_deadline_by_id(deadline_id)? {
            if !deadline.extension_requests.iter().any(|e| e.id == extension.id) {
                deadline.extension_requests.push(extension.clone());
                self.save_deadline(&deadline)?;
            }
        }

        Ok(())
    }

    fn find_extension_by_id(&self, id: Uuid) -> Result<Option<ExtensionRequest>> {
        // Search through all extensions
        for key in self.store.get_keys()? {
            if key.starts_with(EXTENSION_KEY_PREFIX) {
                if let Ok(Some(extension)) = self.store.get_json::<ExtensionRequest>(&key) {
                    if extension.id == id {
                        return Ok(Some(extension));
                    }
                }
            }
        }

        Ok(None)
    }

    fn find_extensions_by_deadline(&self, deadline_id: Uuid) -> Result<Vec<ExtensionRequest>> {
        if let Ok(Some(deadline)) = self.find_deadline_by_id(deadline_id) {
            Ok(deadline.extension_requests)
        } else {
            Ok(Vec::new())
        }
    }

    fn find_pending_extensions(&self) -> Result<Vec<(Uuid, ExtensionRequest)>> {        let mut pending = Vec::new();

        for key in self.store.get_keys()? {
            if key.starts_with(DEADLINE_KEY_PREFIX) {
                if let Ok(Some(deadline)) = self.store.get_json::<Deadline>(&key) {
                    for extension in &deadline.extension_requests {
                        if extension.status == ExtensionStatus::Pending {
                            pending.push((deadline.id, extension.clone()));
                        }
                    }
                }
            }
        }

        Ok(pending)
    }

    fn update_extension_status(&self, id: Uuid, status: ExtensionStatus) -> Result<()> {
        // Find and update the extension
        for key in self.store.get_keys()? {
            if key.starts_with(DEADLINE_KEY_PREFIX) {
                if let Ok(Some(mut deadline)) = self.store.get_json::<Deadline>(&key) {
                    for extension in &mut deadline.extension_requests {
                        if extension.id == id {
                            extension.status = status;
                            extension.ruling_date = Some(Utc::now());
                            self.save_deadline(&deadline)?;
                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl ReminderRepository for SpinKvDeadlineRepository {
    fn save_reminders(&self, reminders: &[DeadlineReminder]) -> Result<()> {
        for reminder in reminders {
            let key = Self::build_reminder_key(Uuid::new_v4());
            self.store.set_json(&key, reminder)?;

            // Add to deadline's reminder history
            if let Ok(Some(mut deadline)) = self.find_deadline_by_id(reminder.deadline_id) {
                deadline.reminders_sent.push(Utc::now());
                self.save_deadline(&deadline)?;
            }
        }

        Ok(())
    }

    fn find_reminders_by_deadline(&self, deadline_id: Uuid) -> Result<Vec<DeadlineReminder>> {
        let reminders: Vec<DeadlineReminder> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(REMINDER_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<DeadlineReminder>(key.as_str()).ok())
            .filter_map(|reminder| reminder)
            .filter(|r| r.deadline_id == deadline_id)
            .collect();

        Ok(reminders)
    }

    fn find_reminders_by_recipient(&self, recipient: &str) -> Result<Vec<DeadlineReminder>> {        let recipient_lower = recipient.to_lowercase();

        let reminders: Vec<DeadlineReminder> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(REMINDER_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<DeadlineReminder>(key.as_str()).ok())
            .filter_map(|reminder| reminder)
            .filter(|r| r.recipient.to_lowercase() == recipient_lower)
            .collect();

        Ok(reminders)
    }

    fn acknowledge_reminder(&self, reminder_id: Uuid) -> Result<()> {
        // In a real system, this would mark the reminder as acknowledged
        // For now, we'll just delete it
        let key = Self::build_reminder_key(reminder_id);
        self.store.delete(&key)?;
        Ok(())
    }

    fn get_pending_reminders(&self) -> Result<Vec<DeadlineReminder>> {        let mut all_deadlines = Vec::new();

        for key in self.store.get_keys()? {
            if key.starts_with(DEADLINE_KEY_PREFIX) {
                if let Ok(Some(deadline)) = self.store.get_json::<Deadline>(&key) {
                    all_deadlines.push(deadline);
                }
            }
        }

        Ok(DeadlineMonitor::generate_reminders(&all_deadlines, Utc::now()))
    }
}

impl DeadlineComplianceRepository for SpinKvDeadlineRepository {
    fn search_deadlines(&self, query: DeadlineQuery) -> Result<(Vec<Deadline>, usize)> {
        let mut deadlines: Vec<Deadline> = if let Some(case_id) = query.case_id {
            self.find_deadlines_by_case(case_id)?
        } else {
            self.store
                .get_keys()?
                .iter()
                .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
                .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
                .filter_map(|deadline| deadline)
                .collect()
        };

        // Apply filters
        if let Some(deadline_type) = query.deadline_type {
            deadlines.retain(|d| std::mem::discriminant(&d.deadline_type) == std::mem::discriminant(&deadline_type));
        }

        if let Some(status) = query.status {
            deadlines.retain(|d| d.status == status);
        }

        if let Some(party) = query.responsible_party {
            deadlines.retain(|d| d.responsible_party.to_lowercase().contains(&party.to_lowercase()));
        }

        if let Some(jurisdictional) = query.is_jurisdictional {
            deadlines.retain(|d| d.is_jurisdictional == jurisdictional);
        }

        if let Some(date_from) = query.due_date_from {
            deadlines.retain(|d| d.due_date >= date_from);
        }

        if let Some(date_to) = query.due_date_to {
            deadlines.retain(|d| d.due_date <= date_to);
        }

        // Sort by due date
        deadlines.sort_by_key(|d| d.due_date);

        // Get total count before pagination
        let total = deadlines.len();

        // Apply pagination
        let paginated: Vec<Deadline> = deadlines
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok((paginated, total))
    }

    fn get_compliance_statistics(&self, case_id: Option<Uuid>) -> Result<ComplianceStatistics> {
        let deadlines = if let Some(id) = case_id {
            self.find_deadlines_by_case(id)?
        } else {            self.store
                .get_keys()?
                .iter()
                .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
                .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
                .filter_map(|deadline| deadline)
                .collect()
        };

        let total = deadlines.len();
        let completed_on_time = deadlines.iter().filter(|d| {
            d.status == DeadlineStatus::Completed &&
            d.completion_date.map_or(false, |cd| cd <= d.due_date)
        }).count();

        let completed_late = deadlines.iter().filter(|d| {
            d.status == DeadlineStatus::Completed &&
            d.completion_date.map_or(false, |cd| cd > d.due_date)
        }).count();

        let pending = deadlines.iter().filter(|d| d.status == DeadlineStatus::Pending || d.status == DeadlineStatus::Approaching).count();
        let overdue = deadlines.iter().filter(|d| d.status == DeadlineStatus::Overdue).count();
        let extended = deadlines.iter().filter(|d| d.status == DeadlineStatus::Extended).count();
        let waived = deadlines.iter().filter(|d| d.status == DeadlineStatus::Waived).count();

        let compliance_rate = if total > 0 {
            (completed_on_time as f32 / total as f32) * 100.0
        } else {
            100.0
        };

        let average_days_early: f32 = deadlines.iter()
            .filter_map(|d| {
                if d.status == DeadlineStatus::Completed {
                    d.completion_date.map(|cd| (d.due_date - cd).num_days() as f32)
                } else {
                    None
                }
            })
            .sum::<f32>() / completed_on_time.max(1) as f32;

        Ok(ComplianceStatistics {
            total_deadlines: total,
            completed_on_time,
            completed_late,
            pending,
            overdue,
            extended,
            waived,
            compliance_rate,
            average_days_early,
        })
    }

    fn find_missed_jurisdictional(&self) -> Result<Vec<Deadline>> {
        let deadlines: Vec<Deadline> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
            .filter_map(|deadline| deadline)
            .filter(|d| d.is_jurisdictional && d.status == DeadlineStatus::Overdue)
            .collect();

        Ok(deadlines)
    }

    fn generate_compliance_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<ComplianceReport> {
        let deadlines: Vec<Deadline> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
            .filter_map(|deadline| deadline)
            .filter(|d| d.due_date >= start_date && d.due_date <= end_date)
            .collect();

        let total_cases = deadlines.iter().map(|d| d.case_id).collect::<std::collections::HashSet<_>>().len();
        let deadlines_tracked = deadlines.len();

        let completed_on_time = deadlines.iter().filter(|d| {
            d.status == DeadlineStatus::Completed &&
            d.completion_date.map_or(false, |cd| cd <= d.due_date)
        }).count();

        let compliance_rate = if deadlines_tracked > 0 {
            (completed_on_time as f32 / deadlines_tracked as f32) * 100.0
        } else {
            100.0
        };

        let jurisdictional_violations = deadlines.iter()
            .filter(|d| d.is_jurisdictional && d.status == DeadlineStatus::Overdue)
            .count();

        // Find most missed deadline type
        let mut type_misses: HashMap<String, usize> = HashMap::new();
        for deadline in &deadlines {
            if deadline.status == DeadlineStatus::Overdue {
                let type_str = format!("{:?}", deadline.deadline_type);
                *type_misses.entry(type_str).or_insert(0) += 1;
            }
        }

        let most_missed_type = type_misses
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(dtype, _)| dtype)
            .unwrap_or_else(|| "None".to_string());

        // Find parties with violations
        let mut party_violations: HashMap<String, usize> = HashMap::new();
        for deadline in &deadlines {
            if deadline.status == DeadlineStatus::Overdue {
                *party_violations.entry(deadline.responsible_party.clone()).or_insert(0) += 1;
            }
        }

        let parties_with_violations: Vec<String> = party_violations.into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(party, _)| party)
            .collect();

        Ok(ComplianceReport {
            period_start: start_date,
            period_end: end_date,
            total_cases,
            deadlines_tracked,
            compliance_rate,
            jurisdictional_violations,
            most_missed_type,
            parties_with_violations,
        })
    }

    fn get_performance_metrics(&self, party_name: Option<String>) -> Result<PerformanceMetrics> {
        let deadlines = if let Some(party) = &party_name {
            self.find_deadlines_by_party(party)?
        } else {            self.store
                .get_keys()?
                .iter()
                .filter(|key| key.starts_with(DEADLINE_KEY_PREFIX))
                .filter_map(|key| self.store.get_json::<Deadline>(key.as_str()).ok())
                .filter_map(|deadline| deadline)
                .collect()
        };

        let total = deadlines.len();
        let on_time = deadlines.iter().filter(|d| {
            d.status == DeadlineStatus::Completed &&
            d.completion_date.map_or(false, |cd| cd <= d.due_date)
        }).count();

        let on_time_percentage = if total > 0 {
            (on_time as f32 / total as f32) * 100.0
        } else {
            100.0
        };

        let total_response_days: i64 = deadlines.iter()
            .filter_map(|d| {
                d.completion_date.map(|cd| (cd - d.triggering_date).num_days())
            })
            .sum();

        let completed_count = deadlines.iter().filter(|d| d.completion_date.is_some()).count();
        let average_response_days = if completed_count > 0 {
            total_response_days as f32 / completed_count as f32
        } else {
            0.0
        };

        let extension_requests = deadlines.iter()
            .map(|d| d.extension_requests.len())
            .sum();

        let violations = deadlines.iter()
            .filter(|d| d.status == DeadlineStatus::Overdue)
            .count();

        // Simple trending logic
        let trending = if violations > 0 {
            "declining".to_string()
        } else if on_time_percentage > 90.0 {
            "improving".to_string()
        } else {
            "stable".to_string()
        };

        Ok(PerformanceMetrics {
            entity: party_name.unwrap_or_else(|| "Overall".to_string()),
            total_deadlines: total,
            on_time_percentage,
            average_response_days,
            extension_requests,
            violations,
            trending,
        })
    }
}