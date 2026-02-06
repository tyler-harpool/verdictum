//! Filing pipeline HTTP handlers
//!
//! This module provides HTTP endpoints for the document filing pipeline,
//! including submission with compliance validation, dry-run validation,
//! and jurisdiction listing.

use crate::adapters::deadline_engine_impl::FrcpDeadlineEngine;
use crate::adapters::privacy_engine_impl::FrcpPrivacyEngine;
use crate::adapters::rules_engine_impl::SpinRulesEngine;
use crate::domain::deadline_calc::{DeadlineComputeRequest, ServiceMethod};
use crate::domain::filing_pipeline::{
    ComplianceReport, FilingContext, FilingReceipt, FilingSubmission,
};
use crate::domain::nef::{DeliveryMethod, NefRecipient, NoticeOfElectronicFiling};
use crate::domain::rule::{RuleAction, TriggerEvent};
use crate::error::{ApiError, ApiResult};
use crate::ports::deadline_engine::DeadlineEngine;
use crate::ports::privacy_engine::PrivacyEngine;
use crate::ports::rules_engine::RulesEngine;
use crate::ports::rules_repository::RulesRepository;
use crate::utils::repository_factory::RepositoryFactory;
use chrono::Utc;
use serde::Serialize;
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use utoipa::ToSchema;
use uuid::Uuid;

/// Available jurisdictions list response
#[derive(Serialize, ToSchema)]
pub struct JurisdictionListResponse {
    /// List of available jurisdiction codes
    pub jurisdictions: Vec<String>,
}

/// Validation-only response (dry run)
#[derive(Serialize, ToSchema)]
pub struct ValidationResponse {
    /// Compliance report from validation
    pub compliance_report: ComplianceReport,
    /// Privacy scan result summary
    pub privacy_clean: bool,
    /// Whether the filing would be accepted
    pub would_accept: bool,
}

/// Submit a filing through the compliance pipeline
///
/// Runs privacy scanning, rules evaluation, and deadline computation
/// before accepting the filing. Returns a receipt with all results.
#[utoipa::path(
    post,
    path = "/api/filings",
    request_body = FilingSubmission,
    responses(
        (status = 201, description = "Filing accepted", body = FilingReceipt),
        (status = 400, description = "Invalid request or missing district header"),
        (status = 422, description = "Filing rejected due to privacy violations or rule blocks"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Filing Pipeline",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., arwd, sdny)", example = "arwd")
    ),
)]
pub fn submit_filing(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let submission: FilingSubmission = serde_json::from_slice(body)
        .map_err(|e| ApiError::BadRequest(format!("Invalid JSON: {}", e)))?;

    // Get rules repository for the district
    let rules_repo = RepositoryFactory::rules_repo_validated(&req)?;

    // Build filing context from submission
    let context = build_filing_context(&submission, &req)?;

    // Step 1: Privacy scan (if document text provided)
    let privacy_engine = FrcpPrivacyEngine::new();
    if let Some(ref doc_text) = submission.document_text {
        let scan_result = privacy_engine.scan(doc_text, &submission.document_type)?;
        if !scan_result.clean {
            return Ok(ResponseBuilder::new(422)
                .header("content-type", "application/json")
                .body(serde_json::to_vec(&scan_result)?)
                .build());
        }
    }

    // Step 2: Load rules and evaluate
    let all_rules = rules_repo.find_all_rules()
        .map_err(|e| ApiError::Internal(format!("Failed to load rules: {}", e)))?;

    let rules_engine = SpinRulesEngine::new();
    let applicable = rules_engine.select_rules(
        &context.jurisdiction_id,
        &TriggerEvent::DocumentFiled,
        &all_rules,
    );
    let sorted = rules_engine.resolve_priority(applicable);
    let mut compliance_report = rules_engine.evaluate(&context, &sorted)?;

    if compliance_report.blocked {
        return Ok(ResponseBuilder::new(422)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&compliance_report)?)
            .build());
    }

    // Step 3: Compute deadlines for GenerateDeadline actions
    let deadline_engine = FrcpDeadlineEngine::new();
    let today = Utc::now().date_naive();
    let service_method = context
        .service_method
        .clone()
        .unwrap_or(ServiceMethod::Electronic);

    for rule in &sorted {
        for action in &rule.actions {
            if let RuleAction::GenerateDeadline {
                description,
                days_from_trigger,
            } = action
            {
                let deadline_req = DeadlineComputeRequest {
                    trigger_date: today,
                    period_days: *days_from_trigger,
                    service_method: service_method.clone(),
                    jurisdiction: context.jurisdiction_id.clone(),
                    description: description.clone(),
                    rule_citation: rule.citation.clone().unwrap_or_default(),
                };

                match deadline_engine.compute_deadline(&deadline_req) {
                    Ok(deadline_result) => {
                        compliance_report.deadlines.push(deadline_result);
                    }
                    Err(e) => {
                        compliance_report
                            .warnings
                            .push(format!("Deadline computation failed: {}", e));
                    }
                }
            }
        }
    }

    // Step 4: Generate NEF record
    let filing_id = Uuid::new_v4();
    let nef = NoticeOfElectronicFiling {
        filing_id,
        case_number: submission.case_number.clone(),
        filed_at: Utc::now(),
        document_type: submission.document_type.clone(),
        filer_name: submission.filer_name.clone(),
        docket_text: format!(
            "{} filed by {}",
            submission.document_type, submission.filer_name
        ),
        recipients: vec![NefRecipient {
            name: submission.filer_name.clone(),
            email: None,
            delivery_method: DeliveryMethod::ElectronicNef,
            served_at: Some(Utc::now()),
        }],
    };

    // Step 5: Build receipt
    let receipt = FilingReceipt {
        filing_id,
        case_number: submission.case_number,
        filed_at: Utc::now(),
        document_type: submission.document_type,
        docket_number: None,
        compliance_report,
        nef: Some(nef),
    };

    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&receipt)?)
        .build())
}

/// Validate a filing without creating a docket entry (dry run)
///
/// Runs the same compliance pipeline as submit but does not persist
/// anything. Returns the compliance report for review.
#[utoipa::path(
    post,
    path = "/api/filings/validate",
    request_body = FilingSubmission,
    responses(
        (status = 200, description = "Validation results", body = ValidationResponse),
        (status = 400, description = "Invalid request or missing district header"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Filing Pipeline",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., arwd, sdny)", example = "arwd")
    ),
)]
pub fn validate_filing(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let submission: FilingSubmission = serde_json::from_slice(body)
        .map_err(|e| ApiError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let rules_repo = RepositoryFactory::rules_repo_validated(&req)?;
    let context = build_filing_context(&submission, &req)?;

    // Privacy scan
    let privacy_engine = FrcpPrivacyEngine::new();
    let privacy_clean = if let Some(ref doc_text) = submission.document_text {
        let scan_result = privacy_engine.scan(doc_text, &submission.document_type)?;
        scan_result.clean
    } else {
        true
    };

    // Rules evaluation
    let all_rules = rules_repo
        .find_all_rules()
        .map_err(|e| ApiError::Internal(format!("Failed to load rules: {}", e)))?;

    let rules_engine = SpinRulesEngine::new();
    let applicable = rules_engine.select_rules(
        &context.jurisdiction_id,
        &TriggerEvent::DocumentFiled,
        &all_rules,
    );
    let sorted = rules_engine.resolve_priority(applicable);
    let compliance_report = rules_engine.evaluate(&context, &sorted)?;

    let would_accept = !compliance_report.blocked && privacy_clean;

    let response = ValidationResponse {
        compliance_report,
        privacy_clean,
        would_accept,
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// List available jurisdictions
///
/// Returns a static list of jurisdiction codes that have rule packs configured.
#[utoipa::path(
    get,
    path = "/api/filings/jurisdictions",
    responses(
        (status = 200, description = "List of available jurisdictions", body = JurisdictionListResponse),
    ),
    tag = "Filing Pipeline",
)]
pub fn list_jurisdictions(
    _req: Request,
    _params: Params,
) -> ApiResult<impl IntoResponse> {
    let response = JurisdictionListResponse {
        jurisdictions: vec![
            "arwd".to_string(),
            "sdny".to_string(),
            "edny".to_string(),
            "ndca".to_string(),
        ],
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Build a FilingContext from a submission and request headers
fn build_filing_context(
    submission: &FilingSubmission,
    req: &Request,
) -> Result<FilingContext, ApiError> {
    let district = extract_district(req)?;

    Ok(FilingContext {
        case_type: submission
            .metadata
            .get("case_type")
            .and_then(|v| v.as_str())
            .unwrap_or("criminal")
            .to_string(),
        document_type: submission.document_type.clone(),
        filer_role: submission.filer_role.clone(),
        jurisdiction_id: district,
        division: submission
            .metadata
            .get("division")
            .and_then(|v| v.as_str())
            .map(String::from),
        assigned_judge: submission
            .metadata
            .get("assigned_judge")
            .and_then(|v| v.as_str())
            .map(String::from),
        service_method: None,
        metadata: submission.metadata.clone(),
    })
}

/// Extract district identifier from request headers
fn extract_district(req: &Request) -> Result<String, ApiError> {
    for (name, value) in req.headers() {
        let header_name = name.to_lowercase();
        if header_name == "x-court-district" || header_name == "x-tenant-id" {
            if let Ok(v) = std::str::from_utf8(value.as_ref()) {
                return Ok(v.to_lowercase());
            }
        }
    }
    Err(ApiError::BadRequest(
        "Missing required header: X-Court-District or X-Tenant-ID".to_string(),
    ))
}
