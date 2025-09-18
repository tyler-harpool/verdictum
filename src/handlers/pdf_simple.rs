//! Simple PDF generation endpoints for court documents

use spin_sdk::http::{Params, Request, Response};
use crate::utils::simple_pdf;
use crate::utils::json_response as json;
use crate::utils::tenant;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateRule16bRequest {
    pub case_number: String,
    pub defendant_names: String,
    pub judge_name: String,
}

/// Generate Rule 16(b) Scheduling Order PDF
#[utoipa::path(
    post,
    path = "/api/pdf/rule-16b-order",
    request_body = GenerateRule16bRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
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