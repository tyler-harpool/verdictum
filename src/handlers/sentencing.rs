//! HTTP handlers for federal sentencing management

use crate::domain::sentencing::*;
use crate::ports::sentencing_repository::SentencingRepository;
use crate::utils::repository_factory::RepositoryFactory;
use spin_sdk::http::{Params, Request, Response};

/// Create a new sentencing record
#[utoipa::path(
    post,
    path = "/api/sentencing",
    request_body = CreateSentencingRequest,
    responses(
        (status = 201, description = "Sentencing created", body = Sentencing),
        (status = 400, description = "Invalid input")
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_sentencing(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let body = req.body().to_vec();

    let request: CreateSentencingRequest = match serde_json::from_slice(&body) {
        Ok(s) => s,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    // Use the constructor to create a properly initialized sentencing
    let sentencing = Sentencing::new(
        request.case_id,
        request.defendant_id,
        request.judge_id
    );

    match repo.create_sentencing(sentencing) {
        Ok(created) => Response::builder()
            .status(201)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&created).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error creating sentencing: {}", e))
            .build()
    }
}

/// Get sentencing by ID
#[utoipa::path(
    get,
    path = "/api/sentencing/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    responses(
        (status = 200, description = "Sentencing found", body = Sentencing),
        (status = 404, description = "Not found")
    ),
    tag = "Sentencing",
)]
pub fn get_sentencing(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();

    match repo.get_sentencing(&id) {
        Ok(Some(sentencing)) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencing).unwrap())
            .build(),
        Ok(None) => Response::builder()
            .status(404)
            .body("Sentencing not found")
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Update sentencing
#[utoipa::path(
    put,
    path = "/api/sentencing/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    request_body = Sentencing,
    responses(
        (status = 200, description = "Updated", body = Sentencing),
        (status = 404, description = "Not found")
    ),
    tag = "Sentencing",
)]
pub fn update_sentencing(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let body = req.body().to_vec();

    let mut sentencing: Sentencing = match serde_json::from_slice(&body) {
        Ok(s) => s,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    let id = params.get("id").unwrap_or("").to_string();
    sentencing.id = id;

    match repo.update_sentencing(sentencing) {
        Ok(updated) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&updated).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Delete sentencing
#[utoipa::path(
    delete,
    path = "/api/sentencing/{id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found")
    ),
    tag = "Sentencing",
)]
pub fn delete_sentencing(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();

    match repo.delete_sentencing(&id) {
        Ok(()) => Response::builder()
            .status(204)
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find sentencings by case
#[utoipa::path(
    get,
    path = "/api/sentencing/case/{case_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = String, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "Sentencings found", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
)]
pub fn find_by_case(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let case_id = params.get("case_id").unwrap_or("").to_string();

    match repo.find_by_case(&case_id) {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find sentencings by defendant
#[utoipa::path(
    get,
    path = "/api/sentencing/defendant/{defendant_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("defendant_id" = String, Path, description = "Defendant ID")
    ),
    responses(
        (status = 200, description = "Sentencings found", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
)]
pub fn find_by_defendant(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let defendant_id = params.get("defendant_id").unwrap_or("").to_string();

    match repo.find_by_defendant(&defendant_id) {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find sentencings by judge
#[utoipa::path(
    get,
    path = "/api/sentencing/judge/{judge_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = String, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "Sentencings found", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
)]
pub fn find_by_judge(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let judge_id = params.get("judge_id").unwrap_or("").to_string();

    match repo.find_by_judge(&judge_id) {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find pending sentencings
#[utoipa::path(
    get,
    path = "/api/sentencing/pending",
    responses(
        (status = 200, description = "Pending sentencings", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn find_pending(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.find_pending_sentencing() {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Calculate sentencing guidelines
#[utoipa::path(
    post,
    path = "/api/sentencing/calculate-guidelines",
    request_body = GuidelinesCalculation,
    responses(
        (status = 200, description = "Guidelines calculated", body = GuidelinesRange)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn calculate_guidelines(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let body = req.body().to_vec();

    let calculation: GuidelinesCalculation = match serde_json::from_slice(&body) {
        Ok(c) => c,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    match repo.calculate_guidelines(calculation) {
        Ok(range) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&range).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get departure statistics
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/departures",
    responses(
        (status = 200, description = "Departure statistics", body = SentencingStatistics)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_departure_stats(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.get_departure_rates() {
        Ok(stats) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&stats).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get variance statistics
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/variances",
    responses(
        (status = 200, description = "Variance statistics", body = SentencingStatistics)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_variance_stats(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.get_variance_rates() {
        Ok(stats) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&stats).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Add departure
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/departure",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    request_body = Departure,
    responses(
        (status = 200, description = "Departure added", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn add_departure(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();
    let body = req.body().to_vec();

    let departure: Departure = match serde_json::from_slice(&body) {
        Ok(d) => d,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    match repo.add_departure(&id, departure) {
        Ok(sentencing) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencing).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Add variance
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/variance",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    request_body = Variance,
    responses(
        (status = 200, description = "Variance added", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn add_variance(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();
    let body = req.body().to_vec();

    let variance: Variance = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    match repo.add_variance(&id, variance) {
        Ok(sentencing) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencing).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get substantial assistance cases
#[utoipa::path(
    get,
    path = "/api/sentencing/substantial-assistance",
    responses(
        (status = 200, description = "Substantial assistance cases", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_substantial_assistance(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.get_substantial_assistance_cases() {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Add special condition
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/special-condition",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    request_body = SpecialCondition,
    responses(
        (status = 200, description = "Special condition added", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn add_special_condition(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();
    let body = req.body().to_vec();

    let condition: SpecialCondition = match serde_json::from_slice(&body) {
        Ok(c) => c,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    match repo.add_special_condition(&id, condition) {
        Ok(sentencing) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencing).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Update supervised release
#[utoipa::path(
    put,
    path = "/api/sentencing/{id}/supervised-release",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    request_body = SupervisedRelease,
    responses(
        (status = 200, description = "Supervised release updated", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn update_supervised_release(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();
    let body = req.body().to_vec();

    let release: SupervisedRelease = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    match repo.update_supervised_release(&id, release) {
        Ok(sentencing) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencing).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find active supervision
#[utoipa::path(
    get,
    path = "/api/sentencing/active-supervision",
    responses(
        (status = 200, description = "Active supervision cases", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn find_active_supervision(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.find_active_supervision() {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Add BOP designation
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/bop-designation",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    request_body = BOPDesignation,
    responses(
        (status = 200, description = "BOP designation added", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn add_bop_designation(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();
    let body = req.body().to_vec();

    let designation: BOPDesignation = match serde_json::from_slice(&body) {
        Ok(d) => d,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    match repo.add_bop_designation(&id, designation) {
        Ok(sentencing) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencing).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get RDAP eligible
#[utoipa::path(
    get,
    path = "/api/sentencing/rdap-eligible",
    responses(
        (status = 200, description = "RDAP eligible cases", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_rdap_eligible(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.get_rdap_eligible() {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get judge sentencing statistics
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/judge/{judge_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = String, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "Judge sentencing statistics", body = SentencingStatistics)
    ),
    tag = "Sentencing",
)]
pub fn get_judge_stats(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let judge_id = params.get("judge_id").unwrap_or("").to_string();

    match repo.get_judge_sentencing_stats(&judge_id) {
        Ok(stats) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&stats).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get district statistics
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/district",
    responses(
        (status = 200, description = "District sentencing statistics", body = SentencingStatistics)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_district_stats(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.get_district_stats() {
        Ok(stats) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&stats).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get trial penalty analysis
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/trial-penalty",
    responses(
        (status = 200, description = "Trial penalty analysis", body = SentencingStatistics)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_trial_penalty(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.get_trial_penalty_analysis() {
        Ok(stats) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&stats).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Add prior sentence to criminal history
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/prior-sentence",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    request_body = PriorSentence,
    responses(
        (status = 200, description = "Prior sentence added", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn add_prior_sentence(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();
    let body = req.body().to_vec();

    let prior: PriorSentence = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => return Response::builder()
            .status(400)
            .body(format!("Invalid JSON: {}", e))
            .build()
    };

    match repo.add_prior_sentence(&id, prior) {
        Ok(sentencing) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencing).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find upcoming sentencings
#[utoipa::path(
    get,
    path = "/api/sentencing/upcoming/{days}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("days" = i32, Path, description = "Days ahead to search")
    ),
    responses(
        (status = 200, description = "Upcoming sentencings", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
)]
pub fn find_upcoming(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let days_str = params.get("days").unwrap_or("30");
    let days: i32 = days_str.parse().unwrap_or(30);

    match repo.find_upcoming_sentencings(days) {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find appeal deadlines approaching
#[utoipa::path(
    get,
    path = "/api/sentencing/appeal-deadlines",
    responses(
        (status = 200, description = "Appeal deadlines approaching", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn find_appeal_deadlines(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    match repo.find_appeal_deadline_approaching() {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Find sentencings by date range
#[utoipa::path(
    get,
    path = "/api/sentencing/date-range",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("start" = String, Query, description = "Start date (YYYY-MM-DD)"),
        ("end" = String, Query, description = "End date (YYYY-MM-DD)")
    ),
    responses(
        (status = 200, description = "Sentencings in date range", body = Vec<Sentencing>)
    ),
    tag = "Sentencing",
)]
pub fn find_by_date_range(req: Request, _params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    // Parse query parameters
    let query = req.query();
    let parsed_params = crate::utils::query_parser::parse_query_string(query);

    let start = crate::utils::query_parser::get_string(&parsed_params, "start").unwrap_or_default();
    let end = crate::utils::query_parser::get_string(&parsed_params, "end").unwrap_or_default();

    if start.is_empty() || end.is_empty() {
        return Response::builder()
            .status(400)
            .body("Missing start or end date parameters")
            .build();
    }

    match repo.find_by_date_range(&start, &end) {
        Ok(sentencings) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&sentencings).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Get offense type statistics
#[utoipa::path(
    get,
    path = "/api/sentencing/statistics/offense/{offense_type}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("offense_type" = String, Path, description = "Offense type (e.g., '2B1' for fraud)")
    ),
    responses(
        (status = 200, description = "Offense type statistics", body = SentencingStatistics)
    ),
    tag = "Sentencing",
)]
pub fn get_offense_type_stats(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let offense_type = params.get("offense_type").unwrap_or("").to_string();

    match repo.get_offense_type_stats(&offense_type) {
        Ok(stats) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&stats).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Calculate criminal history points
#[utoipa::path(
    get,
    path = "/api/sentencing/{id}/criminal-history-points",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    responses(
        (status = 200, description = "Criminal history points", body = i32)
    ),
    tag = "Sentencing",
)]
pub fn calculate_criminal_history_points(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();

    match repo.calculate_criminal_history_points(&id) {
        Ok(points) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&points).unwrap())
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}
/// Calculate final offense level for sentencing
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/calculate-offense-level",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    responses(
        (status = 200, description = "Final offense level calculated", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn calculate_offense_level(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();

    match repo.get_sentencing(&id) {
        Ok(Some(mut sentencing)) => {
            sentencing.calculate_final_offense_level();
            match repo.update_sentencing(sentencing) {
                Ok(updated) => Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&updated).unwrap())
                    .build(),
                Err(e) => Response::builder()
                    .status(500)
                    .body(format!("Error updating sentencing: {}", e))
                    .build()
            }
        },
        Ok(None) => Response::builder()
            .status(404)
            .body("Sentencing not found")
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Lookup guidelines range for sentencing
#[utoipa::path(
    post,
    path = "/api/sentencing/{id}/lookup-guidelines-range",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    responses(
        (status = 200, description = "Guidelines range calculated", body = Sentencing)
    ),
    tag = "Sentencing",
)]
pub fn lookup_guidelines_range(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();

    match repo.get_sentencing(&id) {
        Ok(Some(mut sentencing)) => {
            sentencing.lookup_guidelines_range();
            match repo.update_sentencing(sentencing) {
                Ok(updated) => Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&updated).unwrap())
                    .build(),
                Err(e) => Response::builder()
                    .status(500)
                    .body(format!("Error updating sentencing: {}", e))
                    .build()
            }
        },
        Ok(None) => Response::builder()
            .status(404)
            .body("Sentencing not found")
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}

/// Check if defendant is eligible for safety valve
#[utoipa::path(
    get,
    path = "/api/sentencing/{id}/safety-valve-eligible",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Sentencing ID")
    ),
    responses(
        (status = 200, description = "Safety valve eligibility", body = bool)
    ),
    tag = "Sentencing",
)]
pub fn check_safety_valve_eligible(req: Request, params: Params) -> Response {
    let repo = RepositoryFactory::sentencing_repo(&req);

    let id = params.get("id").unwrap_or("").to_string();

    match repo.get_sentencing(&id) {
        Ok(Some(sentencing)) => {
            let eligible = sentencing.is_safety_valve_eligible();
            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&eligible).unwrap())
                .build()
        },
        Ok(None) => Response::builder()
            .status(404)
            .body("Sentencing not found")
            .build(),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Error: {}", e))
            .build()
    }
}
