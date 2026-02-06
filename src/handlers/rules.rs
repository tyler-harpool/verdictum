//! REST API handlers for rules engine management
//!
//! This module provides HTTP endpoints for managing court rules,
//! including CRUD operations, filtering, and search capabilities.

use crate::domain::rule::{
    Rule, CreateRuleRequest, UpdateRuleRequest, RuleCategory, RuleStatus,
    TriggerEvent,
};
use crate::error::{ApiError, ApiResult};
use crate::ports::rules_repository::{RulesRepository, RuleQuery, RuleQueryRepository};
use crate::utils::{query_parser, repository_factory::RepositoryFactory};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use uuid::Uuid;
use utoipa::ToSchema;
use chrono::Utc;

/// Search response for rules
#[derive(Serialize, ToSchema)]
pub struct RuleSearchResponse {
    pub rules: Vec<Rule>,
    pub total: usize,
}

/// Placeholder request for Phase 2 rule evaluation
#[derive(Deserialize, ToSchema)]
pub struct EvaluateRulesRequest {
    pub trigger: TriggerEvent,
    pub context: serde_json::Value,
}

/// Placeholder response for Phase 2 rule evaluation
#[derive(Serialize, ToSchema)]
pub struct EvaluateRulesResponse {
    pub message: String,
    pub evaluated_count: usize,
}

/// Create a new rule
#[utoipa::path(
    post,
    path = "/api/rules",
    request_body = CreateRuleRequest,
    responses(
        (status = 201, description = "Rule created successfully", body = Rule),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Rules Engine",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_rule(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateRuleRequest = serde_json::from_slice(body)?;

    let now = Utc::now();
    let rule = Rule {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        source: request.source,
        category: request.category,
        triggers: request.triggers,
        conditions: request.conditions,
        actions: request.actions,
        priority: request.priority.unwrap_or(crate::domain::rule::RulePriority::FederalRule),
        status: request.status.unwrap_or(RuleStatus::Draft),
        jurisdiction: request.jurisdiction,
        citation: request.citation,
        effective_date: request.effective_date,
        expiration_date: request.expiration_date,
        supersedes_rule_id: request.supersedes_rule_id,
        created_at: now,
        updated_at: now,
        created_by: request.created_by,
    };

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    repo.save_rule(&rule)?;

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&rule)?)
        .build())
}

/// List all rules with optional search filters
#[utoipa::path(
    get,
    path = "/api/rules",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("category" = Option<String>, Query, description = "Filter by rule category"),
        ("trigger" = Option<String>, Query, description = "Filter by trigger event"),
        ("status" = Option<String>, Query, description = "Filter by rule status"),
        ("jurisdiction" = Option<String>, Query, description = "Filter by jurisdiction"),
        ("source" = Option<String>, Query, description = "Filter by rule source"),
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit")
    ),
    responses(
        (status = 200, description = "List of rules matching filters", body = RuleSearchResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Rules Engine",
)]
pub fn list_rules(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let params = query_parser::parse_query_string(query_string);

    let query = RuleQuery {
        category: query_parser::get_json(&params, "category"),
        trigger: query_parser::get_json(&params, "trigger"),
        status: query_parser::get_json(&params, "status"),
        jurisdiction: query_parser::get_string(&params, "jurisdiction"),
        source: query_parser::get_json(&params, "source"),
        offset: query_parser::get_usize(&params, "offset").unwrap_or(0),
        limit: query_parser::get_usize(&params, "limit").unwrap_or(50),
    };

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (rules, total) = repo.search_rules(query)?;

    let response = RuleSearchResponse { rules, total };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get rules by category
#[utoipa::path(
    get,
    path = "/api/rules/category/{category}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("category" = String, Path, description = "Rule category")
    ),
    responses(
        (status = 200, description = "Rules in the specified category", body = [Rule]),
        (status = 400, description = "Invalid category")
    ),
    tag = "Rules Engine",
)]
pub fn get_rules_by_category(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let category_str = params
        .get("category")
        .ok_or_else(|| ApiError::BadRequest("Category required".to_string()))?;

    let category: RuleCategory = serde_json::from_str(&format!("\"{}\"", category_str))?;

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let rules = repo.find_rules_by_category(category)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&rules)?)
        .build())
}

/// Get rules by trigger event
#[utoipa::path(
    get,
    path = "/api/rules/trigger/{trigger}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("trigger" = String, Path, description = "Trigger event type")
    ),
    responses(
        (status = 200, description = "Rules triggered by the specified event", body = [Rule]),
        (status = 400, description = "Invalid trigger event")
    ),
    tag = "Rules Engine",
)]
pub fn get_rules_by_trigger(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let trigger_str = params
        .get("trigger")
        .ok_or_else(|| ApiError::BadRequest("Trigger required".to_string()))?;

    let trigger: TriggerEvent = serde_json::from_str(&format!("\"{}\"", trigger_str))?;

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let rules = repo.find_rules_by_trigger(trigger)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&rules)?)
        .build())
}

/// Get active rules for a jurisdiction
#[utoipa::path(
    get,
    path = "/api/rules/jurisdiction/{jurisdiction}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("jurisdiction" = String, Path, description = "Jurisdiction identifier")
    ),
    responses(
        (status = 200, description = "Active rules for the specified jurisdiction", body = [Rule]),
        (status = 400, description = "Jurisdiction required")
    ),
    tag = "Rules Engine",
)]
pub fn get_active_rules_for_jurisdiction(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let jurisdiction = params
        .get("jurisdiction")
        .ok_or_else(|| ApiError::BadRequest("Jurisdiction required".to_string()))?;

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let rules = repo.find_active_rules_for_jurisdiction(jurisdiction)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&rules)?)
        .build())
}

/// Evaluate rules (Phase 2 placeholder)
#[utoipa::path(
    post,
    path = "/api/rules/evaluate",
    request_body = EvaluateRulesRequest,
    responses(
        (status = 200, description = "Rule evaluation results", body = EvaluateRulesResponse),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Rules Engine",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn evaluate_rules(_req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let response = EvaluateRulesResponse {
        message: "Rule evaluation is not yet implemented (Phase 2)".to_string(),
        evaluated_count: 0,
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Get a rule by ID
#[utoipa::path(
    get,
    path = "/api/rules/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Rule ID")
    ),
    responses(
        (status = 200, description = "Rule found", body = Rule),
        (status = 404, description = "Rule not found"),
        (status = 400, description = "Invalid rule ID")
    ),
    tag = "Rules Engine",
)]
pub fn get_rule(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid rule ID".to_string()))?;

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let rule = repo
        .find_rule_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Rule not found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&rule)?)
        .build())
}

/// Update a rule
#[utoipa::path(
    put,
    path = "/api/rules/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Rule ID")
    ),
    request_body = UpdateRuleRequest,
    responses(
        (status = 200, description = "Rule updated successfully", body = Rule),
        (status = 404, description = "Rule not found"),
        (status = 400, description = "Invalid request data")
    ),
    tag = "Rules Engine",
)]
pub fn update_rule(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid rule ID".to_string()))?;

    let body = req.body();
    let request: UpdateRuleRequest = serde_json::from_slice(body)?;

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let mut rule = repo
        .find_rule_by_id(id)?
        .ok_or_else(|| ApiError::NotFound("Rule not found".to_string()))?;

    // Apply partial updates
    if let Some(name) = request.name { rule.name = name; }
    if let Some(description) = request.description { rule.description = description; }
    if let Some(source) = request.source { rule.source = source; }
    if let Some(category) = request.category { rule.category = category; }
    if let Some(triggers) = request.triggers { rule.triggers = triggers; }
    if let Some(conditions) = request.conditions { rule.conditions = conditions; }
    if let Some(actions) = request.actions { rule.actions = actions; }
    if let Some(priority) = request.priority { rule.priority = priority; }
    if let Some(status) = request.status { rule.status = status; }
    if let Some(jurisdiction) = request.jurisdiction { rule.jurisdiction = Some(jurisdiction); }
    if let Some(citation) = request.citation { rule.citation = Some(citation); }
    if let Some(effective_date) = request.effective_date { rule.effective_date = Some(effective_date); }
    if let Some(expiration_date) = request.expiration_date { rule.expiration_date = Some(expiration_date); }
    if let Some(supersedes_rule_id) = request.supersedes_rule_id { rule.supersedes_rule_id = Some(supersedes_rule_id); }

    rule.updated_at = Utc::now();
    repo.save_rule(&rule)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&rule)?)
        .build())
}

/// Delete a rule
#[utoipa::path(
    delete,
    path = "/api/rules/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = Uuid, Path, description = "Rule ID")
    ),
    responses(
        (status = 200, description = "Rule deleted"),
        (status = 400, description = "Invalid rule ID")
    ),
    tag = "Rules Engine",
)]
pub fn delete_rule(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params
        .get("id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| ApiError::BadRequest("Invalid rule ID".to_string()))?;

    let repo = match RepositoryFactory::rules_repo_validated(&req) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let deleted = repo.delete_rule(id)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({"deleted": deleted}))?)
        .build())
}
