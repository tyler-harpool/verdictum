//! Batch PDF generation endpoints

use spin_sdk::http::{Request, Response, Params};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::utils::{court_document_generator, federal_forms, tenant};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchPdfRequest {
    pub documents: Vec<DocumentRequest>,
    /// How to return the PDFs: "base64" (default), "urls", "combined"
    #[serde(default = "default_output_format")]
    pub output_format: String,
    /// For combined format: how to separate PDFs in the single document
    #[serde(default)]
    pub combine_pages: bool,
}

fn default_output_format() -> String {
    "base64".to_string()
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum DocumentRequest {
    #[serde(rename = "rule16b")]
    Rule16b {
        case_number: String,
        defendant_names: String,
        judge_name: String,
        signature_base64: Option<String>,
    },
    #[serde(rename = "court_order")]
    CourtOrder {
        case_number: String,
        defendant_names: String,
        judge_name: String,
        order_title: String,
        order_content: String,
        signature_base64: Option<String>,
    },
    #[serde(rename = "minute_entry")]
    MinuteEntry {
        case_number: String,
        defendant_names: String,
        judge_name: String,
        minute_text: String,
    },
    #[serde(rename = "waiver_indictment")]
    WaiverIndictment {
        case_number: String,
        defendant_name: String,
        charges: String,
    },
    #[serde(rename = "conditions_release")]
    ConditionsRelease {
        case_number: String,
        defendant_name: String,
        judge_name: String,
        conditions: Vec<String>,
    },
    #[serde(rename = "criminal_judgment")]
    CriminalJudgment {
        case_number: String,
        defendant_name: String,
        judge_name: String,
        plea: String,
        counts: String,
        sentence: String,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchPdfResponse {
    pub documents: Vec<GeneratedDocument>,
    pub total_generated: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GeneratedDocument {
    pub document_type: String,
    pub case_number: String,
    pub filename: String,
    pub pdf_base64: String,
    pub size_bytes: usize,
}

/// Generate multiple PDFs in a single request
#[utoipa::path(
    post,
    path = "/api/pdf/batch",
    request_body = BatchPdfRequest,
    responses(
        (status = 200, description = "PDFs generated successfully", body = BatchPdfResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY)", example = "SDNY")
    ),
)]
pub fn generate_batch_pdfs(req: Request, _params: Params) -> Response {
    let district = tenant::get_tenant_id(&req);

    let body = req.body().to_vec();
    let request: BatchPdfRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!("Invalid request: {}", e))
                .build();
        }
    };

    let mut generated_documents = Vec::new();

    for doc_request in request.documents {
        let (pdf_bytes, document_type, case_number, filename) = match doc_request {
            DocumentRequest::Rule16b {
                case_number,
                defendant_names,
                judge_name,
                signature_base64
            } => {
                let pdf = if let Some(sig) = signature_base64 {
                    court_document_generator::generate_rule_16b_with_signature(
                        &case_number,
                        &defendant_names,
                        &judge_name,
                        &district,
                        Some(&sig)
                    )
                } else {
                    court_document_generator::generate_rule_16b_order(
                        &case_number,
                        &defendant_names,
                        &judge_name,
                        &district
                    )
                };
                (pdf, "rule16b", case_number.clone(), format!("rule16b-{}.pdf", case_number))
            },

            DocumentRequest::CourtOrder {
                case_number,
                defendant_names,
                judge_name,
                order_title,
                order_content,
                signature_base64
            } => {
                let pdf = if let Some(sig) = signature_base64 {
                    court_document_generator::generate_court_order_with_signature(
                        &case_number,
                        &defendant_names,
                        &judge_name,
                        &district,
                        &order_title,
                        &order_content,
                        Some(&sig)
                    )
                } else {
                    court_document_generator::generate_court_order(
                        &case_number,
                        &defendant_names,
                        &judge_name,
                        &district,
                        &order_title,
                        &order_content
                    )
                };
                (pdf, "court_order", case_number.clone(), format!("court-order-{}.pdf", case_number))
            },

            DocumentRequest::MinuteEntry {
                case_number,
                defendant_names,
                judge_name,
                minute_text
            } => {
                let pdf = court_document_generator::generate_minute_entry(
                    &case_number,
                    &defendant_names,
                    &judge_name,
                    &district,
                    &minute_text
                );
                (pdf, "minute_entry", case_number.clone(), format!("minute-entry-{}.pdf", case_number))
            },

            DocumentRequest::WaiverIndictment {
                case_number,
                defendant_name,
                charges
            } => {
                let pdf = federal_forms::generate_waiver_of_indictment(
                    &case_number,
                    &defendant_name,
                    &district,
                    &charges
                );
                (pdf, "waiver_indictment", case_number.clone(), format!("waiver-indictment-{}.pdf", case_number))
            },

            DocumentRequest::ConditionsRelease {
                case_number,
                defendant_name,
                judge_name,
                conditions
            } => {
                let conditions_refs: Vec<&str> = conditions.iter().map(|s| s.as_str()).collect();
                let pdf = federal_forms::generate_conditions_of_release(
                    &case_number,
                    &defendant_name,
                    &district,
                    &judge_name,
                    conditions_refs
                );
                (pdf, "conditions_release", case_number.clone(), format!("conditions-release-{}.pdf", case_number))
            },

            DocumentRequest::CriminalJudgment {
                case_number,
                defendant_name,
                judge_name,
                plea,
                counts,
                sentence
            } => {
                let pdf = federal_forms::generate_criminal_judgment(
                    &case_number,
                    &defendant_name,
                    &district,
                    &judge_name,
                    &plea,
                    &counts,
                    &sentence
                );
                (pdf, "criminal_judgment", case_number.clone(), format!("criminal-judgment-{}.pdf", case_number))
            },
        };

        let pdf_base64 = BASE64.encode(&pdf_bytes);

        generated_documents.push(GeneratedDocument {
            document_type: document_type.to_string(),
            case_number,
            filename,
            pdf_base64,
            size_bytes: pdf_bytes.len(),
        });
    }

    let response = BatchPdfResponse {
        total_generated: generated_documents.len(),
        documents: generated_documents,
    };

    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response).unwrap_or_default())
        .build()
}

/// Generate multiple PDFs as a ZIP file
#[utoipa::path(
    post,
    path = "/api/pdf/batch/zip",
    request_body = BatchPdfRequest,
    responses(
        (status = 200, description = "ZIP file containing all PDFs", content_type = "application/zip"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_batch_pdfs_zip(_req: Request, _params: Params) -> Response {
    // ZIP generation would require additional dependencies
    // For now, return not implemented
    Response::builder()
        .status(501)
        .header("content-type", "application/json")
        .body("ZIP generation not yet implemented. Use /api/pdf/batch for base64-encoded PDFs.")
        .build()
}