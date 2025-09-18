//! REST API handlers for docket and calendar management
//!
//! This module provides HTTP endpoints for managing docket entries,
//! court calendar, and Speedy Trial Act compliance.

use crate::utils::repository_factory::RepositoryFactory;
use crate::domain::docket::{
    DocketEntry, DocketEntryType, DocketAttachment, CalendarEntry,
    CalendarEventType, EventStatus, SpeedyTrialClock, ExcludableDelay,
    DelayReason, DocketService, CalendarService, SpeedyTrialService
};
use crate::error::{ApiError, ApiResult};
use crate::ports::docket_repository::{
    DocketRepository, CalendarRepository, SpeedyTrialRepository,
    DocketQuery, DocketQueryRepository,
    CalendarSchedulingRepository
};
use crate::utils::query_parser;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use uuid::Uuid;
use utoipa::ToSchema;

/// Request to initialize speedy trial tracking
#[derive(Deserialize, ToSchema)]
pub struct InitSpeedyTrialRequest {
    pub arrest_date: Option<DateTime<Utc>>,
    pub indictment_date: Option<DateTime<Utc>>,
    pub arraignment_date: Option<DateTime<Utc>>,
}

/// Search response for docket entries
#[derive(Serialize, ToSchema)]
pub struct SearchResponse {
    pub entries: Vec<DocketEntry>,
    pub total: usize,
}

/// Response for available court slot
#[derive(Serialize, ToSchema)]
pub struct AvailableSlotResponse {
    pub judge_id: Uuid,
    pub available_date: DateTime<Utc>,
    pub duration_minutes: u32,
}

/// Search response for calendar entries
#[derive(Serialize, ToSchema)]
pub struct CalendarSearchResponse {
    pub events: Vec<CalendarEntry>,
    pub total: usize,
}

/// Request model for creating a docket entry
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDocketEntryRequest {
    pub case_id: Uuid,
    pub entry_type: DocketEntryType,
    pub description: String,
    pub filed_by: Option<String>,
    pub is_sealed: bool,
    pub is_ex_parte: bool,
    pub page_count: Option<u32>,
    pub service_list: Vec<String>,
}

/// Request model for adding an attachment
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddAttachmentRequest {
    pub attachment_number: u32,
    pub description: String,
    pub page_count: u32,
    pub file_size_bytes: u64,
}

/// Request model for scheduling a court event
#[derive(Debug, Deserialize, ToSchema)]
pub struct ScheduleEventRequest {
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub event_type: CalendarEventType,
    pub scheduled_date: DateTime<Utc>,
    pub duration_minutes: u32,
    pub courtroom: String,
    pub description: String,
    pub participants: Vec<String>,
    pub is_public: bool,
}

/// Request model for updating event status
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEventStatusRequest {
    pub status: EventStatus,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Request model for adding excludable delay
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddExcludableDelayRequest {
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub reason: DelayReason,
    pub statutory_reference: String,
    pub order_reference: Option<String>,
}

/// Response model for filing statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct FilingStatsResponse {
    pub total_entries: usize,
    pub motions_filed: usize,
    pub orders_entered: usize,
    pub sealed_entries: usize,
    pub days_since_filing: i64,
    pub last_activity: DateTime<Utc>,
}

impl FilingStatsResponse {
    pub fn from_statistics(stats: crate::ports::docket_repository::FilingStatistics) -> Self {
        Self {
            total_entries: stats.total_entries,
            motions_filed: stats.motions_filed,
            orders_entered: stats.orders_entered,
            sealed_entries: stats.sealed_entries,
            days_since_filing: stats.days_since_filing,
            last_activity: stats.last_activity,
        }
    }
}

/// Create a new docket entry
#[utoipa::path(
    post,
    path = "/api/docket/entries",
    request_body = CreateDocketEntryRequest,
    responses(
        (status = 201, description = "Docket entry created successfully", body = DocketEntry),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Docket Management"
)]
pub fn create_docket_entry(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateDocketEntryRequest = serde_json::from_slice(body)?;

    let mut entry = DocketService::create_entry(
        request.case_id,
        request.entry_type,
        request.description,
        request.filed_by,
    );

    entry.is_sealed = request.is_sealed;
    entry.is_ex_parte = request.is_ex_parte;
    entry.page_count = request.page_count;
    entry.service_list = request.service_list;

    let repo = RepositoryFactory::docket_repo(&req);
    repo.save_entry(&entry)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&entry)?)
        .build())
}

/// Get docket entries for a case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of docket entries for the case", body = [DocketEntry]),
        (status = 400, description = "Invalid case ID"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Docket Management"
)]
pub fn get_case_docket(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let entries = repo.find_entries_by_case(case_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&entries)?)
        .build())
}

/// Get a specific docket entry
#[utoipa::path(
    get,
    path = "/api/docket/entries/{id}",
    params(
        ("id" = Uuid, Path, description = "Docket entry ID")
    ),
    responses(
        (status = 200, description = "Docket entry found", body = DocketEntry),
        (status = 404, description = "Docket entry not found"),
        (status = 400, description = "Invalid entry ID")
    ),
    tag = "Docket Management"
)]
pub fn get_docket_entry(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid entry ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let entry = repo
        .find_entry_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Docket entry not found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&entry)?)
        .build())
}

/// Add attachment to docket entry
#[utoipa::path(
    post,
    path = "/api/docket/entries/{entry_id}/attachments",
    params(
        ("entry_id" = Uuid, Path, description = "Docket entry ID")
    ),
    request_body = AddAttachmentRequest,
    responses(
        (status = 201, description = "Attachment added successfully", body = DocketAttachment),
        (status = 404, description = "Docket entry not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Docket Management"
)]
pub fn add_attachment(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let entry_id = params
        .get("entry_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid entry ID".to_string()))?;

    let body = req.body();
    let request: AddAttachmentRequest = serde_json::from_slice(body)?;

    let repo = RepositoryFactory::docket_repo(&req);
    let mut entry = repo
        .find_entry_by_id(entry_id)?
        .ok_or_else(|| ApiError::NotFound("Docket entry not found".to_string()))?;

    let attachment = DocketAttachment {
        id: Uuid::new_v4(),
        attachment_number: request.attachment_number,
        description: request.description,
        page_count: request.page_count,
        file_size_bytes: request.file_size_bytes,
    };

    entry.attachments.push(attachment.clone());
    repo.save_entry(&entry)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&attachment)?)
        .build())
}

/// Search docket entries
#[utoipa::path(
    get,
    path = "/api/docket/search",
    params(
        ("case_id" = Option<Uuid>, Query, description = "Filter by case ID"),
        ("entry_type" = Option<String>, Query, description = "Filter by entry type"),
        ("filed_by" = Option<String>, Query, description = "Filter by who filed the entry"),
        ("sealed_only" = Option<bool>, Query, description = "Show only sealed entries"),
        ("date_from" = Option<String>, Query, description = "Start date filter (RFC3339 format)"),
        ("date_to" = Option<String>, Query, description = "End date filter (RFC3339 format)"),
        ("search" = Option<String>, Query, description = "Text search in description"),
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit")
    ),
    responses(
        (status = 200, description = "Search results with entries and total count"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Docket Management"
)]
pub fn search_docket(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let params = query_parser::parse_query_string(query_string);

    let query = DocketQuery {
        case_id: query_parser::get_uuid(&params, "case_id"),
        entry_type: query_parser::get_json(&params, "entry_type"),
        filed_by: query_parser::get_string(&params, "filed_by"),
        sealed_only: query_parser::get_bool(&params, "sealed_only").unwrap_or(false),
        date_from: query_parser::get_datetime(&params, "date_from"),
        date_to: query_parser::get_datetime(&params, "date_to"),
        search_text: query_parser::get_string(&params, "search"),
        offset: query_parser::get_usize(&params, "offset").unwrap_or(0),
        limit: query_parser::get_usize(&params, "limit").unwrap_or(50),
    };

    let repo = RepositoryFactory::docket_repo(&req);
    let (entries, total) = repo.search_docket(query)?;


    let response = SearchResponse { entries, total };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Generate docket sheet
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket-sheet",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "Generated docket sheet", content_type = "text/plain"),
        (status = 400, description = "Invalid case ID"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Docket Management"
)]
pub fn generate_docket_sheet(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let docket_sheet = repo.generate_docket_sheet(case_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "text/plain")
        .body(docket_sheet)
        .build())
}

/// Schedule a court event
#[utoipa::path(
    post,
    path = "/api/calendar/events",
    request_body = ScheduleEventRequest,
    responses(
        (status = 201, description = "Event scheduled successfully", body = CalendarEntry),
        (status = 400, description = "Invalid request data or schedule conflict"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Calendar Management"
)]
pub fn schedule_event(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: ScheduleEventRequest = serde_json::from_slice(body)?;

    let mut event = CalendarService::schedule_event(
        request.case_id,
        request.judge_id,
        request.event_type,
        request.scheduled_date,
        request.duration_minutes,
        request.courtroom,
    );

    event.description = request.description;
    event.participants = request.participants;
    event.is_public = request.is_public;

    let repo = RepositoryFactory::docket_repo(&req);

    // Check for conflicts
    let conflicts = repo.find_conflicts(
        request.judge_id,
        request.scheduled_date,
        request.scheduled_date + chrono::Duration::minutes(request.duration_minutes as i64),
    )?;

    if !conflicts.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "Schedule conflict: {} existing events conflict with this time",
            conflicts.len()
        )));
    }

    repo.save_event(&event)?;

    // Generate automatic docket entry
    let docket_entry = DocketService::generate_minute_entry(&event);
    repo.save_entry(&docket_entry)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&event)?)
        .build())
}

/// Get calendar events for a case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/calendar",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of calendar events for the case", body = [CalendarEntry]),
        (status = 400, description = "Invalid case ID"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Calendar Management"
)]
pub fn get_case_calendar(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let events = repo.find_events_by_case(case_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&events)?)
        .build())
}

/// Get judge's schedule
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/schedule",
    params(
        ("judge_id" = Uuid, Path, description = "Judge ID"),
        ("date" = Option<String>, Query, description = "Date to get schedule for (RFC3339 format, defaults to today)")
    ),
    responses(
        (status = 200, description = "Judge's schedule for the specified date", body = [CalendarEntry]),
        (status = 400, description = "Invalid judge ID"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Calendar Management"
)]
pub fn get_judge_schedule(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let date = params
        .get("date")
        .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let repo = RepositoryFactory::docket_repo(&req);
    let events = repo.get_judge_schedule(judge_id, date)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&events)?)
        .build())
}

/// Update event status
#[utoipa::path(
    patch,
    path = "/api/calendar/events/{event_id}/status",
    params(
        ("event_id" = Uuid, Path, description = "Calendar event ID")
    ),
    request_body = UpdateEventStatusRequest,
    responses(
        (status = 200, description = "Event status updated", body = CalendarEntry),
        (status = 404, description = "Event not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Calendar Management"
)]
pub fn update_event_status(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let event_id = params
        .get("event_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid event ID".to_string()))?;

    let body = req.body();
    let request: UpdateEventStatusRequest = serde_json::from_slice(body)?;

    let repo = RepositoryFactory::docket_repo(&req);

    // First update just the status using the dedicated method
    repo.update_event_status(event_id, request.status)?;

    // Then get the event and update other fields if needed
    let mut event = repo
        .find_event_by_id(event_id)?
        .ok_or_else(|| ApiError::NotFound("Event not found".to_string()))?;

    if let Some(start) = request.actual_start {
        event.actual_start = Some(start);
    }
    if let Some(end) = request.actual_end {
        event.actual_end = Some(end);
    }
    if let Some(ref notes) = request.notes {
        event.notes = notes.clone();
    }

    // Save the event with additional updates if any were made
    if request.actual_start.is_some() || request.actual_end.is_some() || request.notes.is_some() {
        repo.save_event(&event)?;
    }

    // Generate minute entry if completed
    if event.status == EventStatus::Completed {
        let minute_entry = DocketService::generate_minute_entry(&event);
        repo.save_entry(&minute_entry)?;
    }

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&event)?)
        .build())
}

/// Find next available slot
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/available-slot",
    params(
        ("judge_id" = Uuid, Path, description = "Judge ID"),
        ("duration" = Option<u32>, Query, description = "Duration in minutes (default: 60)"),
        ("earliest" = Option<String>, Query, description = "Earliest acceptable date (RFC3339 format, defaults to now)")
    ),
    responses(
        (status = 200, description = "Next available time slot found"),
        (status = 400, description = "Invalid judge ID"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Calendar Management"
)]
pub fn find_available_slot(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params
        .get("judge_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid judge ID".to_string()))?;

    let duration = params
        .get("duration")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(60);

    let earliest = params
        .get("earliest")
        .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let repo = RepositoryFactory::docket_repo(&req);
    let next_slot = repo.find_available_slot(judge_id, duration, earliest)?;


    let response = AvailableSlotResponse {
        judge_id,
        available_date: next_slot,
        duration_minutes: duration,
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Initialize Speedy Trial clock
#[utoipa::path(
    post,
    path = "/api/cases/{case_id}/speedy-trial",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    request_body(content = InitSpeedyTrialRequest, description = "Speedy trial initialization data"),
    responses(
        (status = 201, description = "Speedy Trial clock initialized", body = SpeedyTrialClock),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Speedy Trial Management"
)]
pub fn init_speedy_trial(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;


    let body = req.body();
    let request: InitSpeedyTrialRequest = serde_json::from_slice(body)?;

    let arraignment = request.arraignment_date.ok_or_else(|| {
        ApiError::BadRequest("Arraignment date required".to_string())
    })?;

    let indictment = request.indictment_date.unwrap_or(arraignment);

    let clock = SpeedyTrialClock {
        case_id,
        arrest_date: request.arrest_date,
        indictment_date: Some(indictment),
        arraignment_date: Some(arraignment),
        trial_start_deadline: SpeedyTrialService::calculate_deadline(indictment, arraignment),
        excludable_delays: Vec::new(),
        days_elapsed: 0,
        days_remaining: 70,
        is_tolled: false,
        waived: false,
    };

    let repo = RepositoryFactory::docket_repo(&req);
    repo.save_clock(&clock)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&clock)?)
        .build())
}

/// Get Speedy Trial clock status
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/speedy-trial",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "Speedy Trial clock status", body = SpeedyTrialClock),
        (status = 404, description = "Speedy Trial clock not found"),
        (status = 400, description = "Invalid case ID")
    ),
    tag = "Speedy Trial Management"
)]
pub fn get_speedy_trial(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let mut clock = repo
        .find_clock_by_case(case_id)?
        .ok_or_else(|| ApiError::NotFound("Speedy Trial clock not found".to_string()))?;

    // Update days remaining
    clock.days_remaining = SpeedyTrialService::calculate_days_remaining(&clock, Utc::now());

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&clock)?)
        .build())
}

/// Add excludable delay
#[utoipa::path(
    post,
    path = "/api/cases/{case_id}/speedy-trial/delays",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    request_body = AddExcludableDelayRequest,
    responses(
        (status = 200, description = "Excludable delay added", body = SpeedyTrialClock),
        (status = 404, description = "Speedy Trial clock not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Speedy Trial Management"
)]
pub fn add_excludable_delay(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let body = req.body();
    let request: AddExcludableDelayRequest = serde_json::from_slice(body)?;

    let repo = RepositoryFactory::docket_repo(&req);
    let mut clock = repo
        .find_clock_by_case(case_id)?
        .ok_or_else(|| ApiError::NotFound("Speedy Trial clock not found".to_string()))?;

    let days_excluded = request.end_date
        .map(|end| (end - request.start_date).num_days())
        .unwrap_or(0);

    let delay = ExcludableDelay {
        start_date: request.start_date,
        end_date: request.end_date,
        reason: request.reason,
        statutory_reference: request.statutory_reference,
        days_excluded,
        order_reference: request.order_reference,
    };

    clock.excludable_delays.push(delay);
    clock.days_remaining = SpeedyTrialService::calculate_days_remaining(&clock, Utc::now());

    repo.save_clock(&clock)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&clock)?)
        .build())
}

/// Get approaching Speedy Trial deadlines
#[utoipa::path(
    get,
    path = "/api/speedy-trial/deadlines/approaching",
    responses(
        (status = 200, description = "List of cases with approaching Speedy Trial deadlines", body = [SpeedyTrialClock]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Speedy Trial Management"
)]
pub fn get_approaching_deadlines(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = RepositoryFactory::docket_repo(&req);
    let approaching = repo.find_approaching_deadlines(14)?; // 14 days threshold

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&approaching)?)
        .build())
}

/// Get courtroom utilization
#[utoipa::path(
    get,
    path = "/api/courtrooms/utilization",
    params(
        ("start" = Option<String>, Query, description = "Start date (RFC3339 format, defaults to 30 days ago)"),
        ("end" = Option<String>, Query, description = "End date (RFC3339 format, defaults to now)")
    ),
    responses(
        (status = 200, description = "Courtroom utilization statistics"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Analytics"
)]
pub fn get_courtroom_utilization(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let start_date = params
        .get("start")
        .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));

    let end_date = params
        .get("end")
        .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let repo = RepositoryFactory::docket_repo(&req);
    let utilization = repo.get_courtroom_utilization(start_date, end_date)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&utilization)?)
        .build())
}

/// Get docket entries by type
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket/type/{type}",
    params(
        ("case_id" = Uuid, Path, description = "Case ID"),
        ("type" = String, Path, description = "Docket entry type")
    ),
    responses(
        (status = 200, description = "List of docket entries of the specified type", body = [DocketEntry]),
        (status = 400, description = "Invalid case ID or entry type")
    ),
    tag = "Docket Management"
)]
pub fn get_entries_by_type(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let entry_type_str = params
        .get("type")
        .ok_or_else(|| ApiError::BadRequest("Entry type required".to_string()))?;

    let entry_type: DocketEntryType = serde_json::from_str(&format!("\"{}\"", entry_type_str))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let entries = repo.find_entries_by_type(case_id, entry_type)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&entries)?)
        .build())
}

/// Get sealed entries for a case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket/sealed",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of sealed docket entries for the case", body = [DocketEntry]),
        (status = 400, description = "Invalid case ID")
    ),
    tag = "Docket Management"
)]
pub fn get_sealed_entries(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let entries = repo.find_sealed_entries(case_id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&entries)?)
        .build())
}

/// Search entries by text
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/docket/search/{text}",
    params(
        ("case_id" = Uuid, Path, description = "Case ID"),
        ("text" = String, Path, description = "Text to search for in docket entries")
    ),
    responses(
        (status = 200, description = "List of docket entries matching the search text", body = [DocketEntry]),
        (status = 400, description = "Invalid case ID or search text required")
    ),
    tag = "Docket Management"
)]
pub fn search_entries(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let search_text = params
        .get("text")
        .ok_or_else(|| ApiError::BadRequest("Search text required".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let entries = repo.search_entries(case_id, search_text)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&entries)?)
        .build())
}

/// Delete a docket entry
#[utoipa::path(
    delete,
    path = "/api/docket/entries/{id}",
    params(
        ("id" = Uuid, Path, description = "Docket entry ID")
    ),
    responses(
        (status = 200, description = "Docket entry deleted successfully"),
        (status = 404, description = "Docket entry not found"),
        (status = 400, description = "Invalid entry ID")
    ),
    tag = "Docket Management"
)]
pub fn delete_entry(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid entry ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let deleted = repo.delete_entry(id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"deleted": deleted}))?)
        .build())
}

/// Get events by courtroom
#[utoipa::path(
    get,
    path = "/api/courtrooms/{courtroom}/events",
    params(
        ("courtroom" = String, Path, description = "Courtroom identifier"),
        ("start" = Option<String>, Query, description = "Start date (RFC3339 format, defaults to now)"),
        ("end" = Option<String>, Query, description = "End date (RFC3339 format, defaults to 30 days from now)")
    ),
    responses(
        (status = 200, description = "List of events in the specified courtroom", body = [CalendarEntry]),
        (status = 400, description = "Courtroom required")
    ),
    tag = "Calendar Management"
)]
pub fn get_events_by_courtroom(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let courtroom = params
        .get("courtroom")
        .ok_or_else(|| ApiError::BadRequest("Courtroom required".to_string()))?;

    let query_string = req.query();
    let query_params = query_parser::parse_query_string(query_string);

    let start_date = query_parser::get_datetime(&query_params, "start")
        .unwrap_or_else(|| Utc::now());
    let end_date = query_parser::get_datetime(&query_params, "end")
        .unwrap_or_else(|| Utc::now() + chrono::Duration::days(30));

    let repo = RepositoryFactory::docket_repo(&req);
    let events = repo.find_events_by_courtroom(courtroom, start_date, end_date)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&events)?)
        .build())
}

/// Delete a calendar event
#[utoipa::path(
    delete,
    path = "/api/calendar/events/{id}",
    params(
        ("id" = Uuid, Path, description = "Calendar event ID")
    ),
    responses(
        (status = 200, description = "Calendar event deleted successfully"),
        (status = 404, description = "Calendar event not found"),
        (status = 400, description = "Invalid event ID")
    ),
    tag = "Calendar Management"
)]
pub fn delete_event(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid event ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let deleted = repo.delete_event(id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"deleted": deleted}))?)
        .build())
}

/// Get filing statistics for a case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/filing-stats",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "Filing statistics for the case", body = FilingStatsResponse),
        (status = 400, description = "Invalid case ID")
    ),
    tag = "Analytics"
)]
pub fn get_filing_statistics(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let stats = repo.get_filing_statistics(case_id)?;
    let response = FilingStatsResponse::from_statistics(stats);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Find Speedy Trial Act violations
#[utoipa::path(
    get,
    path = "/api/speedy-trial/violations",
    responses(
        (status = 200, description = "List of cases with Speedy Trial Act violations", body = [SpeedyTrialClock]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Speedy Trial Management"
)]
pub fn get_violations(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = RepositoryFactory::docket_repo(&req);
    let violations = repo.find_violations()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&violations)?)
        .build())
}

/// Update Speedy Trial clock
#[utoipa::path(
    put,
    path = "/api/cases/{case_id}/speedy-trial",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    request_body = SpeedyTrialClock,
    responses(
        (status = 200, description = "Speedy Trial clock updated successfully"),
        (status = 400, description = "Invalid case ID or request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Speedy Trial Management"
)]
pub fn update_clock(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let body = req.body();
    let clock: SpeedyTrialClock = serde_json::from_slice(body)?;

    let repo = RepositoryFactory::docket_repo(&req);
    repo.update_clock(case_id, &clock)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(b"{\"success\": true}".to_vec())
        .build())
}

/// Search calendar with filters
#[utoipa::path(
    get,
    path = "/api/calendar/search",
    params(
        ("judge_id" = Option<Uuid>, Query, description = "Filter by judge ID"),
        ("courtroom" = Option<String>, Query, description = "Filter by courtroom"),
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("status" = Option<String>, Query, description = "Filter by event status"),
        ("date_from" = Option<String>, Query, description = "Start date filter (RFC3339 format)"),
        ("date_to" = Option<String>, Query, description = "End date filter (RFC3339 format)"),
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit")
    ),
    responses(
        (status = 200, description = "Search results with calendar events and total count"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Calendar Management"
)]
pub fn search_calendar(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::ports::docket_repository::CalendarQuery;

    let query_string = req.query();
    let params = query_parser::parse_query_string(query_string);

    let query = CalendarQuery {
        judge_id: query_parser::get_uuid(&params, "judge_id"),
        courtroom: query_parser::get_string(&params, "courtroom"),
        event_type: query_parser::get_json(&params, "event_type"),
        status: query_parser::get_json(&params, "status"),
        date_from: query_parser::get_datetime(&params, "date_from"),
        date_to: query_parser::get_datetime(&params, "date_to"),
        offset: query_parser::get_usize(&params, "offset").unwrap_or(0),
        limit: query_parser::get_usize(&params, "limit").unwrap_or(50),
    };

    let repo = RepositoryFactory::docket_repo(&req);
    let (events, total) = repo.search_calendar(query)?;


    let response = CalendarSearchResponse { events, total };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Check if a docket entry requires immediate service
#[utoipa::path(
    get,
    path = "/api/docket/service-check/{entry_type}",
    params(
        ("entry_type" = String, Path, description = "Docket entry type to check")
    ),
    responses(
        (status = 200, description = "Service requirement check result"),
        (status = 400, description = "Entry type required")
    ),
    tag = "Docket Management"
)]
pub fn check_immediate_service(_req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let entry_type_str = params
        .get("entry_type")
        .ok_or_else(|| ApiError::BadRequest("Entry type required".to_string()))?;

    let entry_type: DocketEntryType = serde_json::from_str(&format!("\"{}\"", entry_type_str))?;

    let requires_immediate = DocketService::requires_immediate_service(&entry_type);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({
            "entry_type": entry_type_str,
            "requires_immediate_service": requires_immediate
        }))?)
        .build())
}

/// Check if Speedy Trial deadline is approaching
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/speedy-trial/deadline-check",
    params(
        ("case_id" = Uuid, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "Deadline status check result"),
        (status = 404, description = "Speedy Trial clock not found"),
        (status = 400, description = "Invalid case ID")
    ),
    tag = "Speedy Trial Management"
)]
pub fn check_deadline_approaching(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params
        .get("case_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid case ID".to_string()))?;

    let repo = RepositoryFactory::docket_repo(&req);
    let clock = repo.find_clock_by_case(case_id)?
        .ok_or_else(|| ApiError::NotFound("Speedy Trial clock not found".to_string()))?;

    let is_approaching = SpeedyTrialService::is_deadline_approaching(&clock);
    let is_violated = SpeedyTrialService::is_deadline_violated(&clock);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({
            "case_id": case_id,
            "is_approaching": is_approaching,
            "is_violated": is_violated,
            "days_remaining": clock.days_remaining
        }))?)
        .build())
}