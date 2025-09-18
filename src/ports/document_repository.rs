//! Repository trait for judicial documents (orders and opinions)
//!
//! This module defines the repository interface for managing
//! judicial orders and opinions with their associated metadata.

use crate::domain::order::{JudicialOrder, OrderTemplate, OrderType, OrderStatus};
use crate::domain::opinion::{JudicialOpinion, OpinionDraft};
use crate::error::ApiResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Repository trait for managing judicial documents
pub trait DocumentRepository: Send + Sync {
    // Order operations
    fn create_order(&self, order: JudicialOrder) -> ApiResult<JudicialOrder>;
    fn get_order(&self, order_id: &str) -> ApiResult<Option<JudicialOrder>>;
    fn update_order(&self, order: JudicialOrder) -> ApiResult<JudicialOrder>;
    fn delete_order(&self, order_id: &str) -> ApiResult<()>;
    fn list_orders(&self, filter: OrderFilter) -> ApiResult<Vec<JudicialOrder>>;
    fn find_orders_by_case(&self, case_id: &str) -> ApiResult<Vec<JudicialOrder>>;
    fn find_orders_by_judge(&self, judge_id: &str) -> ApiResult<Vec<JudicialOrder>>;
    fn find_pending_signatures(&self, judge_id: &str) -> ApiResult<Vec<JudicialOrder>>;
    fn find_expiring_orders(&self, days: i64) -> ApiResult<Vec<JudicialOrder>>;
    
    // Order template operations
    fn create_template(&self, template: OrderTemplate) -> ApiResult<OrderTemplate>;
    fn get_template(&self, template_id: &str) -> ApiResult<Option<OrderTemplate>>;
    fn update_template(&self, template: OrderTemplate) -> ApiResult<OrderTemplate>;
    fn delete_template(&self, template_id: &str) -> ApiResult<()>;
    fn list_templates(&self, order_type: Option<OrderType>) -> ApiResult<Vec<OrderTemplate>>;
    fn find_active_templates(&self) -> ApiResult<Vec<OrderTemplate>>;

    // Opinion operations
    fn create_opinion(&self, opinion: JudicialOpinion) -> ApiResult<JudicialOpinion>;
    fn get_opinion(&self, opinion_id: &str) -> ApiResult<Option<JudicialOpinion>>;
    fn update_opinion(&self, opinion: JudicialOpinion) -> ApiResult<JudicialOpinion>;
    fn delete_opinion(&self, opinion_id: &str) -> ApiResult<()>;
    fn list_opinions(&self, filter: OpinionFilter) -> ApiResult<Vec<JudicialOpinion>>;
    fn find_opinions_by_case(&self, case_id: &str) -> ApiResult<Vec<JudicialOpinion>>;
    fn find_opinions_by_author(&self, judge_id: &str) -> ApiResult<Vec<JudicialOpinion>>;
    fn find_published_opinions(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> ApiResult<Vec<JudicialOpinion>>;
    fn search_opinions(&self, query: &str) -> ApiResult<Vec<JudicialOpinion>>;
    fn find_precedential_opinions(&self) -> ApiResult<Vec<JudicialOpinion>>;

    // Draft operations
    fn create_draft(&self, draft: OpinionDraft) -> ApiResult<OpinionDraft>;
    fn get_draft(&self, draft_id: &str) -> ApiResult<Option<OpinionDraft>>;
    fn update_draft(&self, draft: OpinionDraft) -> ApiResult<OpinionDraft>;
    fn list_drafts(&self, opinion_id: &str) -> ApiResult<Vec<OpinionDraft>>;
    fn get_current_draft(&self, opinion_id: &str) -> ApiResult<Option<OpinionDraft>>;

    // Statistics
    fn get_order_statistics(&self, judge_id: Option<&str>) -> ApiResult<OrderStatistics>;
    fn get_opinion_statistics(&self, judge_id: Option<&str>) -> ApiResult<OpinionStatistics>;
    fn get_citation_statistics(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> ApiResult<CitationStatistics>;
}

/// Filter for querying orders
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderFilter {
    pub case_id: Option<String>,
    pub judge_id: Option<String>,
    pub order_type: Option<OrderType>,
    pub status: Option<OrderStatus>,
    pub is_sealed: Option<bool>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub requires_service: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Filter for querying opinions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpinionFilter {
    pub case_id: Option<String>,
    pub author_judge_id: Option<String>,
    pub opinion_type: Option<crate::domain::opinion::OpinionType>,
    pub status: Option<crate::domain::opinion::OpinionStatus>,
    pub disposition: Option<crate::domain::opinion::Disposition>,
    pub is_published: Option<bool>,
    pub is_precedential: Option<bool>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub keywords: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Statistics for orders
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderStatistics {
    pub total_orders: usize,
    pub orders_by_type: Vec<(String, usize)>,
    pub orders_by_status: Vec<(String, usize)>,
    pub pending_signatures: usize,
    pub sealed_orders: usize,
    pub orders_requiring_service: usize,
    pub average_time_to_signature: f64,
    pub average_time_to_service: f64,
    pub orders_this_month: usize,
    pub orders_last_month: usize,
}

/// Statistics for opinions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpinionStatistics {
    pub total_opinions: usize,
    pub opinions_by_type: Vec<(String, usize)>,
    pub opinions_by_disposition: Vec<(String, usize)>,
    pub published_opinions: usize,
    pub precedential_opinions: usize,
    pub average_opinion_length: f64,
    pub average_citations_per_opinion: f64,
    pub opinions_this_month: usize,
    pub opinions_last_month: usize,
    pub most_cited_opinions: Vec<(String, usize)>,
}

/// Statistics for citations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CitationStatistics {
    pub total_citations: usize,
    pub unique_cases_cited: usize,
    pub citations_by_treatment: Vec<(String, usize)>,
    pub most_cited_cases: Vec<CaseCitation>,
    pub citation_trends: Vec<CitationTrend>,
}

/// Case citation information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CaseCitation {
    pub case_name: String,
    pub citation: String,
    pub times_cited: usize,
    pub positive_treatments: usize,
    pub negative_treatments: usize,
}

/// Citation trend over time
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CitationTrend {
    pub month: String,
    pub year: i32,
    pub citation_count: usize,
    pub unique_cases: usize,
}

impl Default for OrderFilter {
    fn default() -> Self {
        Self {
            case_id: None,
            judge_id: None,
            order_type: None,
            status: None,
            is_sealed: None,
            start_date: None,
            end_date: None,
            requires_service: None,
            limit: Some(100),
            offset: None,
        }
    }
}

impl Default for OpinionFilter {
    fn default() -> Self {
        Self {
            case_id: None,
            author_judge_id: None,
            opinion_type: None,
            status: None,
            disposition: None,
            is_published: None,
            is_precedential: None,
            start_date: None,
            end_date: None,
            keywords: None,
            limit: Some(100),
            offset: None,
        }
    }
}