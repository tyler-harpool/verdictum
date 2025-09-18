//! Court document PDF generation endpoints

use spin_sdk::http::{Params, Request, Response};
use crate::utils::court_document_generator;
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
    pub defendant_names: String,
    pub judge_name: String,
    pub order_title: String,
    pub order_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_base64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateMinuteEntryRequest {
    pub case_number: String,
    pub defendant_names: String,
    pub judge_name: String,
    pub minute_text: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateSignedRule16bRequest {
    #[schema(example = "23-cr-00123")]
    pub case_number: String,
    #[schema(example = "John Doe, Jane Smith")]
    pub defendant_names: String,
    #[schema(example = "Hon. Sarah Johnson")]
    pub judge_name: String,
    #[schema(example = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_base64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JudgeSignatureUpload {
    #[schema(example = "judge-123")]
    pub judge_id: String,
    #[schema(example = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==")]
    pub signature_base64: String,
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
    let pdf_bytes = court_document_generator::generate_rule_16b_order(
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
pub fn generate_court_order(req: Request, _params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);

    let body = req.body().to_vec();
    let request: GenerateOrderRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    let pdf_bytes = if let Some(sig) = &request.signature_base64 {
        court_document_generator::generate_court_order_with_signature(
            &request.case_number,
            &request.defendant_names,
            &request.judge_name,
            &tenant_id,
            &request.order_title,
            &request.order_content,
            Some(sig.as_str()),
        )
    } else {
        court_document_generator::generate_court_order(
            &request.case_number,
            &request.defendant_names,
            &request.judge_name,
            &tenant_id,
            &request.order_title,
            &request.order_content,
        )
    };

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"court-order-{}.pdf\"", request.case_number))
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
        (status = 400, description = "Invalid request")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_minute_entry(req: Request, _params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);

    let body = req.body().to_vec();
    let request: GenerateMinuteEntryRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    let pdf_bytes = court_document_generator::generate_minute_entry(
        &request.case_number,
        &request.defendant_names,
        &request.judge_name,
        &tenant_id,
        &request.minute_text,
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"minute-entry-{}.pdf\"", request.case_number))
        .body(pdf_bytes)
        .build()
}

/// Generate Rule 16(b) order with judge signature
#[utoipa::path(
    post,
    path = "/api/pdf/rule-16b-signed",
    request_body = GenerateSignedRule16bRequest,
    responses(
        (status = 200, description = "PDF generated successfully with signature", content_type = "application/pdf"),
        (status = 400, description = "Invalid request")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn generate_signed_rule_16b_order(req: Request, _params: Params) -> Response {
    let tenant_id = tenant::get_tenant_id(&req);

    let body = req.body().to_vec();
    let request: GenerateSignedRule16bRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    // Generate PDF with signature if provided
    let pdf_bytes = court_document_generator::generate_rule_16b_with_signature(
        &request.case_number,
        &request.defendant_names,
        &request.judge_name,
        &tenant_id,
        request.signature_base64.as_deref()
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"Rule_16b_Order_Signed_{}.pdf\"", request.case_number))
        .body(pdf_bytes)
        .build()
}

/// Auto-generate Rule 16(b) order
#[utoipa::path(
    post,
    path = "/api/pdf/auto/rule-16b/{case_id}",
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
    let (case_number, defendants, judge, signature) = match case_id {
        "23-cr-00123" => (
            "23-cr-00123",
            "John Doe, Jane Smith",
            "Hon. Sarah Johnson",
            None  // Could add a test signature here
        ),
        "24-cr-00456" => (
            "24-cr-00456",
            "Robert Brown",
            "Hon. Michael Davis",
            None
        ),
        _ => (
            case_id,
            "Unknown Defendant",
            "Hon. District Judge",
            None
        )
    };

    // Generate PDF with optional signature
    let pdf_bytes = court_document_generator::generate_rule_16b_with_signature(
        case_number,
        defendants,
        judge,
        &tenant_id,
        signature
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("inline; filename=\"Rule_16b_Order_{}.pdf\"", case_number))
        .body(pdf_bytes)
        .build()
}

/// Upload judge signature for automatic document signing
#[utoipa::path(
    post,
    path = "/api/pdf/judge-signature",
    request_body = JudgeSignatureUpload,
    responses(
        (status = 200, description = "Signature uploaded successfully"),
        (status = 400, description = "Invalid request")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn upload_judge_signature(req: Request, _params: Params) -> Response {
    let _tenant_id = tenant::get_tenant_id(&req);

    let body = req.body().to_vec();
    let _upload: JudgeSignatureUpload = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(format!("Invalid request: {}", e))
            .build(),
    };

    // In a real implementation, you would store the signature in a key-value store
    // For now, we'll just return success
    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(r#"{"message": "Judge signature uploaded successfully"}"#)
        .build()
}