//! Administrative handlers for multi-tenant operations

use crate::adapters::{
    spin_kv_case_repository::SpinKvCaseRepository,
    spin_kv_deadline_repository::SpinKvDeadlineRepository,
    spin_kv_docket_repository::SpinKvDocketRepository,
    spin_kv_judge_repository::SpinKvJudgeRepository,
};
use crate::error::ApiResult;
use crate::ports::case_repository::CaseRepository;
use crate::ports::deadline_repository::DeadlineRepository;
use crate::ports::judge_repository::JudgeRepository;
use crate::utils::query_parser;
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};

/// Initialize a new tenant's data stores
#[utoipa::path(
    post,
    path = "/api/admin/tenants/init",
    params(
        ("tenant_id" = Option<String>, Query, description = "Tenant identifier (defaults to 'default')")
    ),
    responses(
        (status = 200, description = "Tenant data stores initialized successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Administration"
)]
pub fn init_tenant(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let params = query_parser::parse_query_string(query_string);

    let tenant_id = query_parser::get_string(&params, "tenant_id")
        .unwrap_or_else(|| "default".to_string());

    // Initialize repositories with tenant-specific stores
    let _case_repo = SpinKvCaseRepository::with_store(format!("cases_{}", tenant_id));
    let _deadline_repo = SpinKvDeadlineRepository::with_store(format!("deadlines_{}", tenant_id));
    let _docket_repo = SpinKvDocketRepository::with_store(format!("docket_{}", tenant_id));
    let _judge_repo = SpinKvJudgeRepository::with_store(format!("judges_{}", tenant_id));

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({
            "tenant_id": tenant_id,
            "stores_initialized": ["cases", "deadlines", "docket", "judges"]
        }))?)
        .build())
}

/// Get tenant statistics
#[utoipa::path(
    get,
    path = "/api/admin/tenants/stats",
    params(
        ("tenant_id" = Option<String>, Query, description = "Tenant identifier (defaults to 'default')")
    ),
    responses(
        (status = 200, description = "Tenant statistics including counts of cases, deadlines, judges, etc."),
        (status = 500, description = "Internal server error")
    ),
    tag = "Administration"
)]
pub fn get_tenant_stats(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let query_string = req.query();
    let params = query_parser::parse_query_string(query_string);

    let tenant_id = query_parser::get_string(&params, "tenant_id")
        .unwrap_or_else(|| "default".to_string());

    // Use tenant-specific repositories
    let case_repo = SpinKvCaseRepository::with_store(format!("cases_{}", tenant_id));
    let deadline_repo = SpinKvDeadlineRepository::with_store(format!("deadlines_{}", tenant_id));
    let _docket_repo = SpinKvDocketRepository::with_store(format!("docket_{}", tenant_id));
    let judge_repo = SpinKvJudgeRepository::with_store(format!("judges_{}", tenant_id));

    // Get counts from each repository (these would be actual count methods in production)
    let case_count = case_repo.find_all_cases().map(|cases| cases.len()).unwrap_or(0);
    let deadline_count = deadline_repo.find_deadlines_by_status(crate::domain::deadline::DeadlineStatus::Pending)
        .map(|d| d.len()).unwrap_or(0);
    let docket_count = 0; // No find_all method for docket entries - would need to iterate cases
    let judge_count = judge_repo.find_all_judges().map(|j| j.len()).unwrap_or(0);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({
            "tenant_id": tenant_id,
            "statistics": {
                "cases": case_count,
                "pending_deadlines": deadline_count,
                "docket_entries": docket_count,
                "judges": judge_count
            }
        }))?)
        .build())
}