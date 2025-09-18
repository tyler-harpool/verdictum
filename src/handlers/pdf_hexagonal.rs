//! PDF generation handlers following hexagonal architecture

use spin_sdk::http::{Request, Response, Params};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domain::document::{
    CaseNumber, JudgeName, District,
    DocumentType, DocumentMetadata, ElectronicSignature, DocumentError
};
use crate::ports::document_generator::DocumentRequest;
use crate::services::pdf_service::create_pdf_service;
use crate::utils::tenant;

/// Helper function to determine response format from URL parameter or Accept header
fn wants_pdf(req: &Request, params: &Params) -> bool {
    // First check URL parameter
    if let Some(format) = params.get("format") {
        return format == "pdf";
    }
    // Fall back to Accept header for backward compatibility
    req.header("accept")
        .and_then(|v| v.as_str())
        .map(|accept| accept.contains("application/pdf"))
        .unwrap_or(false)
}

/// Helper function to build response based on format parameter or Accept header
fn build_response(req: &Request, params: &Params, generated: crate::domain::document::GeneratedDocument, doc_type: &str, case_number: String) -> Response {
    if wants_pdf(req, params) {
        // Return raw PDF
        Response::builder()
            .status(200)
            .header("content-type", "application/pdf")
            .header("content-disposition", format!(r#"attachment; filename="{}""#, generated.filename))
            .body(generated.pdf_data)
            .build()
    } else {
        // Return JSON with base64 PDF
        let response = PdfResponse {
            case_number,
            document_type: doc_type.to_string(),
            filename: generated.filename.clone(),
            pdf_base64: generated.to_base64(),
            size_bytes: generated.pdf_data.len(),
            document_id: generated.document.id.as_uuid().to_string(),
        };

        Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&response).unwrap_or_default())
            .build()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Rule16bRequest {
    pub case_number: String,
    pub defendant_names: String,
    pub judge_name: String,
    #[serde(default)]
    pub trial_date: Option<String>,
    #[serde(default)]
    pub discovery_deadline: Option<String>,
    #[serde(default)]
    pub motion_deadline: Option<String>,
    #[serde(default)]
    pub pretrial_conference_date: Option<String>,
    #[serde(default)]
    pub signature_base64: Option<String>,
    #[serde(default)]
    pub judge_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CourtOrderRequest {
    pub case_number: String,
    pub defendant_names: String,
    pub judge_name: String,
    pub order_title: String,
    pub order_content: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub signature_base64: Option<String>,
    #[serde(default)]
    pub judge_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MinuteEntryRequest {
    pub case_number: String,
    pub defendant_names: String,
    pub judge_name: String,
    pub minute_text: String,
    #[serde(default)]
    pub hearing_date: Option<String>,
    #[serde(default)]
    pub hearing_type: Option<String>,
    #[serde(default)]
    pub court_reporter: Option<String>,
    #[serde(default)]
    pub next_hearing: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WaiverIndictmentRequest {
    pub case_number: String,
    pub defendant_name: String,
    pub charges: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConditionsReleaseRequest {
    pub case_number: String,
    pub defendant_name: String,
    pub judge_name: String,
    pub conditions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CriminalJudgmentRequest {
    pub case_number: String,
    pub defendant_name: String,
    pub judge_name: String,
    pub plea: String,
    pub counts: String,
    pub sentence: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PdfResponse {
    pub case_number: String,
    pub document_type: String,
    pub filename: String,
    pub pdf_base64: String,
    pub size_bytes: usize,
    pub document_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchPdfRequest {
    pub documents: Vec<DocumentRequestDto>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum DocumentRequestDto {
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
    pub documents: Vec<PdfResponse>,
    pub total_generated: usize,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct StoreSignatureRequest {
    pub judge_id: String,
    pub signature_base64: String,
}

fn create_document_request(
    dto: DocumentRequestDto,
    district: District,
) -> Result<DocumentRequest, DocumentError> {
    let (case_number, document_type, metadata) = match dto {
        DocumentRequestDto::Rule16b {
            case_number,
            defendant_names,
            judge_name,
            signature_base64,
        } => {
            let signature = signature_base64.map(|sig| ElectronicSignature::new(judge_name.clone(), &sig));
            (
                CaseNumber::new(case_number)?,
                DocumentType::Rule16b,
                DocumentMetadata::Rule16b {
                    defendant_names,
                    judge_name: JudgeName::new(judge_name)?,
                    signature,
                },
            )
        }
        DocumentRequestDto::CourtOrder {
            case_number,
            defendant_names,
            judge_name,
            order_title,
            order_content,
            signature_base64,
        } => {
            let signature = signature_base64.map(|sig| ElectronicSignature::new(judge_name.clone(), &sig));
            (
                CaseNumber::new(case_number)?,
                DocumentType::CourtOrder,
                DocumentMetadata::CourtOrder {
                    defendant_names,
                    judge_name: JudgeName::new(judge_name)?,
                    order_title,
                    order_content,
                    signature,
                },
            )
        }
        DocumentRequestDto::MinuteEntry {
            case_number,
            defendant_names,
            judge_name,
            minute_text,
        } => (
            CaseNumber::new(case_number)?,
            DocumentType::MinuteEntry,
            DocumentMetadata::MinuteEntry {
                defendant_names,
                judge_name: JudgeName::new(judge_name)?,
                minute_text,
            },
        ),
        DocumentRequestDto::WaiverIndictment {
            case_number,
            defendant_name,
            charges,
        } => (
            CaseNumber::new(case_number)?,
            DocumentType::WaiverIndictment,
            DocumentMetadata::WaiverIndictment {
                defendant_name,
                charges,
            },
        ),
        DocumentRequestDto::ConditionsRelease {
            case_number,
            defendant_name,
            judge_name,
            conditions,
        } => (
            CaseNumber::new(case_number)?,
            DocumentType::ConditionsRelease,
            DocumentMetadata::ConditionsRelease {
                defendant_name,
                judge_name: JudgeName::new(judge_name)?,
                conditions,
            },
        ),
        DocumentRequestDto::CriminalJudgment {
            case_number,
            defendant_name,
            judge_name,
            plea,
            counts,
            sentence,
        } => (
            CaseNumber::new(case_number)?,
            DocumentType::CriminalJudgment,
            DocumentMetadata::CriminalJudgment {
                defendant_name,
                judge_name: JudgeName::new(judge_name)?,
                plea,
                counts,
                sentence,
            },
        ),
    };

    Ok(DocumentRequest {
        case_number,
        document_type,
        district,
        metadata,
    })
}

/// Generate a signed Rule 16(b) order
///
/// Returns either raw PDF or JSON with base64 PDF based on Accept header:
/// - Accept: application/pdf → Raw PDF binary
/// - Accept: application/json → JSON with base64-encoded PDF
#[utoipa::path(
    post,
    path = "/api/pdf/signed/rule16b/{format}",
    request_body = Rule16bRequest,
    responses(
        (status = 200, description = "PDF generated successfully (JSON format)", body = PdfResponse, content_type = "application/json"),
        (status = 200, description = "PDF generated successfully (PDF format)", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("format" = String, Path, description = "Response format: 'pdf' for raw PDF, 'json' for base64-encoded JSON", example = "pdf"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_signed_rule16b(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let body = req.body().to_vec();
    let mut request: Rule16bRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid request: {}"}}"#, e))
                .build();
        }
    };

    // Try to fetch stored signature for the judge
    // Extract judge ID from judge name (assuming format "Hon. John Smith" -> use initials or full name as ID)
    // For demo, we'll use a hardcoded judge_id
    let judge_id = uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap_or_else(|_| uuid::Uuid::new_v4());

    // Create PDF service to get signature
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Try to get stored signature
    if request.signature_base64.is_none() {
        if let Ok(Some(signature)) = service.get_signature_sync(judge_id) {
            request.signature_base64 = Some(signature.signature_base64);
        }
    }

    // Now create the document with the signature
    let doc_request = match create_document_request(
        DocumentRequestDto::Rule16b {
            case_number: request.case_number.clone(),
            defendant_names: request.defendant_names,
            judge_name: request.judge_name,
            signature_base64: request.signature_base64,
        },
        district,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    // Generate the document
    let generated = match service.generate_document_sync(doc_request) {
        Ok(doc) => doc,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Failed to generate PDF: {}"}}"#, e))
                .build();
        }
    };

    build_response(&req, &params, generated, "rule16b-signed", request.case_number)
}

/// Generate a court order
///
/// Returns either raw PDF or JSON with base64 PDF based on Accept header:
/// - Accept: application/pdf → Raw PDF binary
/// - Accept: application/json → JSON with base64-encoded PDF
#[utoipa::path(
    post,
    path = "/api/pdf/court-order/{format}",
    request_body = CourtOrderRequest,
    responses(
        (status = 200, description = "PDF generated successfully (JSON format)", body = PdfResponse, content_type = "application/json"),
        (status = 200, description = "PDF generated successfully (PDF format)", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("format" = String, Path, description = "Response format: 'pdf' for raw PDF, 'json' for base64-encoded JSON", example = "pdf"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_court_order(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let body = req.body().to_vec();
    let request: CourtOrderRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid request: {}"}}"#, e))
                .build();
        }
    };

    let doc_request = match create_document_request(
        DocumentRequestDto::CourtOrder {
            case_number: request.case_number.clone(),
            defendant_names: request.defendant_names,
            judge_name: request.judge_name,
            order_title: request.order_title,
            order_content: request.order_content,
            signature_base64: request.signature_base64,
        },
        district,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    // Create PDF service with dependency injection
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Use the service to generate the document
    let generated = match service.generate_document_sync(doc_request) {
        Ok(g) => g,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Generation failed: {}"}}"#, e))
                .build();
        }
    };

    build_response(&req, &params, generated, "court_order", request.case_number)
}

/// Generate a minute entry
///
/// Returns either raw PDF or JSON with base64 PDF based on Accept header:
/// - Accept: application/pdf → Raw PDF binary
/// - Accept: application/json → JSON with base64-encoded PDF
#[utoipa::path(
    post,
    path = "/api/pdf/minute-entry/{format}",
    request_body = MinuteEntryRequest,
    responses(
        (status = 200, description = "PDF generated successfully (JSON format)", body = PdfResponse, content_type = "application/json"),
        (status = 200, description = "PDF generated successfully (PDF format)", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("format" = String, Path, description = "Response format: 'pdf' for raw PDF, 'json' for base64-encoded JSON", example = "pdf"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_minute_entry(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let body = req.body().to_vec();
    let request: MinuteEntryRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid request: {}"}}"#, e))
                .build();
        }
    };

    let doc_request = match create_document_request(
        DocumentRequestDto::MinuteEntry {
            case_number: request.case_number.clone(),
            defendant_names: request.defendant_names,
            judge_name: request.judge_name,
            minute_text: request.minute_text,
        },
        district,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    // Create PDF service with dependency injection
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Use the service to generate the document
    let generated = match service.generate_document_sync(doc_request) {
        Ok(g) => g,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Generation failed: {}"}}"#, e))
                .build();
        }
    };

    build_response(&req, &params, generated, "minute_entry", request.case_number)
}

/// Generate a Rule 16(b) scheduling order
///
/// Returns either raw PDF or JSON with base64 PDF based on Accept header:
/// - Accept: application/pdf → Raw PDF binary
/// - Accept: application/json → JSON with base64-encoded PDF
#[utoipa::path(
    post,
    path = "/api/pdf/rule16b/{format}",
    request_body = Rule16bRequest,
    responses(
        (status = 200, description = "PDF generated successfully (JSON format)", body = PdfResponse, content_type = "application/json"),
        (status = 200, description = "PDF generated successfully (PDF format)", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("format" = String, Path, description = "Response format: 'pdf' for raw PDF, 'json' for base64-encoded JSON", example = "pdf"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_rule16b(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let body = req.body().to_vec();
    let request: Rule16bRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid request: {}"}}"#, e))
                .build();
        }
    };

    let doc_request = match create_document_request(
        DocumentRequestDto::Rule16b {
            case_number: request.case_number.clone(),
            defendant_names: request.defendant_names,
            judge_name: request.judge_name,
            signature_base64: request.signature_base64,
        },
        district,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    // Create PDF service with dependency injection
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Use the service to generate the document
    let generated = match service.generate_document_sync(doc_request) {
        Ok(g) => g,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Generation failed: {}"}}"#, e))
                .build();
        }
    };

    build_response(&req, &params, generated, "rule16b", request.case_number)
}

/// Generate waiver of indictment
///
/// Returns either raw PDF or JSON with base64 PDF based on Accept header:
/// - Accept: application/pdf → Raw PDF binary
/// - Accept: application/json → JSON with base64-encoded PDF
#[utoipa::path(
    post,
    path = "/api/pdf/waiver-indictment/{format}",
    request_body = WaiverIndictmentRequest,
    responses(
        (status = 200, description = "PDF generated successfully (JSON format)", body = PdfResponse, content_type = "application/json"),
        (status = 200, description = "PDF generated successfully (PDF format)", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("format" = String, Path, description = "Response format: 'pdf' for raw PDF, 'json' for base64-encoded JSON", example = "pdf"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_waiver_indictment(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let case_id = params.get("case_id").unwrap_or("24-cr-00001");
    let body = req.body().to_vec();
    let request: WaiverIndictmentRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            // Use default values for testing
            WaiverIndictmentRequest {
                case_number: case_id.to_string(),
                defendant_name: "Test Defendant".to_string(),
                charges: "18 U.S.C. § 1343 (Wire Fraud)".to_string(),
            }
        }
    };

    let doc_request = match create_document_request(
        DocumentRequestDto::WaiverIndictment {
            case_number: request.case_number.clone(),
            defendant_name: request.defendant_name,
            charges: request.charges,
        },
        district,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    // Create PDF service with dependency injection
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Use the service to generate the document
    let generated = match service.generate_document_sync(doc_request) {
        Ok(g) => g,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Generation failed: {}"}}"#, e))
                .build();
        }
    };

    build_response(&req, &params, generated, "waiver_indictment", request.case_number)
}

/// Generate conditions of release
///
/// Returns either raw PDF or JSON with base64 PDF based on Accept header:
/// - Accept: application/pdf → Raw PDF binary
/// - Accept: application/json → JSON with base64-encoded PDF
#[utoipa::path(
    post,
    path = "/api/pdf/conditions-release/{format}",
    request_body = ConditionsReleaseRequest,
    responses(
        (status = 200, description = "PDF generated successfully (JSON format)", body = PdfResponse, content_type = "application/json"),
        (status = 200, description = "PDF generated successfully (PDF format)", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("format" = String, Path, description = "Response format: 'pdf' for raw PDF, 'json' for base64-encoded JSON", example = "pdf"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_conditions_release(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let case_id = params.get("case_id").unwrap_or("24-cr-00001");
    let body = req.body().to_vec();
    let request: ConditionsReleaseRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            // Use default values for testing
            ConditionsReleaseRequest {
                case_number: case_id.to_string(),
                defendant_name: "Test Defendant".to_string(),
                judge_name: "Hon. Test Judge".to_string(),
                conditions: vec![
                    "Report to Pretrial Services as directed".to_string(),
                    "Surrender passport".to_string(),
                    "No contact with co-defendants".to_string(),
                ],
            }
        }
    };

    let doc_request = match create_document_request(
        DocumentRequestDto::ConditionsRelease {
            case_number: request.case_number.clone(),
            defendant_name: request.defendant_name,
            judge_name: request.judge_name,
            conditions: request.conditions,
        },
        district,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    // Create PDF service with dependency injection
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Use the service to generate the document
    let generated = match service.generate_document_sync(doc_request) {
        Ok(g) => g,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Generation failed: {}"}}"#, e))
                .build();
        }
    };

    build_response(&req, &params, generated, "conditions_release", request.case_number)
}

/// Generate criminal judgment
///
/// Returns either raw PDF or JSON with base64 PDF based on Accept header:
/// - Accept: application/pdf → Raw PDF binary
/// - Accept: application/json → JSON with base64-encoded PDF
#[utoipa::path(
    post,
    path = "/api/pdf/criminal-judgment/{format}",
    request_body = CriminalJudgmentRequest,
    responses(
        (status = 200, description = "PDF generated successfully (JSON format)", body = PdfResponse, content_type = "application/json"),
        (status = 200, description = "PDF generated successfully (PDF format)", content_type = "application/pdf"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation",
    params(
        ("format" = String, Path, description = "Response format: 'pdf' for raw PDF, 'json' for base64-encoded JSON", example = "pdf"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_criminal_judgment(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let case_id = params.get("case_id").unwrap_or("24-cr-00001");
    let body = req.body().to_vec();
    let request: CriminalJudgmentRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            // Use default values for testing
            CriminalJudgmentRequest {
                case_number: case_id.to_string(),
                defendant_name: "Test Defendant".to_string(),
                judge_name: "Hon. Test Judge".to_string(),
                plea: "guilty".to_string(),
                counts: "1-3".to_string(),
                sentence: "60 months imprisonment, 3 years supervised release".to_string(),
            }
        }
    };

    let doc_request = match create_document_request(
        DocumentRequestDto::CriminalJudgment {
            case_number: request.case_number.clone(),
            defendant_name: request.defendant_name,
            judge_name: request.judge_name,
            plea: request.plea,
            counts: request.counts,
            sentence: request.sentence,
        },
        district,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    // Create PDF service with dependency injection
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Use the service to generate the document
    let generated = match service.generate_document_sync(doc_request) {
        Ok(g) => g,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Generation failed: {}"}}"#, e))
                .build();
        }
    };

    build_response(&req, &params, generated, "criminal_judgment", request.case_number)
}

/// Generate multiple PDFs in a single request
///
/// Always returns JSON with base64-encoded PDFs (batch operations don't support raw PDF response)
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
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn generate_batch_pdfs(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let district = match District::new(district_str.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "{}"}}"#, e))
                .build();
        }
    };

    let body = req.body().to_vec();
    let request: BatchPdfRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid request: {}"}}"#, e))
                .build();
        }
    };

    let mut documents = Vec::new();
    for dto in request.documents {
        let doc_request = match create_document_request(dto, district.clone()) {
            Ok(r) => r,
            Err(e) => {
                return Response::builder()
                    .status(400)
                    .header("content-type", "application/json")
                    .body(format!(r#"{{"error": "{}"}}"#, e))
                    .build();
            }
        };
        documents.push(doc_request.to_court_document());
    }

    // Create PDF service with dependency injection
    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    // Use the service to generate documents
    let doc_requests: Vec<DocumentRequest> = documents.into_iter()
        .map(|d| DocumentRequest {
            case_number: d.case_number,
            document_type: d.document_type,
            district: d.district,
            metadata: d.metadata,
        })
        .collect();

    let generated_docs = match service.generate_batch_sync(doc_requests) {
        Ok(docs) => docs,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Batch generation failed: {}"}}"#, e))
                .build();
        }
    };

    let mut pdf_responses = Vec::new();
    for generated in generated_docs {
        pdf_responses.push(PdfResponse {
            case_number: generated.document.case_number.as_str().to_string(),
            document_type: match generated.document.document_type {
                DocumentType::Rule16b => "rule16b",
                DocumentType::CourtOrder => "court_order",
                DocumentType::MinuteEntry => "minute_entry",
                DocumentType::WaiverIndictment => "waiver_indictment",
                DocumentType::ConditionsRelease => "conditions_release",
                DocumentType::CriminalJudgment => "criminal_judgment",
            }
            .to_string(),
            filename: generated.filename.clone(),
            pdf_base64: generated.to_base64(),
            size_bytes: generated.pdf_data.len(),
            document_id: generated.document.id.as_uuid().to_string(),
        });
    }

    let response = BatchPdfResponse {
        total_generated: pdf_responses.len(),
        documents: pdf_responses,
    };

    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response).unwrap_or_default())
        .build()
}

/// Store a judge's signature for later use in document signing
#[utoipa::path(
    post,
    path = "/api/signatures",
    request_body = StoreSignatureRequest,
    responses(
        (status = 200, description = "Signature stored successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "signature-management",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn store_signature(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);

    let body = req.body().to_vec();
    let request: StoreSignatureRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid request: {}"}}"#, e))
                .build();
        }
    };

    let judge_id = match uuid::Uuid::parse_str(&request.judge_id) {
        Ok(id) => id,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid judge ID: {}"}}"#, e))
                .build();
        }
    };

    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    match service.store_signature_sync(judge_id, &request.signature_base64) {
        Ok(_) => {
            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(r#"{"message": "Signature stored successfully"}"#.as_bytes().to_vec())
                .build()
        }
        Err(e) => {
            Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Failed to store signature: {}"}}"#, e))
                .build()
        }
    }
}

/// Retrieve a stored judge's signature
#[utoipa::path(
    get,
    path = "/api/signatures/{judge_id}",
    responses(
        (status = 200, description = "Signature retrieved successfully"),
        (status = 404, description = "Signature not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "signature-management",
    params(
        ("judge_id" = String, Path, description = "Judge ID"),
        ("X-Court-District" = String, Header, description = "Federal court district", example = "SDNY")
    ),
)]
pub fn get_signature(req: Request, params: Params) -> Response {
    let district_str = tenant::get_tenant_id(&req);
    let judge_id_str = params.get("judge_id").unwrap_or("");

    let judge_id = match uuid::Uuid::parse_str(judge_id_str) {
        Ok(id) => id,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Invalid judge ID: {}"}}"#, e))
                .build();
        }
    };

    let service = match create_pdf_service(&district_str) {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Service initialization failed: {}"}}"#, e))
                .build();
        }
    };

    match service.get_signature_sync(judge_id) {
        Ok(Some(signature)) => {
            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(serde_json::to_vec(&signature).unwrap_or_default())
                .build()
        }
        Ok(None) => {
            Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(r#"{"error": "Signature not found"}"#.as_bytes().to_vec())
                .build()
        }
        Err(e) => {
            Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(format!(r#"{{"error": "Failed to retrieve signature: {}"}}"#, e))
                .build()
        }
    }
}