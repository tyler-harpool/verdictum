//! PDF generation endpoints for court documents

use spin_sdk::http::{Params, Request, Response};
use crate::utils::pdf_generator;
use crate::utils::json_response as json;
use crate::utils::repository_factory::RepositoryFactory;
use crate::utils::tenant;
use crate::ports::case_repository::CaseRepository;
use crate::ports::judge_repository::JudgeRepository;
use crate::ports::docket_repository::DocketRepository;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateRule16bRequest {
    pub case_id: String,
    pub judge_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateOrderRequest {
    pub case_id: String,
    pub judge_id: String,
    pub order_title: String,
    pub order_content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateMinuteEntryRequest {
    pub case_id: String,
    pub docket_entry_number: i32,
    pub judge_id: String,
}

/// Generate Rule 16(b) Scheduling Order PDF
#[utoipa::path(
    post,
    path = "/api/pdf/rule-16b-order",
    request_body = GenerateRule16bRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Case or judge not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_rule_16b_order(req: Request, _params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);

    let request: GenerateRule16bRequest = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    // Get repositories
    let case_repo = RepositoryFactory::case_repo(&req);
    let judge_repo = RepositoryFactory::judge_repo(&req);

    // Fetch case
    let case = match case_repo.find_by_id(&request.case_id) {
        Ok(Some(c)) => c,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Case not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching case: {}", e))
            .build(),
    };

    // Parse and fetch judge
    let judge_uuid = match Uuid::parse_str(&request.judge_id) {
        Ok(id) => id,
        Err(_) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body("Invalid judge ID format")
            .build(),
    };

    let judge = match judge_repo.find_judge_by_id(judge_uuid) {
        Ok(Some(j)) => j,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Judge not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching judge: {}", e))
            .build(),
    };

    // Generate PDF
    let pdf_bytes = pdf_generator::generate_rule_16b_order(&case, &judge, &tenant_id);

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"Rule_16b_Order_{}.pdf\"", case.case_number))
        .body(pdf_bytes)
        .build()
}

/// Generate custom court order PDF
#[utoipa::path(
    post,
    path = "/api/pdf/court-order",
    request_body = GenerateOrderRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Case or judge not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_court_order(req: Request, _params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);

    let request: GenerateOrderRequest = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    // Get repositories
    let case_repo = RepositoryFactory::case_repo(&req);
    let judge_repo = RepositoryFactory::judge_repo(&req);

    // Fetch case
    let case = match case_repo.find_by_id(&request.case_id) {
        Ok(Some(c)) => c,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Case not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching case: {}", e))
            .build(),
    };

    // Parse and fetch judge
    let judge_uuid = match Uuid::parse_str(&request.judge_id) {
        Ok(id) => id,
        Err(_) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body("Invalid judge ID format")
            .build(),
    };

    let judge = match judge_repo.find_judge_by_id(judge_uuid) {
        Ok(Some(j)) => j,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Judge not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching judge: {}", e))
            .build(),
    };

    // Generate PDF
    let pdf_bytes = pdf_generator::generate_criminal_order(
        &case,
        &judge,
        &request.order_title,
        &request.order_content,
        &tenant_id
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"{}.pdf\"",
            request.order_title.replace(" ", "_")))
        .body(pdf_bytes)
        .build()
}

/// Generate minute entry PDF
#[utoipa::path(
    post,
    path = "/api/pdf/minute-entry",
    request_body = GenerateMinuteEntryRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Case, docket entry or judge not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_minute_entry(req: Request, _params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);

    let request: GenerateMinuteEntryRequest = match json::parse_body(req.body()) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    // Get repositories
    let case_repo = RepositoryFactory::case_repo(&req);
    let judge_repo = RepositoryFactory::judge_repo(&req);
    let docket_repo = RepositoryFactory::docket_repo(&req);

    // Fetch case
    let case = match case_repo.find_by_id(&request.case_id) {
        Ok(Some(c)) => c,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Case not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching case: {}", e))
            .build(),
    };

    // Fetch docket entry
    let entries = match docket_repo.find_entries_by_case(&request.case_id) {
        Ok(entries) => entries,
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching docket entries: {}", e))
            .build(),
    };

    let entry = match entries.iter().find(|e| e.entry_number == request.docket_entry_number) {
        Some(e) => e,
        None => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Docket entry not found")
            .build(),
    };

    // Parse and fetch judge
    let judge_uuid = match Uuid::parse_str(&request.judge_id) {
        Ok(id) => id,
        Err(_) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body("Invalid judge ID format")
            .build(),
    };

    let judge = match judge_repo.find_judge_by_id(judge_uuid) {
        Ok(Some(j)) => j,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Judge not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching judge: {}", e))
            .build(),
    };

    // Generate PDF
    let pdf_bytes = pdf_generator::generate_minute_entry_pdf(&case, entry, &judge, &tenant_id);

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"Minute_Entry_{}_{}.pdf\"",
            case.case_number, entry.entry_number))
        .body(pdf_bytes)
        .build()
}

/// Generate Rule 16(b) order automatically when case is assigned
#[utoipa::path(
    post,
    path = "/api/pdf/auto-rule-16b/{case_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = String, Path, description = "Criminal case ID")
    ),
    responses(
        (status = 200, description = "PDF generated and docketed successfully", content_type = "application/pdf"),
        (status = 404, description = "Case not found or no judge assigned"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation"
)]
pub fn auto_generate_rule_16b(req: Request, params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or_default();

    // Get repositories
    let case_repo = RepositoryFactory::case_repo(&req);
    let judge_repo = RepositoryFactory::judge_repo(&req);
    let docket_repo = RepositoryFactory::docket_repo(&req);

    // Fetch case
    let mut case = match case_repo.find_by_id(case_id) {
        Ok(Some(c)) => c,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Case not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching case: {}", e))
            .build(),
    };

    // Check if judge is assigned
    let judge_id = match &case.assigned_judge {
        Some(id) => id.clone(),
        None => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("No judge assigned to case")
            .build(),
    };

    // Parse judge ID as UUID
    let judge_uuid = match Uuid::parse_str(&judge_id) {
        Ok(id) => id,
        Err(_) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body("Invalid judge ID format in case")
            .build(),
    };

    // Fetch judge
    let judge = match judge_repo.find_judge_by_id(judge_uuid) {
        Ok(Some(j)) => j,
        Ok(None) => return Response::builder()
            .status(404)
            .header("content-type", "application/json")
            .body("Judge not found")
            .build(),
        Err(e) => return Response::builder()
            .status(500)
            .header("content-type", "application/json")
            .body(format!("Error fetching judge: {}", e))
            .build(),
    };

    // Generate PDF
    let pdf_bytes = pdf_generator::generate_rule_16b_order(&case, &judge, &tenant_id);

    // Add docket entry for the generated order
    let docket_entry = crate::domain::docket::DocketEntry::new(
        case_id.to_string(),
        "Scheduling Order pursuant to Fed. R. Crim. P. 16(b) (Auto-generated by CM/ECF)".to_string(),
        crate::domain::docket::DocketEntryType::Order,
        Some("Clerk of Court".to_string())
    );

    if let Err(e) = docket_repo.create_entry(docket_entry) {
        // Log error but continue - PDF generation was successful
        eprintln!("Failed to add docket entry: {}", e);
    }

    // Update case status if needed
    case.status = crate::domain::criminal_case::CaseStatus::Active;
    if let Err(e) = case_repo.update(&case) {
        // Log error but continue
        eprintln!("Failed to update case status: {}", e);
    }

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"Rule_16b_Order_{}.pdf\"", case.case_number))
        .body(pdf_bytes)
        .build()
}