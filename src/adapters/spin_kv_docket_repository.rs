//! Spin Key-Value Store implementation for docket repository
//!
//! This adapter implements the DocketRepository traits using Spin's
//! built-in key-value store for persistence.

use crate::adapters::store_utils::open_validated_store;
use crate::domain::docket::{
    DocketEntry, CalendarEntry, SpeedyTrialClock, DocketEntryType,
    EventStatus, CalendarService, SpeedyTrialService
};
use crate::ports::docket_repository::{
    DocketRepository, CalendarRepository, SpeedyTrialRepository,
    DocketQuery, CalendarQuery, DocketQueryRepository,
    CalendarSchedulingRepository, FilingStatistics, CourtroomUtilization
};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use spin_sdk::key_value::Store;
use uuid::Uuid;
use std::collections::HashMap;

const DOCKET_KEY_PREFIX: &str = "docket-";
const CALENDAR_KEY_PREFIX: &str = "calendar-";
const SPEEDY_KEY_PREFIX: &str = "speedy-";
const INDEX_KEY_PREFIX: &str = "idx-";

/// Spin KV implementation of the DocketRepository
pub struct SpinKvDocketRepository {
    store: Store,
}

impl SpinKvDocketRepository {
    /// Create repository with specific store name for multi-tenancy
    pub fn with_store(store_name: String) -> Self {
        let store = open_validated_store(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }

    fn build_docket_key(id: Uuid) -> String {
        format!("{}{}", DOCKET_KEY_PREFIX, id)
    }

    fn build_calendar_key(id: Uuid) -> String {
        format!("{}{}", CALENDAR_KEY_PREFIX, id)
    }

    fn build_speedy_key(case_id: Uuid) -> String {
        format!("{}{}", SPEEDY_KEY_PREFIX, case_id)
    }

    fn build_case_docket_index_key(case_id: Uuid) -> String {
        format!("{}case-docket-{}", INDEX_KEY_PREFIX, case_id)
    }

    fn build_entry_number_key(case_id: Uuid) -> String {
        format!("{}entry-number-{}", INDEX_KEY_PREFIX, case_id)
    }
}

impl DocketRepository for SpinKvDocketRepository {
    fn save_entry(&self, entry: &DocketEntry) -> Result<()> {
        // Set entry number if not set
        let mut entry = entry.clone();
        if entry.entry_number == 0 {
            entry.entry_number = self.get_next_entry_number(entry.case_id)?;
        }

        let key = Self::build_docket_key(entry.id);
        self.store.set_json(&key, &entry)?;

        // Update case docket index
        let index_key = Self::build_case_docket_index_key(entry.case_id);
        let mut entries = self.find_entries_by_case(entry.case_id)?;
        entries.push(entry.clone());
        entries.sort_by_key(|e| e.entry_number);
        self.store.set_json(&index_key, &entries)?;

        Ok(())
    }

    fn find_entry_by_id(&self, id: Uuid) -> Result<Option<DocketEntry>> {        let key = Self::build_docket_key(id);
        self.store.get_json::<DocketEntry>(&key)
    }

    fn find_entries_by_case(&self, case_id: Uuid) -> Result<Vec<DocketEntry>> {        let index_key = Self::build_case_docket_index_key(case_id);

        match self.store.get_json::<Vec<DocketEntry>>(&index_key)? {
            Some(entries) => Ok(entries),
            None => Ok(Vec::new()),
        }
    }

    fn find_entries_by_type(&self, case_id: Uuid, entry_type: DocketEntryType) -> Result<Vec<DocketEntry>> {
        let entries = self.find_entries_by_case(case_id)?;
        Ok(entries.into_iter().filter(|e| {
            std::mem::discriminant(&e.entry_type) == std::mem::discriminant(&entry_type)
        }).collect())
    }

    fn get_next_entry_number(&self, case_id: Uuid) -> Result<u32> {        let key = Self::build_entry_number_key(case_id);

        let current = self.store.get(&key)
            .ok()
            .flatten()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let next = current + 1;
        self.store.set(&key, next.to_string().as_bytes())?;

        Ok(next)
    }

    fn find_sealed_entries(&self, case_id: Uuid) -> Result<Vec<DocketEntry>> {
        let entries = self.find_entries_by_case(case_id)?;
        Ok(entries.into_iter().filter(|e| e.is_sealed).collect())
    }

    fn search_entries(&self, case_id: Uuid, search_text: &str) -> Result<Vec<DocketEntry>> {
        let entries = self.find_entries_by_case(case_id)?;
        let search_lower = search_text.to_lowercase();

        Ok(entries.into_iter().filter(|e| {
            e.description.to_lowercase().contains(&search_lower) ||
            e.filed_by.as_ref().map_or(false, |f| f.to_lowercase().contains(&search_lower))
        }).collect())
    }

    fn delete_entry(&self, id: Uuid) -> Result<bool> {        let key = Self::build_docket_key(id);

        let exists = self.store.exists(&key)?;
        if exists {
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CalendarRepository for SpinKvDocketRepository {
    fn save_event(&self, event: &CalendarEntry) -> Result<()> {        let key = Self::build_calendar_key(event.id);
        self.store.set_json(&key, event)?;
        Ok(())
    }

    fn find_event_by_id(&self, id: Uuid) -> Result<Option<CalendarEntry>> {        let key = Self::build_calendar_key(id);
        self.store.get_json::<CalendarEntry>(&key)
    }

    fn find_events_by_case(&self, case_id: Uuid) -> Result<Vec<CalendarEntry>> {
        let events: Vec<CalendarEntry> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(CALENDAR_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<CalendarEntry>(key.as_str()).ok())
            .filter_map(|event| event)
            .filter(|e| e.case_id == case_id)
            .collect();

        Ok(events)
    }

    fn find_events_by_judge(&self, judge_id: Uuid) -> Result<Vec<CalendarEntry>> {
        let events: Vec<CalendarEntry> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(CALENDAR_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<CalendarEntry>(key.as_str()).ok())
            .filter_map(|event| event)
            .filter(|e| e.judge_id == judge_id)
            .collect();

        Ok(events)
    }

    fn find_events_by_courtroom(&self, courtroom: &str, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<CalendarEntry>> {
        let events: Vec<CalendarEntry> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(CALENDAR_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<CalendarEntry>(key.as_str()).ok())
            .filter_map(|event| event)
            .filter(|e| {
                e.courtroom == courtroom &&
                e.scheduled_date >= start_date &&
                e.scheduled_date <= end_date
            })
            .collect();

        Ok(events)
    }

    fn find_events_in_range(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<CalendarEntry>> {
        let events: Vec<CalendarEntry> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(CALENDAR_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<CalendarEntry>(key.as_str()).ok())
            .filter_map(|event| event)
            .filter(|e| e.scheduled_date >= start_date && e.scheduled_date <= end_date)
            .collect();

        Ok(events)
    }

    fn find_conflicts(&self, judge_id: Uuid, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<CalendarEntry>> {
        let events = self.find_events_by_judge(judge_id)?;

        Ok(events.into_iter().filter(|e| {
            let event_end = e.scheduled_date + Duration::minutes(e.duration_minutes as i64);
            e.status != EventStatus::Cancelled &&
            ((start_time >= e.scheduled_date && start_time < event_end) ||
             (end_time > e.scheduled_date && end_time <= event_end) ||
             (start_time <= e.scheduled_date && end_time >= event_end))
        }).collect())
    }

    fn update_event_status(&self, id: Uuid, status: EventStatus) -> Result<()> {
        if let Some(mut event) = self.find_event_by_id(id)? {
            event.status = status;
            self.save_event(&event)?;
        }
        Ok(())
    }

    fn delete_event(&self, id: Uuid) -> Result<bool> {        let key = Self::build_calendar_key(id);

        let exists = self.store.exists(&key)?;
        if exists {
            self.store.delete(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl SpeedyTrialRepository for SpinKvDocketRepository {
    fn save_clock(&self, clock: &SpeedyTrialClock) -> Result<()> {        let key = Self::build_speedy_key(clock.case_id);
        self.store.set_json(&key, clock)?;
        Ok(())
    }

    fn find_clock_by_case(&self, case_id: Uuid) -> Result<Option<SpeedyTrialClock>> {        let key = Self::build_speedy_key(case_id);
        self.store.get_json::<SpeedyTrialClock>(&key)
    }

    fn find_approaching_deadlines(&self, days_threshold: i64) -> Result<Vec<SpeedyTrialClock>> {
        let clocks: Vec<SpeedyTrialClock> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(SPEEDY_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<SpeedyTrialClock>(key.as_str()).ok())
            .filter_map(|clock| clock)
            .filter(|c| !c.waived && c.days_remaining <= days_threshold && c.days_remaining > 0)
            .collect();

        Ok(clocks)
    }

    fn find_violations(&self) -> Result<Vec<SpeedyTrialClock>> {
        let clocks: Vec<SpeedyTrialClock> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(SPEEDY_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<SpeedyTrialClock>(key.as_str()).ok())
            .filter_map(|clock| clock)
            .filter(|c| SpeedyTrialService::is_deadline_violated(c))
            .collect();

        Ok(clocks)
    }

    fn update_clock(&self, _case_id: Uuid, clock: &SpeedyTrialClock) -> Result<()> {
        self.save_clock(clock)
    }
}

impl DocketQueryRepository for SpinKvDocketRepository {
    fn search_docket(&self, query: DocketQuery) -> Result<(Vec<DocketEntry>, usize)> {
        let mut entries = if let Some(case_id) = query.case_id {
            self.find_entries_by_case(case_id)?
        } else {
            self.store
                .get_keys()?
                .iter()
                .filter(|key| key.starts_with(DOCKET_KEY_PREFIX))
                .filter_map(|key| self.store.get_json::<DocketEntry>(key.as_str()).ok())
                .filter_map(|entry| entry)
                .collect()
        };

        // Apply filters
        if let Some(entry_type) = query.entry_type {
            entries.retain(|e| std::mem::discriminant(&e.entry_type) == std::mem::discriminant(&entry_type));
        }

        if let Some(filed_by) = query.filed_by {
            entries.retain(|e| e.filed_by.as_ref().map_or(false, |f| f.to_lowercase().contains(&filed_by.to_lowercase())));
        }

        if query.sealed_only {
            entries.retain(|e| e.is_sealed);
        }

        if let Some(date_from) = query.date_from {
            entries.retain(|e| e.date_filed >= date_from);
        }

        if let Some(date_to) = query.date_to {
            entries.retain(|e| e.date_filed <= date_to);
        }

        if let Some(search_text) = query.search_text {
            let search_lower = search_text.to_lowercase();
            entries.retain(|e| {
                e.description.to_lowercase().contains(&search_lower) ||
                e.filed_by.as_ref().map_or(false, |f| f.to_lowercase().contains(&search_lower))
            });
        }

        // Sort by entry number
        entries.sort_by_key(|e| e.entry_number);

        // Get total count before pagination
        let total = entries.len();

        // Apply pagination
        let paginated: Vec<DocketEntry> = entries
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok((paginated, total))
    }

    fn get_filing_statistics(&self, case_id: Uuid) -> Result<FilingStatistics> {
        let entries = self.find_entries_by_case(case_id)?;

        let motions_filed = entries.iter().filter(|e| matches!(e.entry_type, DocketEntryType::Motion)).count();
        let orders_entered = entries.iter().filter(|e| matches!(e.entry_type, DocketEntryType::Order | DocketEntryType::MinuteOrder | DocketEntryType::SchedulingOrder)).count();
        let sealed_entries = entries.iter().filter(|e| e.is_sealed).count();

        let first_entry = entries.iter().min_by_key(|e| e.date_filed);
        let last_entry = entries.iter().max_by_key(|e| e.date_filed);

        let days_since_filing = first_entry
            .map(|e| (Utc::now() - e.date_filed).num_days())
            .unwrap_or(0);

        let last_activity = last_entry
            .map(|e| e.date_filed)
            .unwrap_or_else(Utc::now);

        // Find most active filer
        let mut filer_counts: HashMap<String, usize> = HashMap::new();
        for entry in &entries {
            if let Some(filer) = &entry.filed_by {
                *filer_counts.entry(filer.clone()).or_insert(0) += 1;
            }
        }

        let most_active_filer = filer_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(filer, _)| filer)
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(FilingStatistics {
            total_entries: entries.len(),
            motions_filed,
            orders_entered,
            sealed_entries,
            days_since_filing,
            last_activity,
            most_active_filer,
        })
    }

    fn generate_docket_sheet(&self, case_id: Uuid) -> Result<String> {
        let entries = self.find_entries_by_case(case_id)?;
        let mut docket_sheet = String::new();

        docket_sheet.push_str(&format!("DOCKET SHEET - Case ID: {}\n", case_id));
        docket_sheet.push_str(&format!("Generated: {}\n\n", Utc::now().format("%m/%d/%Y %H:%M")));

        for entry in entries {
            docket_sheet.push_str(&format!(
                "{:4} | {} | {} | {}\n",
                entry.entry_number,
                entry.date_filed.format("%m/%d/%Y"),
                entry.filed_by.as_deref().unwrap_or("COURT"),
                entry.description
            ));

            if entry.is_sealed {
                docket_sheet.push_str("      [SEALED]\n");
            }

            if !entry.attachments.is_empty() {
                for attachment in &entry.attachments {
                    docket_sheet.push_str(&format!(
                        "      Attachment {}: {}\n",
                        attachment.attachment_number,
                        attachment.description
                    ));
                }
            }
        }

        Ok(docket_sheet)
    }
}

impl CalendarSchedulingRepository for SpinKvDocketRepository {
    fn search_calendar(&self, query: CalendarQuery) -> Result<(Vec<CalendarEntry>, usize)> {
        let mut events: Vec<CalendarEntry> = self.store
            .get_keys()?
            .iter()
            .filter(|key| key.starts_with(CALENDAR_KEY_PREFIX))
            .filter_map(|key| self.store.get_json::<CalendarEntry>(key.as_str()).ok())
            .filter_map(|event| event)
            .collect();

        // Apply filters
        if let Some(judge_id) = query.judge_id {
            events.retain(|e| e.judge_id == judge_id);
        }

        if let Some(courtroom) = query.courtroom {
            events.retain(|e| e.courtroom == courtroom);
        }

        if let Some(event_type) = query.event_type {
            events.retain(|e| std::mem::discriminant(&e.event_type) == std::mem::discriminant(&event_type));
        }

        if let Some(status) = query.status {
            events.retain(|e| e.status == status);
        }

        if let Some(date_from) = query.date_from {
            events.retain(|e| e.scheduled_date >= date_from);
        }

        if let Some(date_to) = query.date_to {
            events.retain(|e| e.scheduled_date <= date_to);
        }

        // Sort by scheduled date
        events.sort_by_key(|e| e.scheduled_date);

        // Get total count before pagination
        let total = events.len();

        // Apply pagination
        let paginated: Vec<CalendarEntry> = events
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        Ok((paginated, total))
    }

    fn find_available_slot(&self, judge_id: Uuid, duration_minutes: u32, earliest: DateTime<Utc>) -> Result<DateTime<Utc>> {
        let events = self.find_events_by_judge(judge_id)?;
        Ok(CalendarService::find_next_available_slot(&events, judge_id, duration_minutes, earliest))
    }

    fn get_judge_schedule(&self, judge_id: Uuid, date: DateTime<Utc>) -> Result<Vec<CalendarEntry>> {
        let start_of_day = date.date_naive().and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        let end_of_day = start_of_day + Duration::days(1);

        let events = self.find_events_by_judge(judge_id)?;

        Ok(events.into_iter().filter(|e| {
            e.scheduled_date >= start_of_day && e.scheduled_date < end_of_day
        }).collect())
    }

    fn get_courtroom_utilization(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<CourtroomUtilization> {
        let events = self.find_events_in_range(start_date, end_date)?;

        let mut courtroom_events: HashMap<String, Vec<CalendarEntry>> = HashMap::new();
        for event in &events {
            courtroom_events.entry(event.courtroom.clone()).or_insert_with(Vec::new).push(event.clone());
        }

        let total_courtrooms = courtroom_events.len();

        // Calculate utilization
        let business_hours_per_day = 8.0; // 9 AM to 5 PM
        let days = (end_date - start_date).num_days() as f32;
        let total_available_hours = business_hours_per_day * days * total_courtrooms as f32;

        let total_used_hours: f32 = events.iter()
            .map(|e| e.duration_minutes as f32 / 60.0)
            .sum();

        let average_utilization_percent = if total_available_hours > 0.0 {
            (total_used_hours / total_available_hours) * 100.0
        } else {
            0.0
        };

        // Find busiest courtroom
        let busiest_courtroom = courtroom_events
            .iter()
            .max_by_key(|(_, events)| events.len())
            .map(|(courtroom, _)| courtroom.clone())
            .unwrap_or_else(|| "None".to_string());

        // Find peak hours
        let mut hour_counts: HashMap<u32, usize> = HashMap::new();
        for event in &events {
            use chrono::Timelike;
            let hour = event.scheduled_date.hour();
            *hour_counts.entry(hour).or_insert(0) += 1;
        }

        let mut hour_counts_vec: Vec<(u32, usize)> = hour_counts.into_iter().collect();
        hour_counts_vec.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        let mut peak_hours: Vec<u32> = hour_counts_vec
            .into_iter()
            .take(3)
            .map(|(hour, _)| hour)
            .collect();
        peak_hours.sort();

        Ok(CourtroomUtilization {
            total_courtrooms,
            total_events: events.len(),
            average_utilization_percent,
            busiest_courtroom,
            peak_hours,
        })
    }
}

