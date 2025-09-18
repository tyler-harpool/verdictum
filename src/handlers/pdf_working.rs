//! Working PDF generation endpoints that compile

use spin_sdk::http::{Params, Request, Response};
use crate::utils::simple_pdf;
use crate::utils::tenant;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateRule16bRequest {
    pub case_number: String,
    pub defendant_names: String,
    pub judge_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateOrderRequest {
    pub case_number: String,
    pub judge_name: String,
    pub order_title: String,
    pub order_content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateMinuteEntryRequest {
    pub case_number: String,
    pub docket_entry_description: String,
    pub judge_name: String,
}

/// Generate Rule 16(b) Scheduling Order PDF
#[utoipa::path(
    post,
    path = "/api/pdf/rule-16b-order",
    request_body = GenerateRule16bRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Invalid request")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_rule_16b_order(req: Request, _params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);

    let body = req.body().to_vec();
    let request: GenerateRule16bRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    // Generate PDF
    let pdf_bytes = simple_pdf::generate_simple_rule_16b_order(
        &request.case_number,
        &request.defendant_names,
        &request.judge_name,
        &tenant_id
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"Rule_16b_Order_{}.pdf\"", request.case_number))
        .body(pdf_bytes)
        .build()
}

/// Generate court order PDF
#[utoipa::path(
    post,
    path = "/api/pdf/court-order",
    request_body = GenerateOrderRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Invalid request")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_court_order(_req: Request, _params: Params) -> Response {
    Response::builder()
        .status(501)
        .header("content-type", "application/json")
        .body("Court order generation coming soon")
        .build()
}

/// Generate minute entry PDF
#[utoipa::path(
    post,
    path = "/api/pdf/minute-entry",
    request_body = GenerateMinuteEntryRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Invalid request")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_minute_entry(_req: Request, _params: Params) -> Response {
    Response::builder()
        .status(501)
        .header("content-type", "application/json")
        .body("Minute entry generation coming soon")
        .build()
}

/// Auto-generate Rule 16(b) order
#[utoipa::path(
    post,
    path = "/api/pdf/auto-rule-16b/{case_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = String, Path, description = "Criminal case ID", example = "23-cr-00123")
    ),
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf")
    ),
    tag = "pdf-generation"
)]
pub fn auto_generate_rule_16b(req: Request, params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or("23-cr-00123");

    // For testing, we'll use mock data based on the case_id
    let (case_number, defendants, judge) = match case_id {
        "23-cr-00123" => (
            "23-cr-00123",
            "John Doe, Jane Smith",
            "Hon. Sarah Johnson"
        ),
        "24-cr-00456" => (
            "24-cr-00456",
            "Robert Brown",
            "Hon. Michael Davis"
        ),
        _ => (
            case_id,
            "Unknown Defendant",
            "Hon. District Judge"
        )
    };

    // Generate PDF with the test data
    let pdf_bytes = simple_pdf::generate_simple_rule_16b_order(
        case_number,
        defendants,
        judge,
        &tenant_id
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"Rule_16b_Order_{}.pdf\"", case_number))
        .body(pdf_bytes)
        .build()
}