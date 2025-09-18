//! Spin ToDo API - A RESTful API for managing ToDo items
//!
//! This application provides a complete CRUD API for ToDo items using
//! Spin's WebAssembly runtime and key-value store for persistence.
//!
//! ## Features
//! - Create, read, update, and delete ToDo items
//! - Toggle completion status of ToDo items
//! - Persistent storage using Spin's key-value store
//! - Interactive OpenAPI documentation with Swagger UI
//! - Soft delete functionality (items are marked as deleted, not removed)
//!
//! ## API Endpoints
//! - `GET /api/todos` - Retrieve all ToDo items
//! - `GET /api/todos/:id` - Get a specific ToDo item
//! - `POST /api/todos` - Create a new ToDo item
//! - `POST /api/todos/:id/toggle` - Toggle completion status
//! - `DELETE /api/todos/:id` - Delete a ToDo item (soft delete)
//! - `GET /docs` - Interactive API documentation
//! - `GET /docs/openapi-description.json` - OpenAPI specification

use spin_sdk::http::{IntoResponse, Request, Router};
use spin_sdk::http_component;

mod adapters;
mod domain;
mod error;
mod handlers;
mod macros;
mod ports;
mod services;
mod utils;

pub use error::{ApiError, ApiResult};

/// Main HTTP component handler for the Spin ToDo API
///
/// This function sets up the router with all API endpoints and documentation routes.
/// It's the entry point for all HTTP requests to the application.
#[http_component]
async fn handle_spin_todo_api(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::default();

    // Health check endpoint
    router.get("/api/health", handlers::health::health_check);

    // Configuration Management endpoints
    router.get("/api/config", handlers::config::get_config);
    router.get("/api/config/overrides/district", handlers::config::get_district_overrides);
    router.get("/api/config/overrides/judge", handlers::config::get_judge_overrides);
    router.put("/api/config/overrides/district", handlers::config::update_district_config);
    router.put("/api/config/overrides/judge", handlers::config::update_judge_config);
    router.delete("/api/config/overrides/district", handlers::config::clear_district_overrides);
    router.delete("/api/config/overrides/judge", handlers::config::clear_judge_overrides);
    router.post("/api/config/preview", handlers::config::preview_config);

    // ToDo API endpoints
    router.get("/api/todos", handlers::todo::get_all);
    router.get("/api/todos/:id", handlers::todo::get_by_id);
    router.post("/api/todos", handlers::todo::create_todo);
    router.post("/api/todos/:id/toggle", handlers::todo::toggle_by_id);
    router.delete("/api/todos/:id", handlers::todo::delete_by_id);

    // Criminal Case API endpoints (using hexagonal architecture)
    router.get("/api/cases", handlers::criminal_case::search_cases);
    router.get("/api/cases/statistics", handlers::criminal_case::get_case_statistics);
    router.get("/api/cases/by-number/:case_number", handlers::criminal_case::get_case_by_number);
    router.get("/api/cases/by-judge/:judge", handlers::criminal_case::get_cases_by_judge);
    router.get("/api/cases/count-by-status/:status", handlers::criminal_case::count_by_status);
    router.get("/api/cases/:id", handlers::criminal_case::get_case_by_id);
    router.post("/api/cases", handlers::criminal_case::create_case);
    router.post("/api/cases/:id/defendants", handlers::criminal_case::add_defendant);
    router.post("/api/cases/:id/evidence", handlers::criminal_case::add_evidence);
    router.post("/api/cases/:id/notes", handlers::criminal_case::add_note);
    router.post("/api/cases/:id/plea", handlers::criminal_case::enter_plea);
    router.post("/api/cases/:id/events", handlers::criminal_case::schedule_court_event);
    router.post("/api/cases/:id/motions", handlers::criminal_case::file_motion);
    router.patch("/api/cases/:id/motions/ruling", handlers::criminal_case::rule_on_motion);
    router.patch("/api/cases/:id/status", handlers::criminal_case::update_case_status);
    router.patch("/api/cases/:id/priority", handlers::criminal_case::update_case_priority);
    router.delete("/api/cases/:id", handlers::criminal_case::delete_case);

    // Judge Management API endpoints
    router.post("/api/judges", handlers::judge::create_judge);
    router.get("/api/judges", handlers::judge::get_all_judges);
    router.get("/api/judges/available", handlers::judge::get_available_judges);
    router.get("/api/judges/workload", handlers::judge::get_workload_stats);
    router.get("/api/judges/search", handlers::judge::search_judges);
    router.get("/api/judges/:id", handlers::judge::get_judge_by_id);
    router.patch("/api/judges/:id/status", handlers::judge::update_judge_status);
    router.post("/api/judges/:judge_id/conflicts", handlers::judge::add_conflict);
    router.get("/api/judges/conflicts/check/:party", handlers::judge::check_conflicts);

    // Case Assignment endpoints
    router.post("/api/assignments", handlers::judge::assign_case);
    router.get("/api/assignments/case/:case_id", handlers::judge::get_case_assignment);

    // Recusal endpoints
    router.post("/api/judges/:judge_id/recusals", handlers::judge::file_recusal);
    router.patch("/api/recusals/:recusal_id/ruling", handlers::judge::rule_on_recusal);
    router.get("/api/recusals/pending", handlers::judge::get_pending_recusals);

    // Docket Management API endpoints
    router.post("/api/docket/entries", handlers::docket::create_docket_entry);
    router.get("/api/docket/case/:case_id", handlers::docket::get_case_docket);
    router.get("/api/docket/entries/:id", handlers::docket::get_docket_entry);
    router.post("/api/docket/entries/:entry_id/attachments", handlers::docket::add_attachment);
    router.get("/api/docket/search", handlers::docket::search_docket);
    router.get("/api/docket/sheet/:case_id", handlers::docket::generate_docket_sheet);

    // Calendar Management endpoints
    router.post("/api/calendar/events", handlers::docket::schedule_event);
    router.get("/api/calendar/case/:case_id", handlers::docket::get_case_calendar);
    router.get("/api/calendar/judge/:judge_id", handlers::docket::get_judge_schedule);
    router.patch("/api/calendar/events/:event_id/status", handlers::docket::update_event_status);
    router.get("/api/calendar/available-slot/:judge_id", handlers::docket::find_available_slot);
    router.get("/api/calendar/utilization", handlers::docket::get_courtroom_utilization);

    // Speedy Trial Act endpoints
    router.post("/api/speedy-trial/:case_id", handlers::docket::init_speedy_trial);
    router.get("/api/speedy-trial/:case_id", handlers::docket::get_speedy_trial);
    router.post("/api/speedy-trial/:case_id/delays", handlers::docket::add_excludable_delay);
    router.get("/api/speedy-trial/approaching", handlers::docket::get_approaching_deadlines);

    // Deadline Management API endpoints
    router.post("/api/deadlines", handlers::deadline::create_deadline);
    router.get("/api/deadlines/case/:case_id", handlers::deadline::get_case_deadlines);
    router.get("/api/deadlines/:id", handlers::deadline::get_deadline);
    router.post("/api/deadlines/:id/complete", handlers::deadline::complete_deadline);
    router.get("/api/deadlines/upcoming", handlers::deadline::get_upcoming_deadlines);
    router.get("/api/deadlines/urgent", handlers::deadline::get_urgent_deadlines);
    router.get("/api/deadlines/search", handlers::deadline::search_deadlines);
    router.post("/api/deadlines/calculate", handlers::deadline::calculate_frcp_deadlines);

    // Extension Management endpoints
    router.post("/api/deadlines/:deadline_id/extensions", handlers::deadline::request_extension);
    router.patch("/api/extensions/:extension_id/ruling", handlers::deadline::rule_on_extension);

    // Compliance and Reporting endpoints
    router.get("/api/compliance/stats", handlers::deadline::get_compliance_stats);
    router.get("/api/compliance/report", handlers::deadline::generate_compliance_report);
    router.get("/api/compliance/performance", handlers::deadline::get_performance_metrics);
    router.get("/api/compliance/missed-jurisdictional", handlers::deadline::get_missed_jurisdictional);

    // Reminder endpoints
    router.get("/api/reminders/pending", handlers::deadline::get_pending_reminders);
    router.post("/api/reminders/send", handlers::deadline::send_reminders);
    router.get("/api/reminders/deadline/:deadline_id", handlers::deadline::get_reminders_by_deadline);
    router.get("/api/reminders/recipient/:recipient", handlers::deadline::get_reminders_by_recipient);
    router.post("/api/reminders/:reminder_id/acknowledge", handlers::deadline::acknowledge_reminder);

    // Additional Deadline endpoints
    router.get("/api/deadlines/case/:case_id/type/:type", handlers::deadline::get_deadlines_by_type);
    router.patch("/api/deadlines/:id/status", handlers::deadline::update_deadline_status);
    router.delete("/api/deadlines/:id", handlers::deadline::delete_deadline);

    // Additional Judge endpoints
    router.get("/api/judges/status/:status", handlers::judge::get_judges_by_status);
    router.get("/api/judges/district/:district", handlers::judge::get_judges_by_district);
    router.delete("/api/judges/:id", handlers::judge::delete_judge);
    router.get("/api/judges/vacation", handlers::judge::get_judges_on_vacation);
    router.get("/api/judges/:judge_id/conflicts", handlers::judge::get_conflicts_by_judge);
    router.get("/api/judges/:judge_id/conflicts/:party", handlers::judge::has_conflict);
    router.delete("/api/judges/:judge_id/conflicts/:conflict_id", handlers::judge::delete_conflict);

    // Additional Assignment endpoints
    router.get("/api/assignments/history/:case_id", handlers::judge::get_assignment_history);
    router.delete("/api/assignments/:id", handlers::judge::delete_assignment);

    // Additional Recusal endpoints
    router.get("/api/recusals/case/:case_id", handlers::judge::get_recusals_by_case);
    router.get("/api/recusals/judge/:judge_id", handlers::judge::get_recusals_by_judge);
    router.post("/api/recusals/:recusal_id/process", handlers::judge::process_recusal);

    // Additional Docket endpoints
    router.get("/api/docket/case/:case_id/type/:type", handlers::docket::get_entries_by_type);
    router.get("/api/docket/case/:case_id/sealed", handlers::docket::get_sealed_entries);
    router.get("/api/docket/case/:case_id/search/:text", handlers::docket::search_entries);
    router.delete("/api/docket/entries/:id", handlers::docket::delete_entry);
    router.get("/api/docket/statistics/:case_id", handlers::docket::get_filing_statistics);

    // Additional Calendar endpoints
    router.get("/api/calendar/courtroom/:courtroom", handlers::docket::get_events_by_courtroom);
    router.delete("/api/calendar/events/:id", handlers::docket::delete_event);

    // Additional Speedy Trial endpoints
    router.get("/api/speedy-trial/violations", handlers::docket::get_violations);
    router.patch("/api/speedy-trial/:case_id/clock", handlers::docket::update_clock);

    // Feature Management endpoints
    router.get("/api/features", handlers::features::get_features);
    router.patch("/api/features", handlers::features::update_feature);
    router.get("/api/features/implementation", handlers::features::get_implementation_status);
    router.patch("/api/features/implementation", handlers::features::update_implementation);
    router.get("/api/features/blocked", handlers::features::get_blocked_features);
    router.get("/api/features/ready", handlers::features::get_ready_features);
    router.post("/api/features/manager", handlers::features::create_feature_manager);
    router.get("/api/features/enabled/:feature_path", handlers::features::is_feature_enabled);
    router.post("/api/features/override", handlers::features::set_feature_override);
    router.delete("/api/features/overrides", handlers::features::clear_feature_overrides);

    // Additional Extension endpoints
    router.get("/api/extensions/:id", handlers::deadline::get_extension_by_id);
    router.get("/api/extensions/deadline/:deadline_id", handlers::deadline::get_extensions_by_deadline);
    router.get("/api/extensions/pending", handlers::deadline::get_pending_extensions);

    // Federal Rules endpoints
    router.get("/api/federal-rules", handlers::deadline::get_federal_rules);

    // Additional Calendar endpoints
    router.get("/api/calendar/search", handlers::docket::search_calendar);

    // Additional Docket service endpoints
    router.get("/api/docket/immediate-service/:entry_type", handlers::docket::check_immediate_service);
    router.get("/api/speedy-trial/approaching/:case_id", handlers::docket::check_deadline_approaching);

    // Admin endpoints for multi-tenancy
    router.post("/api/admin/init-tenant", handlers::admin::init_tenant);
    router.get("/api/admin/tenant-stats", handlers::admin::get_tenant_stats);

    // Judicial Orders API endpoints
    router.post("/api/orders", handlers::order::create_order);
    router.get("/api/orders", handlers::order::list_orders);
    router.get("/api/orders/:order_id", handlers::order::get_order);
    router.patch("/api/orders/:order_id", handlers::order::update_order);
    router.delete("/api/orders/:order_id", handlers::order::delete_order);
    router.post("/api/orders/:order_id/sign", handlers::order::sign_order);
    router.post("/api/orders/:order_id/issue", handlers::order::issue_order);
    router.post("/api/orders/:order_id/service", handlers::order::add_service_record);
    router.get("/api/cases/:case_id/orders", handlers::order::get_orders_by_case);
    router.get("/api/judges/:judge_id/orders", handlers::order::get_orders_by_judge);
    router.get("/api/judges/:judge_id/orders/pending-signatures", handlers::order::get_pending_signatures);
    router.get("/api/orders/expiring", handlers::order::get_expiring_orders);
    router.get("/api/orders/statistics", handlers::order::get_order_statistics);
    router.post("/api/orders/from-template", handlers::order::create_from_template);

    // Order Templates endpoints
    router.post("/api/templates/orders", handlers::order::create_template);
    router.get("/api/templates/orders", handlers::order::list_templates);
    router.get("/api/templates/orders/active", handlers::order::find_active_templates);
    router.get("/api/templates/orders/:template_id", handlers::order::get_template);
    router.put("/api/templates/orders/:template_id", handlers::order::update_template);
    router.delete("/api/templates/orders/:template_id", handlers::order::delete_template);
    router.post("/api/templates/:template_id/generate-content", handlers::order::generate_template_content);

    // Order helper method endpoints
    router.get("/api/orders/:id/is-expired", handlers::order::check_order_expired);
    router.get("/api/orders/:id/requires-attention", handlers::order::check_requires_attention);

    // Judicial Opinions API endpoints
    router.post("/api/opinions", handlers::opinion::create_opinion);
    router.get("/api/opinions", handlers::opinion::list_opinions);
    router.get("/api/opinions/:opinion_id", handlers::opinion::get_opinion);
    router.patch("/api/opinions/:opinion_id", handlers::opinion::update_opinion);
    router.delete("/api/opinions/:opinion_id", handlers::opinion::delete_opinion);
    router.post("/api/opinions/:opinion_id/file", handlers::opinion::file_opinion);
    router.post("/api/opinions/:opinion_id/publish", handlers::opinion::publish_opinion);
    router.post("/api/opinions/:opinion_id/votes", handlers::opinion::add_judge_vote);
    router.post("/api/opinions/:opinion_id/citations", handlers::opinion::add_citation);
    router.post("/api/opinions/:opinion_id/headnotes", handlers::opinion::add_headnote);
    router.get("/api/cases/:case_id/opinions", handlers::opinion::get_opinions_by_case);
    router.get("/api/judges/:judge_id/opinions", handlers::opinion::get_opinions_by_author);
    router.get("/api/opinions/search", handlers::opinion::search_opinions);
    router.get("/api/opinions/precedential", handlers::opinion::get_precedential_opinions);
    router.get("/api/opinions/statistics", handlers::opinion::get_opinion_statistics);
    router.get("/api/opinions/citations/statistics", handlers::opinion::get_citation_statistics);

    // Opinion Drafts endpoints
    router.post("/api/opinions/:opinion_id/drafts", handlers::opinion::create_draft);
    router.get("/api/opinions/:opinion_id/drafts", handlers::opinion::get_drafts);
    router.get("/api/opinions/:opinion_id/drafts/current", handlers::opinion::get_current_draft);
    router.post("/api/opinions/:opinion_id/drafts/:draft_id/comments", handlers::opinion::add_draft_comment);
    router.patch("/api/opinions/:opinion_id/drafts/:draft_id/comments/:comment_id/resolve", handlers::opinion::resolve_draft_comment);

    // Opinion helper method endpoints
    router.get("/api/opinions/:id/is-majority", handlers::opinion::is_majority_opinion);
    router.get("/api/opinions/:id/is-binding", handlers::opinion::is_binding_opinion);
    router.get("/api/opinions/:id/calculate-statistics", handlers::opinion::calculate_opinion_statistics);

    // Federal Sentencing System endpoints
    router.post("/api/sentencing", handlers::sentencing::create_sentencing);
    router.get("/api/sentencing/:id", handlers::sentencing::get_sentencing);
    router.put("/api/sentencing/:id", handlers::sentencing::update_sentencing);
    router.delete("/api/sentencing/:id", handlers::sentencing::delete_sentencing);
    router.get("/api/sentencing/case/:case_id", handlers::sentencing::find_by_case);
    router.get("/api/sentencing/defendant/:defendant_id", handlers::sentencing::find_by_defendant);
    router.get("/api/sentencing/judge/:judge_id", handlers::sentencing::find_by_judge);
    router.get("/api/sentencing/pending", handlers::sentencing::find_pending);
    router.post("/api/sentencing/calculate-guidelines", handlers::sentencing::calculate_guidelines);
    router.get("/api/sentencing/statistics/departures", handlers::sentencing::get_departure_stats);
    router.get("/api/sentencing/statistics/variances", handlers::sentencing::get_variance_stats);
    router.post("/api/sentencing/:id/departure", handlers::sentencing::add_departure);
    router.post("/api/sentencing/:id/variance", handlers::sentencing::add_variance);
    router.get("/api/sentencing/substantial-assistance", handlers::sentencing::get_substantial_assistance);
    router.post("/api/sentencing/:id/special-condition", handlers::sentencing::add_special_condition);
    router.put("/api/sentencing/:id/supervised-release", handlers::sentencing::update_supervised_release);
    router.get("/api/sentencing/active-supervision", handlers::sentencing::find_active_supervision);
    router.post("/api/sentencing/:id/bop-designation", handlers::sentencing::add_bop_designation);
    router.get("/api/sentencing/rdap-eligible", handlers::sentencing::get_rdap_eligible);
    router.get("/api/sentencing/statistics/judge/:judge_id", handlers::sentencing::get_judge_stats);
    router.get("/api/sentencing/statistics/district", handlers::sentencing::get_district_stats);
    router.get("/api/sentencing/statistics/trial-penalty", handlers::sentencing::get_trial_penalty);
    router.post("/api/sentencing/:id/prior-sentence", handlers::sentencing::add_prior_sentence);
    router.get("/api/sentencing/upcoming/:days", handlers::sentencing::find_upcoming);
    router.get("/api/sentencing/appeal-deadlines", handlers::sentencing::find_appeal_deadlines);
    router.get("/api/sentencing/date-range", handlers::sentencing::find_by_date_range);
    router.get("/api/sentencing/statistics/offense/:offense_type", handlers::sentencing::get_offense_type_stats);
    router.get("/api/sentencing/:id/criminal-history-points", handlers::sentencing::calculate_criminal_history_points);
    router.post("/api/sentencing/:id/calculate-offense-level", handlers::sentencing::calculate_offense_level);
    router.post("/api/sentencing/:id/lookup-guidelines-range", handlers::sentencing::lookup_guidelines_range);
    router.get("/api/sentencing/:id/safety-valve-eligible", handlers::sentencing::check_safety_valve_eligible);

    // Attorney Management API endpoints
    router.post("/api/attorneys", handlers::attorney::create_attorney);
    router.get("/api/attorneys", handlers::attorney::list_attorneys);
    router.get("/api/attorneys/search", handlers::attorney::search_attorneys);
    router.get("/api/attorneys/bar-number/:bar_number", handlers::attorney::get_attorney_by_bar_number);
    router.get("/api/attorneys/:id", handlers::attorney::get_attorney);
    router.put("/api/attorneys/:id", handlers::attorney::update_attorney);
    router.delete("/api/attorneys/:id", handlers::attorney::delete_attorney);
    router.get("/api/attorneys/status/:status", handlers::attorney::get_attorneys_by_status);
    router.get("/api/attorneys/firm/:firm_name", handlers::attorney::get_attorneys_by_firm);

    // Bar Admission endpoints
    router.post("/api/attorneys/:id/bar-admissions", handlers::attorney::add_bar_admission);
    router.delete("/api/attorneys/:id/bar-admissions/:state", handlers::attorney::remove_bar_admission);
    router.get("/api/attorneys/bar-state/:state", handlers::attorney::get_attorneys_by_bar_state);

    // Federal Admission endpoints
    router.post("/api/attorneys/:id/federal-admissions", handlers::attorney::add_federal_admission);
    router.delete("/api/attorneys/:id/federal-admissions/:court", handlers::attorney::remove_federal_admission);
    router.get("/api/attorneys/federal-court/:court", handlers::attorney::get_attorneys_admitted_to_court);

    // Pro Hac Vice endpoints
    router.post("/api/attorneys/:id/pro-hac-vice", handlers::attorney::add_pro_hac_vice);
    router.patch("/api/attorneys/:id/pro-hac-vice/:case_id/status", handlers::attorney::update_pro_hac_vice_status);
    router.get("/api/attorneys/pro-hac-vice/active", handlers::attorney::get_active_pro_hac_vice);
    router.get("/api/attorneys/pro-hac-vice/case/:case_id", handlers::attorney::get_pro_hac_vice_by_case);

    // CJA Panel endpoints
    router.post("/api/attorneys/:id/cja-panel/:district", handlers::attorney::add_to_cja_panel);
    router.delete("/api/attorneys/:id/cja-panel/:district", handlers::attorney::remove_from_cja_panel);
    router.get("/api/attorneys/cja-panel/:district", handlers::attorney::get_cja_panel_attorneys);
    router.post("/api/attorneys/:id/cja-appointments", handlers::attorney::add_cja_appointment);
    router.get("/api/attorneys/:id/cja-appointments", handlers::attorney::get_cja_appointments);
    router.get("/api/attorneys/cja/pending-vouchers", handlers::attorney::get_pending_cja_vouchers);

    // ECF Registration endpoints
    router.put("/api/attorneys/:id/ecf-registration", handlers::attorney::update_ecf_registration);
    router.get("/api/attorneys/:id/is-in-good-standing", handlers::attorney::check_good_standing);
    router.get("/api/attorneys/:id/can-practice/:court", handlers::attorney::check_federal_practice);
    router.get("/api/attorneys/:id/has-ecf-privileges", handlers::attorney::check_ecf_privileges);
    router.post("/api/attorneys/:id/calculate-win-rate", handlers::attorney::calculate_attorney_win_rate);
    router.get("/api/attorneys/ecf-access", handlers::attorney::get_attorneys_with_ecf);
    router.delete("/api/attorneys/:id/ecf-access", handlers::attorney::revoke_ecf_access);

    // Disciplinary Action endpoints
    router.post("/api/attorneys/:id/disciplinary-actions", handlers::attorney::add_disciplinary_action);
    router.get("/api/attorneys/:id/disciplinary-actions", handlers::attorney::get_disciplinary_history);
    router.get("/api/attorneys/with-discipline", handlers::attorney::get_attorneys_with_discipline);

    // Party Management endpoints
    router.post("/api/parties", handlers::attorney::create_party);
    router.get("/api/parties/:id", handlers::attorney::get_party);
    router.put("/api/parties/:id", handlers::attorney::update_party);
    router.delete("/api/parties/:id", handlers::attorney::delete_party);
    router.get("/api/parties/case/:case_id", handlers::attorney::list_parties_by_case);
    router.get("/api/parties/attorney/:attorney_id", handlers::attorney::list_parties_by_attorney);
    router.patch("/api/parties/:id/status", handlers::attorney::update_party_status);
    router.get("/api/parties/:id/needs-service", handlers::attorney::check_party_needs_service);
    router.get("/api/parties/:id/lead-counsel", handlers::attorney::get_party_lead_counsel);
    router.get("/api/parties/:id/is-represented", handlers::attorney::check_party_represented);
    router.get("/api/parties/unrepresented", handlers::attorney::get_unrepresented_parties);

    // Representation endpoints
    router.post("/api/representations", handlers::attorney::add_representation);
    router.post("/api/representations/:id/end", handlers::attorney::end_representation);
    router.get("/api/representations/:id", handlers::attorney::get_representation);
    router.get("/api/representations/attorney/:attorney_id/active", handlers::attorney::get_active_representations);
    router.get("/api/representations/case/:case_id", handlers::attorney::get_case_representations);
    router.post("/api/representations/substitute", handlers::attorney::substitute_attorney);

    // Service Records endpoints
    router.post("/api/service-records", handlers::attorney::create_service_record);
    router.get("/api/service-records/document/:document_id", handlers::attorney::get_service_by_document);
    router.get("/api/service-records/party/:party_id", handlers::attorney::get_service_by_party);
    router.post("/api/service-records/:id/complete", handlers::attorney::mark_service_completed);

    // Conflict Checking endpoints
    router.post("/api/conflict-checks", handlers::attorney::create_conflict_check);
    router.get("/api/conflict-checks/attorney/:attorney_id", handlers::attorney::get_attorney_conflicts);
    router.post("/api/conflict-checks/check", handlers::attorney::check_party_conflicts);
    router.post("/api/conflict-checks/:id/clear", handlers::attorney::clear_conflict);

    // Attorney Metrics endpoints
    router.get("/api/attorneys/:id/metrics", handlers::attorney::get_attorney_metrics);
    router.get("/api/attorneys/:id/win-rate", handlers::attorney::get_attorney_win_rate);
    router.get("/api/attorneys/:id/case-count", handlers::attorney::get_attorney_case_count);
    router.get("/api/attorneys/top-performers", handlers::attorney::get_top_attorneys);

    // Bulk Operations endpoints
    router.post("/api/attorneys/bulk/update-status", handlers::attorney::bulk_update_status);
    router.post("/api/service-records/bulk/:document_id", handlers::attorney::bulk_add_to_service);
    router.post("/api/representations/migrate", handlers::attorney::migrate_representations);

    // PDF Generation endpoints - Court Orders (Hexagonal Architecture)
    // Format parameter: 'pdf' for raw PDF, 'json' for base64-encoded JSON
    router.post("/api/pdf/rule16b/:format", handlers::pdf_hexagonal::generate_rule16b);
    router.post("/api/pdf/rule16b", handlers::pdf_hexagonal::generate_rule16b); // Default to JSON
    router.post("/api/pdf/signed/rule16b/:format", handlers::pdf_hexagonal::generate_signed_rule16b);
    router.post("/api/pdf/signed/rule16b", handlers::pdf_hexagonal::generate_signed_rule16b); // Default to JSON
    router.post("/api/pdf/court-order/:format", handlers::pdf_hexagonal::generate_court_order);
    router.post("/api/pdf/court-order", handlers::pdf_hexagonal::generate_court_order); // Default to JSON
    router.post("/api/pdf/minute-entry/:format", handlers::pdf_hexagonal::generate_minute_entry);
    router.post("/api/pdf/minute-entry", handlers::pdf_hexagonal::generate_minute_entry); // Default to JSON
    // Auto-generation endpoints (TODO: implement in hexagonal)
    // router.get("/api/pdf/auto/rule-16b/:case_id", auto_generate_rule_16b);
    // router.post("/api/pdf/judge-signature", upload_judge_signature);

    // PDF Generation endpoints - Federal Forms
    router.post("/api/pdf/waiver-indictment/:format", handlers::pdf_hexagonal::generate_waiver_indictment);
    router.post("/api/pdf/waiver-indictment", handlers::pdf_hexagonal::generate_waiver_indictment); // Default to JSON
    router.post("/api/pdf/conditions-release/:format", handlers::pdf_hexagonal::generate_conditions_release);
    router.post("/api/pdf/conditions-release", handlers::pdf_hexagonal::generate_conditions_release); // Default to JSON
    router.post("/api/pdf/criminal-judgment/:format", handlers::pdf_hexagonal::generate_criminal_judgment);
    router.post("/api/pdf/criminal-judgment", handlers::pdf_hexagonal::generate_criminal_judgment); // Default to JSON
    // Auto-generation endpoints (TODO: migrate to hexagonal)
    // router.get("/api/pdf/auto/waiver-indictment/:case_id", auto_generate_waiver);
    // router.get("/api/pdf/auto/conditions-release/:case_id", auto_generate_conditions);
    // router.get("/api/pdf/auto/criminal-judgment/:case_id", auto_generate_judgment);

    // Batch PDF Generation (Hexagonal Architecture)
    router.post("/api/pdf/batch", handlers::pdf_hexagonal::generate_batch_pdfs);
    // router.post("/api/pdf/batch/zip", handlers::pdf_batch::generate_batch_pdfs_zip); // TODO: implement ZIP in hexagonal

    // Signature Management endpoints
    router.post("/api/signatures", handlers::pdf_hexagonal::store_signature);
    router.get("/api/signatures/:judge_id", handlers::pdf_hexagonal::get_signature);

    // Documentation endpoints
    router.get(
        "/docs/openapi-description.json",
        handlers::docs::get_openapi_description,
    );
    router.get("/docs/*", handlers::docs::render_openapi_docs_ui);

    Ok(router.handle(req))
}
