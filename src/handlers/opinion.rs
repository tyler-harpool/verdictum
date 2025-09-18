//! REST API handlers for judicial opinions
//!
//! This module provides HTTP endpoints for managing judicial opinions,
//! drafts, citations, and publication workflows.

use crate::domain::opinion::{
    JudicialOpinion, OpinionDraft, OpinionType, OpinionStatus, Disposition,
    Citation, JudgeVote, VoteType, LegalCitation, CitationTreatment, Headnote,
    DraftComment, OpinionStatistics as DomainOpinionStatistics
};
use crate::error::{ApiError, ApiResult};
use crate::ports::document_repository::{
    DocumentRepository, OpinionFilter, OpinionStatistics, CitationStatistics
};
use crate::utils::repository_factory::RepositoryFactory;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Params, Request, ResponseBuilder};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Helper macro to get tenant-specific repository
macro_rules! get_tenant_repo {
    ($req:expr) => {{
        RepositoryFactory::document_repo($req)
    }}
}

/// Request to create a new opinion
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOpinionRequest {
    pub case_id: String,
    pub case_name: String,
    pub docket_number: String,
    pub author_judge_id: String,
    pub author_judge_name: String,
    pub opinion_type: OpinionType,
    pub title: String,
    pub syllabus: String,
    pub content: String,
}

/// Request to update an opinion
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOpinionRequest {
    pub title: Option<String>,
    pub syllabus: Option<String>,
    pub content: Option<String>,
    pub disposition: Option<Disposition>,
    pub status: Option<OpinionStatus>,
    pub keywords: Option<Vec<String>>,
}

/// Request to publish an opinion
#[derive(Debug, Deserialize, ToSchema)]
pub struct PublishOpinionRequest {
    pub is_precedential: bool,
    pub citation: Citation,
}

/// Request to add a judge vote
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddVoteRequest {
    pub judge_id: String,
    pub judge_name: String,
    pub vote: VoteType,
    pub opinion_id: Option<String>,
    pub notes: String,
}

/// Request to add a legal citation
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddCitationRequest {
    pub citation_text: String,
    pub case_name: String,
    pub reporter: String,
    pub year: Option<i32>,
    pub court: Option<String>,
    pub proposition: String,
    pub treatment: CitationTreatment,
}

/// Request to add a headnote
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddHeadnoteRequest {
    pub topic: String,
    pub subtopic: Option<String>,
    pub text: String,
    pub key_number: Option<String>,
    pub cited_paragraphs: Vec<String>,
}

/// Request to create a draft
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDraftRequest {
    pub content: String,
    pub changes_summary: String,
    pub created_by: String,
}

/// Request to add a comment to draft
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddCommentRequest {
    pub judge_id: String,
    pub judge_name: String,
    pub paragraph_ref: Option<String>,
    pub comment_text: String,
}

/// Response for opinion lists
#[derive(Debug, Serialize, ToSchema)]
pub struct OpinionListResponse {
    pub opinions: Vec<JudicialOpinion>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

/// Create a new opinion
#[utoipa::path(
    post,
    path = "/api/opinions",
    request_body = CreateOpinionRequest,
    responses(
        (status = 201, description = "Opinion created successfully", body = JudicialOpinion),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn create_opinion(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateOpinionRequest = serde_json::from_slice(body)?;
    
    let mut opinion = JudicialOpinion::new(
        request.case_id,
        request.case_name,
        request.docket_number,
        request.author_judge_id,
        request.author_judge_name,
        request.opinion_type,
        request.title,
    );
    
    opinion.syllabus = request.syllabus;
    opinion.content = request.content;
    
    let repo = get_tenant_repo!(&req);
    let created = repo.create_opinion(opinion)?;
    
    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&created)?)
        .build())
}

/// Get an opinion by ID
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 200, description = "Opinion found", body = JudicialOpinion),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn get_opinion(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let opinion = repo.get_opinion(opinion_id)?
        .ok_or_else(|| ApiError::NotFound("Opinion not found".to_string()))?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&opinion)?)
        .build())
}

/// Update an opinion
#[utoipa::path(
    patch,
    path = "/api/opinions/{opinion_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    request_body = UpdateOpinionRequest,
    responses(
        (status = 200, description = "Opinion updated successfully", body = JudicialOpinion),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn update_opinion(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let body = req.body();
    let request: UpdateOpinionRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut opinion = repo.get_opinion(opinion_id)?
        .ok_or_else(|| ApiError::NotFound("Opinion not found".to_string()))?;
    
    if let Some(title) = request.title {
        opinion.title = title;
    }
    if let Some(syllabus) = request.syllabus {
        opinion.syllabus = syllabus;
    }
    if let Some(content) = request.content {
        opinion.content = content;
    }
    if let Some(disposition) = request.disposition {
        opinion.disposition = disposition;
    }
    if let Some(status) = request.status {
        opinion.status = status;
    }
    if let Some(keywords) = request.keywords {
        opinion.keywords = keywords;
    }
    
    opinion.updated_at = Utc::now();
    let updated = repo.update_opinion(opinion)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Delete an opinion
#[utoipa::path(
    delete,
    path = "/api/opinions/{opinion_id}",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 204, description = "Opinion deleted successfully"),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn delete_opinion(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    repo.delete_opinion(opinion_id)?;
    
    Ok(ResponseBuilder::new(204).build())
}

/// List opinions with filters
#[utoipa::path(
    get,
    path = "/api/opinions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = Option<String>, Query, description = "Filter by case ID"),
        ("author_judge_id" = Option<String>, Query, description = "Filter by author judge"),
        ("is_published" = Option<bool>, Query, description = "Filter by publication status"),
        ("is_precedential" = Option<bool>, Query, description = "Filter by precedential status"),
        ("keywords" = Option<String>, Query, description = "Search keywords (comma-separated)"),
        ("limit" = Option<usize>, Query, description = "Maximum number of results"),
        ("offset" = Option<usize>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List of opinions", body = OpinionListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn list_opinions(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let keywords = query_map.get("keywords")
        .map(|s| s.split(',').map(|k| k.trim().to_string()).collect());
    
    let filter = OpinionFilter {
        case_id: query_map.get("case_id").map(|s| s.to_string()),
        author_judge_id: query_map.get("author_judge_id").map(|s| s.to_string()),
        is_published: query_map.get("is_published").and_then(|s| s.parse().ok()),
        is_precedential: query_map.get("is_precedential").and_then(|s| s.parse().ok()),
        keywords,
        limit: query_map.get("limit").and_then(|s| s.parse().ok()),
        offset: query_map.get("offset").and_then(|s| s.parse().ok()),
        ..Default::default()
    };
    
    let limit = filter.limit.unwrap_or(100);
    let offset = filter.offset.unwrap_or(0);
    
    let repo = get_tenant_repo!(&req);
    let opinions = repo.list_opinions(filter)?;
    
    let response = OpinionListResponse {
        total: opinions.len(),
        opinions,
        offset,
        limit,
    };
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// File an opinion
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/file",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 200, description = "Opinion filed successfully", body = JudicialOpinion),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn file_opinion(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let mut opinion = repo.get_opinion(opinion_id)?
        .ok_or_else(|| ApiError::NotFound("Opinion not found".to_string()))?;
    
    opinion.file();
    let updated = repo.update_opinion(opinion)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Publish an opinion
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/publish",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    request_body = PublishOpinionRequest,
    responses(
        (status = 200, description = "Opinion published successfully", body = JudicialOpinion),
        (status = 404, description = "Opinion not found"),
        (status = 400, description = "Opinion not filed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn publish_opinion(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let body = req.body();
    let request: PublishOpinionRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut opinion = repo.get_opinion(opinion_id)?
        .ok_or_else(|| ApiError::NotFound("Opinion not found".to_string()))?;
    
    if !matches!(opinion.status, OpinionStatus::Filed) {
        return Err(ApiError::BadRequest("Opinion must be filed before publishing".to_string()));
    }
    
    opinion.is_precedential = request.is_precedential;
    opinion.publish(request.citation);
    let updated = repo.update_opinion(opinion)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Add a judge vote to an opinion
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/votes",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    request_body = AddVoteRequest,
    responses(
        (status = 200, description = "Vote added successfully", body = JudicialOpinion),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn add_judge_vote(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let body = req.body();
    let request: AddVoteRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut opinion = repo.get_opinion(opinion_id)?
        .ok_or_else(|| ApiError::NotFound("Opinion not found".to_string()))?;
    
    let vote = JudgeVote {
        judge_id: request.judge_id,
        judge_name: request.judge_name,
        vote: request.vote,
        opinion_id: request.opinion_id,
        notes: request.notes,
    };
    
    opinion.add_joining_judge(vote);
    let updated = repo.update_opinion(opinion)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Add a legal citation to an opinion
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/citations",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    request_body = AddCitationRequest,
    responses(
        (status = 200, description = "Citation added successfully", body = JudicialOpinion),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn add_citation(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let body = req.body();
    let request: AddCitationRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut opinion = repo.get_opinion(opinion_id)?
        .ok_or_else(|| ApiError::NotFound("Opinion not found".to_string()))?;
    
    let citation = LegalCitation {
        citation_text: request.citation_text,
        case_name: request.case_name,
        reporter: request.reporter,
        year: request.year,
        court: request.court,
        page: None,
        pin_cite: None,
        proposition: request.proposition,
        treatment: request.treatment,
    };
    
    opinion.add_citation(citation);
    let updated = repo.update_opinion(opinion)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Add a headnote to an opinion
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/headnotes",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    request_body = AddHeadnoteRequest,
    responses(
        (status = 200, description = "Headnote added successfully", body = JudicialOpinion),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn add_headnote(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let body = req.body();
    let request: AddHeadnoteRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut opinion = repo.get_opinion(opinion_id)?
        .ok_or_else(|| ApiError::NotFound("Opinion not found".to_string()))?;
    
    let headnote = Headnote {
        number: opinion.headnotes.len() as i32 + 1,
        topic: request.topic,
        subtopic: request.subtopic,
        text: request.text,
        key_number: request.key_number,
        cited_paragraphs: request.cited_paragraphs,
    };
    
    opinion.add_headnote(headnote);
    let updated = repo.update_opinion(opinion)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Get opinions by case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/opinions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("case_id" = String, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of opinions for the case", body = Vec<JudicialOpinion>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn get_opinions_by_case(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params.get("case_id")
        .ok_or_else(|| ApiError::BadRequest("Case ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let opinions = repo.find_opinions_by_case(case_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&opinions)?)
        .build())
}

/// Get opinions by author
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/opinions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = String, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "List of opinions by the judge", body = Vec<JudicialOpinion>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn get_opinions_by_author(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params.get("judge_id")
        .ok_or_else(|| ApiError::BadRequest("Judge ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let opinions = repo.find_opinions_by_author(judge_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&opinions)?)
        .build())
}

/// Search opinions
#[utoipa::path(
    get,
    path = "/api/opinions/search",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("q" = String, Query, description = "Search query")
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<JudicialOpinion>),
        (status = 400, description = "Search query required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn search_opinions(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let search_query = query_map.get("q")
        .ok_or_else(|| ApiError::BadRequest("Search query required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let opinions = repo.search_opinions(search_query)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&opinions)?)
        .build())
}

/// Get precedential opinions
#[utoipa::path(
    get,
    path = "/api/opinions/precedential",
    responses(
        (status = 200, description = "List of precedential opinions", body = Vec<JudicialOpinion>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY")
    ),
)]
pub fn get_precedential_opinions(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = get_tenant_repo!(&req);
    let opinions = repo.find_precedential_opinions()?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&opinions)?)
        .build())
}

/// Create a draft for an opinion
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/drafts",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    request_body = CreateDraftRequest,
    responses(
        (status = 201, description = "Draft created successfully", body = OpinionDraft),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Opinion Drafts",
)]
pub fn create_draft(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let body = req.body();
    let request: CreateDraftRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    
    // Get current draft to determine version number
    let drafts = repo.list_drafts(opinion_id)?;
    let version = drafts.first().map(|d| d.version + 1).unwrap_or(1);
    
    let draft = OpinionDraft::new(
        opinion_id.to_string(),
        version,
        request.content,
        request.changes_summary,
        request.created_by,
    );
    
    let created = repo.create_draft(draft)?;
    
    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&created)?)
        .build())
}

/// Get drafts for an opinion
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}/drafts",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 200, description = "List of drafts", body = Vec<OpinionDraft>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Opinion Drafts",
)]
pub fn get_drafts(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id")
        .ok_or_else(|| ApiError::BadRequest("Opinion ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let drafts = repo.list_drafts(opinion_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&drafts)?)
        .build())
}

/// Get opinion statistics
#[utoipa::path(
    get,
    path = "/api/opinions/statistics",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("judge_id" = Option<String>, Query, description = "Filter by judge ID")
    ),
    responses(
        (status = 200, description = "Opinion statistics", body = OpinionStatistics),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn get_opinion_statistics(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let judge_id = query_map.get("judge_id").map(|s| *s);
    
    let repo = get_tenant_repo!(&req);
    let stats = repo.get_opinion_statistics(judge_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&stats)?)
        .build())
}

/// Get citation statistics
#[utoipa::path(
    get,
    path = "/api/opinions/citations/statistics",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("start_date" = Option<String>, Query, description = "Start date (ISO 8601)"),
        ("end_date" = Option<String>, Query, description = "End date (ISO 8601)")
    ),
    responses(
        (status = 200, description = "Citation statistics", body = CitationStatistics),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Opinions",
)]
pub fn get_citation_statistics(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let start_date = query_map.get("start_date")
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(365));
    
    let end_date = query_map.get("end_date")
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now());
    
    let repo = get_tenant_repo!(&req);
    let stats = repo.get_citation_statistics(start_date, end_date)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&stats)?)
        .build())
}

/// Get the current draft for an opinion
#[utoipa::path(
    get,
    path = "/api/opinions/{opinion_id}/drafts/current",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 200, description = "Current draft", body = OpinionDraft),
        (status = 404, description = "No current draft found")
    ),
    tag = "Opinion Drafts",
)]
pub fn get_current_draft(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);
    let draft = repo.get_current_draft(&opinion_id)?
        .ok_or_else(|| ApiError::NotFound("No current draft found".to_string()))?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&draft)?)
        .build())
}

/// Check if opinion is majority opinion
#[utoipa::path(
    get,
    path = "/api/opinions/{id}/is-majority",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 200, description = "Success", body = bool),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Opinions",
)]
pub fn is_majority_opinion(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params.get("id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);
    let opinion = repo.get_opinion(&id)?
        .ok_or_else(|| ApiError::NotFound(format!("Opinion {} not found", id)))?;

    let is_majority = opinion.is_majority();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&is_majority)?)
        .build())
}

/// Check if opinion creates binding precedent
#[utoipa::path(
    get,
    path = "/api/opinions/{id}/is-binding",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 200, description = "Success", body = bool),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Opinions",
)]
pub fn is_binding_opinion(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params.get("id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);
    let opinion = repo.get_opinion(&id)?
        .ok_or_else(|| ApiError::NotFound(format!("Opinion {} not found", id)))?;

    let is_binding = opinion.is_binding();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&is_binding)?)
        .build())
}

/// Calculate detailed statistics for an opinion
#[utoipa::path(
    get,
    path = "/api/opinions/{id}/calculate-statistics",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("id" = String, Path, description = "Opinion ID")
    ),
    responses(
        (status = 200, description = "Success", body = DomainOpinionStatistics),
        (status = 404, description = "Opinion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Opinions",
)]
pub fn calculate_opinion_statistics(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params.get("id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);
    let opinion = repo.get_opinion(&id)?
        .ok_or_else(|| ApiError::NotFound(format!("Opinion {} not found", id)))?;

    let statistics = opinion.calculate_statistics();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&statistics)?)
        .build())
}

/// Add comment to opinion draft
#[utoipa::path(
    post,
    path = "/api/opinions/{opinion_id}/drafts/{draft_id}/comments",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID"),
        ("draft_id" = String, Path, description = "Draft ID")
    ),
    request_body = AddCommentRequest,
    responses(
        (status = 200, description = "Comment added successfully"),
        (status = 404, description = "Draft not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Opinion Drafts",
)]
pub fn add_draft_comment(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id").unwrap_or("").to_string();
    let draft_id = params.get("draft_id").unwrap_or("").to_string();

    let body = req.body();
    let comment_request: AddCommentRequest = serde_json::from_slice(body)
        .map_err(|e| ApiError::SerializationError(e.to_string()))?;

    let repo = get_tenant_repo!(&req);

    // Get the draft
    let mut draft = repo.get_draft(&draft_id)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {} not found", draft_id)))?;

    // Verify it belongs to the correct opinion
    if draft.opinion_id != opinion_id {
        return Err(ApiError::BadRequest("Draft does not belong to this opinion".to_string()));
    }

    // Create and add the comment
    let comment = DraftComment {
        id: uuid::Uuid::new_v4().to_string(),
        judge_id: comment_request.judge_id,
        judge_name: comment_request.judge_name,
        paragraph_ref: comment_request.paragraph_ref,
        comment_text: comment_request.comment_text,
        created_at: Utc::now(),
        resolved: false,
        resolved_at: None,
    };

    draft.add_comment(comment);

    // Save the updated draft back
    let updated_draft = repo.update_draft(draft)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated_draft)?)
        .build())
}

/// Resolve comment on opinion draft
#[utoipa::path(
    patch,
    path = "/api/opinions/{opinion_id}/drafts/{draft_id}/comments/{comment_id}/resolve",
    params(
        ("X-Court-District" = String, Header, description = "Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)", example = "SDNY"),
        ("opinion_id" = String, Path, description = "Opinion ID"),
        ("draft_id" = String, Path, description = "Draft ID"),
        ("comment_id" = String, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment resolved successfully"),
        (status = 404, description = "Draft or comment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Opinion Drafts",
)]
pub fn resolve_draft_comment(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let opinion_id = params.get("opinion_id").unwrap_or("").to_string();
    let draft_id = params.get("draft_id").unwrap_or("").to_string();
    let comment_id = params.get("comment_id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);

    // Get the draft
    let mut draft = repo.get_draft(&draft_id)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {} not found", draft_id)))?;

    // Verify it belongs to the correct opinion
    if draft.opinion_id != opinion_id {
        return Err(ApiError::BadRequest("Draft does not belong to this opinion".to_string()));
    }

    // Resolve the comment
    draft.resolve_comment(&comment_id);

    // Save the updated draft back
    let updated_draft = repo.update_draft(draft)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated_draft)?)
        .build())
}
