//! Federal forms endpoints handler

use spin_sdk::http::{Request, Response, Params};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::utils::{federal_forms, tenant};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WaiverOfIndictmentRequest {
    pub defendant_name: String,
    pub charges: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ConditionsOfReleaseRequest {
    pub defendant_name: String,
    pub judge_name: String,
    pub conditions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CriminalJudgmentRequest {
    pub defendant_name: String,
    pub judge_name: String,
    pub plea: String,
    pub counts: String,
    pub sentence: String,
}

/// Generate Form AO 455: Waiver of an Indictment
#[utoipa::path(
    post,
    path = "/api/pdf/waiver-indictment/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case identifier")
    ),
    request_body = WaiverOfIndictmentRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation"
)]
pub fn generate_waiver_of_indictment(req: Request, params: Params) -> Response {
    let district = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or("24-cr-00789");

    let body = req.body().to_vec();
    let request: WaiverOfIndictmentRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!("Invalid request: {}", e))
                .build();
        }
    };

    let pdf_bytes = federal_forms::generate_waiver_of_indictment(
        case_id,
        &request.defendant_name,
        &district,
        &request.charges,
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"waiver-indictment-{}.pdf\"", case_id))
        .body(pdf_bytes)
        .build()
}

/// Generate Form AO 199A: Order Setting Conditions of Release
#[utoipa::path(
    post,
    path = "/api/pdf/conditions-release/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case identifier")
    ),
    request_body = ConditionsOfReleaseRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation"
)]
pub fn generate_conditions_of_release(req: Request, params: Params) -> Response {
    let district = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or("24-cr-00789");

    let body = req.body().to_vec();
    let request: ConditionsOfReleaseRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!("Invalid request: {}", e))
                .build();
        }
    };

    let conditions: Vec<&str> = request.conditions.iter().map(|s| s.as_str()).collect();

    let pdf_bytes = federal_forms::generate_conditions_of_release(
        case_id,
        &request.defendant_name,
        &district,
        &request.judge_name,
        conditions,
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"conditions-release-{}.pdf\"", case_id))
        .body(pdf_bytes)
        .build()
}

/// Generate Form AO 245B: Judgment in a Criminal Case
#[utoipa::path(
    post,
    path = "/api/pdf/criminal-judgment/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case identifier")
    ),
    request_body = CriminalJudgmentRequest,
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation"
)]
pub fn generate_criminal_judgment(req: Request, params: Params) -> Response {
    let district = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or("24-cr-00789");

    let body = req.body().to_vec();
    let request: CriminalJudgmentRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(format!("Invalid request: {}", e))
                .build();
        }
    };

    let pdf_bytes = federal_forms::generate_criminal_judgment(
        case_id,
        &request.defendant_name,
        &district,
        &request.judge_name,
        &request.plea,
        &request.counts,
        &request.sentence,
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"criminal-judgment-{}.pdf\"", case_id))
        .body(pdf_bytes)
        .build()
}

/// Auto-generate waiver with test data
#[utoipa::path(
    get,
    path = "/api/pdf/auto/waiver-indictment/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case identifier")
    ),
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation"
)]
pub fn auto_generate_waiver(req: Request, params: Params) -> Response {
    let district = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or("24-cr-00789");

    // Test data
    let (defendant, charges) = match case_id {
        "24-cr-00789" => ("John Michael Smith", "18 U.S.C. § 1343 (Wire Fraud) - Count 1\n18 U.S.C. § 1001 (False Statements) - Count 2"),
        "23-cr-00456" => ("Maria Rodriguez", "21 U.S.C. § 841(a)(1) (Possession with Intent to Distribute)"),
        _ => ("Test Defendant", "18 U.S.C. § 371 (Conspiracy)")
    };

    let pdf_bytes = federal_forms::generate_waiver_of_indictment(
        case_id,
        defendant,
        &district,
        charges,
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"waiver-indictment-{}.pdf\"", case_id))
        .body(pdf_bytes)
        .build()
}

/// Auto-generate conditions of release with test data
#[utoipa::path(
    get,
    path = "/api/pdf/auto/conditions-release/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case identifier")
    ),
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation"
)]
pub fn auto_generate_conditions(req: Request, params: Params) -> Response {
    let district = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or("24-cr-00789");

    let (defendant, judge) = match case_id {
        "24-cr-00789" => ("John Michael Smith", "Hon. Sarah Johnson"),
        "23-cr-00456" => ("Maria Rodriguez", "Hon. Michael Davis"),
        _ => ("Test Defendant", "Hon. District Judge")
    };

    let conditions = vec![
        "Report to Pretrial Services as directed",
        "Not commit any federal, state, or local crime",
        "Surrender all passports and not obtain new travel documents",
        "Travel restricted to SDNY and EDNY",
        "Maintain or seek employment",
        "Not possess firearms or destructive devices",
        "Submit to drug testing as directed",
        "Maintain residence as approved by Pretrial Services",
    ];

    let pdf_bytes = federal_forms::generate_conditions_of_release(
        case_id,
        defendant,
        &district,
        judge,
        conditions,
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"conditions-release-{}.pdf\"", case_id))
        .body(pdf_bytes)
        .build()
}

/// Auto-generate criminal judgment with test data
#[utoipa::path(
    get,
    path = "/api/pdf/auto/criminal-judgment/{case_id}",
    params(
        ("case_id" = String, Path, description = "Case identifier")
    ),
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 500, description = "Internal server error")
    ),
    tag = "pdf-generation"
)]
pub fn auto_generate_judgment(req: Request, params: Params) -> Response {
    let district = tenant::get_tenant_id(&req);
    let case_id = params.get("case_id").unwrap_or("24-cr-00789");

    let (defendant, judge, plea, counts, sentence) = match case_id {
        "24-cr-00789" => (
            "John Michael Smith",
            "Hon. Sarah Johnson",
            "Guilty",
            "Count 1: 18 U.S.C. § 1343 (Wire Fraud)",
            "60 months imprisonment"
        ),
        "23-cr-00456" => (
            "Maria Rodriguez",
            "Hon. Michael Davis",
            "Guilty",
            "Count 1: 21 U.S.C. § 841(a)(1)",
            "36 months imprisonment"
        ),
        _ => (
            "Test Defendant",
            "Hon. District Judge",
            "No Contest",
            "Count 1: 18 U.S.C. § 371",
            "24 months imprisonment"
        )
    };

    let pdf_bytes = federal_forms::generate_criminal_judgment(
        case_id,
        defendant,
        &district,
        judge,
        plea,
        counts,
        sentence,
    );

    Response::builder()
        .status(200)
        .header("content-type", "application/pdf")
        .header("content-disposition", format!("attachment; filename=\"criminal-judgment-{}.pdf\"", case_id))
        .body(pdf_bytes)
        .build()
}