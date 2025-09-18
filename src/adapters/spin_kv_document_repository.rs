//! Spin KV adapter for document repository
//!
//! This module provides a Spin KV Store implementation of the DocumentRepository trait.

use crate::domain::order::{JudicialOrder, OrderTemplate, OrderType, OrderStatus};
use crate::domain::opinion::{JudicialOpinion, OpinionDraft};
use crate::error::{ApiError, ApiResult};
use crate::ports::document_repository::{
    DocumentRepository, OrderFilter, OpinionFilter, OrderStatistics,
    OpinionStatistics, CitationStatistics, CaseCitation
};
use chrono::{DateTime, Datelike, Utc};
use spin_sdk::key_value::Store;
use std::collections::HashMap;

/// Spin KV implementation of DocumentRepository
pub struct SpinKvDocumentRepository {
    store: Store,
}

impl SpinKvDocumentRepository {
    /// Create repository with tenant ID
    pub fn new(tenant_id: &str) -> Result<Self, String> {
        // Use the tenant's existing store
        let store_name = tenant_id.to_lowercase();
        Store::open(&store_name)
            .map(|store| Self { store })
            .map_err(|e| format!("Failed to open store: {}", e))
    }

    /// Create repository with specific store name for multi-tenancy
    pub fn with_store(store_name: String) -> Self {
        let store = Store::open(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }

    fn order_key(id: &str) -> String {
        format!("order:{}", id)
    }

    fn template_key(id: &str) -> String {
        format!("template:{}", id)
    }

    fn opinion_key(id: &str) -> String {
        format!("opinion:{}", id)
    }

    fn draft_key(id: &str) -> String {
        format!("draft:{}", id)
    }
}


impl DocumentRepository for SpinKvDocumentRepository {
    // Order operations
    fn create_order(&self, order: JudicialOrder) -> ApiResult<JudicialOrder> {        let key = Self::order_key(&order.id);
        
        self.store.set_json(&key, &order)
            .map_err(|e| ApiError::Internal(format!("Failed to store order: {}", e)))?;
        
        // Update indices
        self.update_order_indices(&order)?;
        
        Ok(order)
    }

    fn get_order(&self, order_id: &str) -> ApiResult<Option<JudicialOrder>> {        let key = Self::order_key(order_id);
        
        self.store.get_json(&key)
            .map_err(|e| ApiError::Internal(format!("Failed to get order: {}", e)))
    }

    fn update_order(&self, order: JudicialOrder) -> ApiResult<JudicialOrder> {        let key = Self::order_key(&order.id);
        
        self.store.set_json(&key, &order)
            .map_err(|e| ApiError::Internal(format!("Failed to update order: {}", e)))?;
        
        // Update indices
        self.update_order_indices(&order)?;
        
        Ok(order)
    }

    fn delete_order(&self, order_id: &str) -> ApiResult<()> {        let key = Self::order_key(order_id);
        
        self.store.delete(&key)
            .map_err(|e| ApiError::Internal(format!("Failed to delete order: {}", e)))?;
        
        // Remove from indices
        self.remove_order_from_indices(order_id)?;
        
        Ok(())
    }

    fn list_orders(&self, filter: OrderFilter) -> ApiResult<Vec<JudicialOrder>> {        
        // Get all order IDs from index
        let order_ids: Vec<String> = self.store.get_json("index:orders")
            .map_err(|e| ApiError::Internal(format!("Failed to get order index: {}", e)))?
            .unwrap_or_default();
        
        let mut orders = Vec::new();
        for id in order_ids {
            if let Some(order) = self.get_order(&id)? {
                // Apply filters
                if filter.case_id.as_ref().map_or(true, |c| &order.case_id == c)
                    && filter.judge_id.as_ref().map_or(true, |j| &order.judge_id == j)
                    && filter.order_type.as_ref().map_or(true, |t| std::mem::discriminant(&order.order_type) == std::mem::discriminant(t))
                    && filter.status.as_ref().map_or(true, |s| std::mem::discriminant(&order.status) == std::mem::discriminant(s))
                    && filter.is_sealed.map_or(true, |s| order.is_sealed == s)
                    && filter.start_date.map_or(true, |d| order.created_at >= d)
                    && filter.end_date.map_or(true, |d| order.created_at <= d)
                {
                    orders.push(order);
                }
            }
        }
        
        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);
        orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(orders.into_iter().skip(offset).take(limit).collect())
    }

    fn find_orders_by_case(&self, case_id: &str) -> ApiResult<Vec<JudicialOrder>> {
        self.list_orders(OrderFilter {
            case_id: Some(case_id.to_string()),
            ..Default::default()
        })
    }

    fn find_orders_by_judge(&self, judge_id: &str) -> ApiResult<Vec<JudicialOrder>> {
        self.list_orders(OrderFilter {
            judge_id: Some(judge_id.to_string()),
            ..Default::default()
        })
    }

    fn find_pending_signatures(&self, judge_id: &str) -> ApiResult<Vec<JudicialOrder>> {
        self.list_orders(OrderFilter {
            judge_id: Some(judge_id.to_string()),
            status: Some(OrderStatus::PendingSignature),
            ..Default::default()
        })
    }

    fn find_expiring_orders(&self, days: i64) -> ApiResult<Vec<JudicialOrder>> {
        let future_date = Utc::now() + chrono::Duration::days(days);
        let orders = self.list_orders(OrderFilter::default())?;
        
        Ok(orders.into_iter()
            .filter(|o| o.expiration_date.map_or(false, |exp| exp <= future_date && exp > Utc::now()))
            .collect())
    }

    // Template operations
    fn create_template(&self, template: OrderTemplate) -> ApiResult<OrderTemplate> {        let key = Self::template_key(&template.id);
        
        self.store.set_json(&key, &template)
            .map_err(|e| ApiError::Internal(format!("Failed to store template: {}", e)))?;
        
        // Update template index
        let mut template_ids: Vec<String> = self.store.get_json("index:templates")
            .map_err(|e| ApiError::Internal(format!("Failed to get template index: {}", e)))?
            .unwrap_or_default();
        
        if !template_ids.contains(&template.id) {
            template_ids.push(template.id.clone());
            self.store.set_json("index:templates", &template_ids)
                .map_err(|e| ApiError::Internal(format!("Failed to update template index: {}", e)))?;
        }
        
        Ok(template)
    }

    fn get_template(&self, template_id: &str) -> ApiResult<Option<OrderTemplate>> {        let key = Self::template_key(template_id);
        
        self.store.get_json(&key)
            .map_err(|e| ApiError::Internal(format!("Failed to get template: {}", e)))
    }

    fn update_template(&self, template: OrderTemplate) -> ApiResult<OrderTemplate> {        let key = Self::template_key(&template.id);
        
        self.store.set_json(&key, &template)
            .map_err(|e| ApiError::Internal(format!("Failed to update template: {}", e)))?;
        
        Ok(template)
    }

    fn delete_template(&self, template_id: &str) -> ApiResult<()> {        let key = Self::template_key(template_id);
        
        self.store.delete(&key)
            .map_err(|e| ApiError::Internal(format!("Failed to delete template: {}", e)))?;
        
        // Remove from index
        let mut template_ids: Vec<String> = self.store.get_json("index:templates")
            .map_err(|e| ApiError::Internal(format!("Failed to get template index: {}", e)))?
            .unwrap_or_default();
        
        template_ids.retain(|id| id != template_id);
        self.store.set_json("index:templates", &template_ids)
            .map_err(|e| ApiError::Internal(format!("Failed to update template index: {}", e)))?;
        
        Ok(())
    }

    fn list_templates(&self, order_type: Option<OrderType>) -> ApiResult<Vec<OrderTemplate>> {        
        let template_ids: Vec<String> = self.store.get_json("index:templates")
            .map_err(|e| ApiError::Internal(format!("Failed to get template index: {}", e)))?
            .unwrap_or_default();
        
        let mut templates = Vec::new();
        for id in template_ids {
            if let Some(template) = self.get_template(&id)? {
                if order_type.as_ref().map_or(true, |t| std::mem::discriminant(&template.order_type) == std::mem::discriminant(t)) {
                    templates.push(template);
                }
            }
        }
        
        Ok(templates)
    }

    fn find_active_templates(&self) -> ApiResult<Vec<OrderTemplate>> {
        let templates = self.list_templates(None)?;
        Ok(templates.into_iter().filter(|t| t.is_active).collect())
    }

    // Opinion operations
    fn create_opinion(&self, opinion: JudicialOpinion) -> ApiResult<JudicialOpinion> {        let key = Self::opinion_key(&opinion.id);
        
        self.store.set_json(&key, &opinion)
            .map_err(|e| ApiError::Internal(format!("Failed to store opinion: {}", e)))?;
        
        // Update indices
        self.update_opinion_indices(&opinion)?;
        
        Ok(opinion)
    }

    fn get_opinion(&self, opinion_id: &str) -> ApiResult<Option<JudicialOpinion>> {        let key = Self::opinion_key(opinion_id);
        
        self.store.get_json(&key)
            .map_err(|e| ApiError::Internal(format!("Failed to get opinion: {}", e)))
    }

    fn update_opinion(&self, opinion: JudicialOpinion) -> ApiResult<JudicialOpinion> {        let key = Self::opinion_key(&opinion.id);
        
        self.store.set_json(&key, &opinion)
            .map_err(|e| ApiError::Internal(format!("Failed to update opinion: {}", e)))?;
        
        // Update indices
        self.update_opinion_indices(&opinion)?;
        
        Ok(opinion)
    }

    fn delete_opinion(&self, opinion_id: &str) -> ApiResult<()> {        let key = Self::opinion_key(opinion_id);
        
        self.store.delete(&key)
            .map_err(|e| ApiError::Internal(format!("Failed to delete opinion: {}", e)))?;
        
        // Remove from indices
        self.remove_opinion_from_indices(opinion_id)?;
        
        Ok(())
    }

    fn list_opinions(&self, filter: OpinionFilter) -> ApiResult<Vec<JudicialOpinion>> {        
        let opinion_ids: Vec<String> = self.store.get_json("index:opinions")
            .map_err(|e| ApiError::Internal(format!("Failed to get opinion index: {}", e)))?
            .unwrap_or_default();
        
        let mut opinions = Vec::new();
        for id in opinion_ids {
            if let Some(opinion) = self.get_opinion(&id)? {
                // Apply filters
                if filter.case_id.as_ref().map_or(true, |c| &opinion.case_id == c)
                    && filter.author_judge_id.as_ref().map_or(true, |j| &opinion.author_judge_id == j)
                    && filter.is_published.map_or(true, |p| opinion.is_published == p)
                    && filter.is_precedential.map_or(true, |p| opinion.is_precedential == p)
                    && filter.start_date.map_or(true, |d| opinion.created_at >= d)
                    && filter.end_date.map_or(true, |d| opinion.created_at <= d)
                {
                    // Check keywords if provided
                    if let Some(keywords) = &filter.keywords {
                        let content_lower = opinion.content.to_lowercase();
                        if keywords.iter().all(|k| content_lower.contains(&k.to_lowercase())) {
                            opinions.push(opinion);
                        }
                    } else {
                        opinions.push(opinion);
                    }
                }
            }
        }
        
        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);
        opinions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(opinions.into_iter().skip(offset).take(limit).collect())
    }

    fn find_opinions_by_case(&self, case_id: &str) -> ApiResult<Vec<JudicialOpinion>> {
        self.list_opinions(OpinionFilter {
            case_id: Some(case_id.to_string()),
            ..Default::default()
        })
    }

    fn find_opinions_by_author(&self, judge_id: &str) -> ApiResult<Vec<JudicialOpinion>> {
        self.list_opinions(OpinionFilter {
            author_judge_id: Some(judge_id.to_string()),
            ..Default::default()
        })
    }

    fn find_published_opinions(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> ApiResult<Vec<JudicialOpinion>> {
        self.list_opinions(OpinionFilter {
            is_published: Some(true),
            start_date: Some(start_date),
            end_date: Some(end_date),
            ..Default::default()
        })
    }

    fn search_opinions(&self, query: &str) -> ApiResult<Vec<JudicialOpinion>> {
        let keywords = query.split_whitespace().map(|s| s.to_string()).collect();
        self.list_opinions(OpinionFilter {
            keywords: Some(keywords),
            ..Default::default()
        })
    }

    fn find_precedential_opinions(&self) -> ApiResult<Vec<JudicialOpinion>> {
        self.list_opinions(OpinionFilter {
            is_published: Some(true),
            is_precedential: Some(true),
            ..Default::default()
        })
    }

    // Draft operations
    fn create_draft(&self, draft: OpinionDraft) -> ApiResult<OpinionDraft> {        let key = Self::draft_key(&draft.id);
        
        self.store.set_json(&key, &draft)
            .map_err(|e| ApiError::Internal(format!("Failed to store draft: {}", e)))?;
        
        // Update draft index for the opinion
        let index_key = format!("index:drafts:{}", draft.opinion_id);
        let mut draft_ids: Vec<String> = self.store.get_json(&index_key)
            .map_err(|e| ApiError::Internal(format!("Failed to get draft index: {}", e)))?
            .unwrap_or_default();
        
        if !draft_ids.contains(&draft.id) {
            draft_ids.push(draft.id.clone());
            self.store.set_json(&index_key, &draft_ids)
                .map_err(|e| ApiError::Internal(format!("Failed to update draft index: {}", e)))?;
        }
        
        // Mark other drafts as not current if this is current
        if draft.is_current {
            for other_id in &draft_ids {
                if other_id != &draft.id {
                    if let Some(mut other_draft) = self.get_draft(other_id)? {
                        other_draft.is_current = false;
                        self.update_draft(other_draft)?;
                    }
                }
            }
        }
        
        Ok(draft)
    }

    fn get_draft(&self, draft_id: &str) -> ApiResult<Option<OpinionDraft>> {        let key = Self::draft_key(draft_id);
        
        self.store.get_json(&key)
            .map_err(|e| ApiError::Internal(format!("Failed to get draft: {}", e)))
    }

    fn update_draft(&self, draft: OpinionDraft) -> ApiResult<OpinionDraft> {        let key = Self::draft_key(&draft.id);
        
        self.store.set_json(&key, &draft)
            .map_err(|e| ApiError::Internal(format!("Failed to update draft: {}", e)))?;
        
        Ok(draft)
    }

    fn list_drafts(&self, opinion_id: &str) -> ApiResult<Vec<OpinionDraft>> {        let index_key = format!("index:drafts:{}", opinion_id);
        
        let draft_ids: Vec<String> = self.store.get_json(&index_key)
            .map_err(|e| ApiError::Internal(format!("Failed to get draft index: {}", e)))?
            .unwrap_or_default();
        
        let mut drafts = Vec::new();
        for id in draft_ids {
            if let Some(draft) = self.get_draft(&id)? {
                drafts.push(draft);
            }
        }
        
        drafts.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(drafts)
    }

    fn get_current_draft(&self, opinion_id: &str) -> ApiResult<Option<OpinionDraft>> {
        let drafts = self.list_drafts(opinion_id)?;
        Ok(drafts.into_iter().find(|d| d.is_current))
    }

    // Statistics
    fn get_order_statistics(&self, judge_id: Option<&str>) -> ApiResult<OrderStatistics> {
        let filter = OrderFilter {
            judge_id: judge_id.map(|j| j.to_string()),
            ..Default::default()
        };
        
        let orders = self.list_orders(filter)?;
        let total = orders.len();
        
        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_status: HashMap<String, usize> = HashMap::new();
        let mut pending_signatures = 0;
        let mut sealed_orders = 0;
        let mut requiring_service = 0;
        
        let now = Utc::now();
        let this_month = orders.iter()
            .filter(|o| {
                let created = o.created_at.date_naive();
                let now_date = now.date_naive();
                created.month() == now_date.month() && created.year() == now_date.year()
            })
            .count();

        let last_month = now - chrono::Duration::days(30);
        let last_month_count = orders.iter()
            .filter(|o| {
                let created = o.created_at.date_naive();
                let last_date = last_month.date_naive();
                created.month() == last_date.month() && created.year() == last_date.year()
            })
            .count();
        
        for order in &orders {
            // Count by type
            let type_name = format!("{:?}", order.order_type);
            *by_type.entry(type_name).or_insert(0) += 1;
            
            // Count by status
            let status_name = format!("{:?}", order.status);
            *by_status.entry(status_name).or_insert(0) += 1;
            
            // Count specific conditions
            if matches!(order.status, OrderStatus::PendingSignature) {
                pending_signatures += 1;
            }
            if order.is_sealed {
                sealed_orders += 1;
            }
            if order.service_list.iter().any(|s| matches!(s.status, crate::domain::order::ServiceStatus::Pending)) {
                requiring_service += 1;
            }
        }
        
        Ok(OrderStatistics {
            total_orders: total,
            orders_by_type: by_type.into_iter().collect(),
            orders_by_status: by_status.into_iter().collect(),
            pending_signatures,
            sealed_orders,
            orders_requiring_service: requiring_service,
            average_time_to_signature: 0.0, // Would need to track this separately
            average_time_to_service: 0.0, // Would need to track this separately
            orders_this_month: this_month,
            orders_last_month: last_month_count,
        })
    }

    fn get_opinion_statistics(&self, judge_id: Option<&str>) -> ApiResult<OpinionStatistics> {
        let filter = OpinionFilter {
            author_judge_id: judge_id.map(|j| j.to_string()),
            ..Default::default()
        };
        
        let opinions = self.list_opinions(filter)?;
        let total = opinions.len();
        
        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_disposition: HashMap<String, usize> = HashMap::new();
        let mut published = 0;
        let mut precedential = 0;
        let mut total_length = 0;
        let mut total_citations = 0;
        
        let now = Utc::now();
        let this_month = opinions.iter()
            .filter(|o| {
                let created = o.created_at.date_naive();
                let now_date = now.date_naive();
                created.month() == now_date.month() && created.year() == now_date.year()
            })
            .count();

        let last_month = now - chrono::Duration::days(30);
        let last_month_count = opinions.iter()
            .filter(|o| {
                let created = o.created_at.date_naive();
                let last_date = last_month.date_naive();
                created.month() == last_date.month() && created.year() == last_date.year()
            })
            .count();
        
        for opinion in &opinions {
            // Count by type
            let type_name = format!("{:?}", opinion.opinion_type);
            *by_type.entry(type_name).or_insert(0) += 1;
            
            // Count by disposition
            let disp_name = format!("{:?}", opinion.disposition);
            *by_disposition.entry(disp_name).or_insert(0) += 1;
            
            // Count specific conditions
            if opinion.is_published {
                published += 1;
            }
            if opinion.is_precedential {
                precedential += 1;
            }
            
            // Calculate averages
            total_length += opinion.content.len();
            total_citations += opinion.legal_citations.len();
        }
        
        let avg_length = if total > 0 { total_length as f64 / total as f64 } else { 0.0 };
        let avg_citations = if total > 0 { total_citations as f64 / total as f64 } else { 0.0 };
        
        Ok(OpinionStatistics {
            total_opinions: total,
            opinions_by_type: by_type.into_iter().collect(),
            opinions_by_disposition: by_disposition.into_iter().collect(),
            published_opinions: published,
            precedential_opinions: precedential,
            average_opinion_length: avg_length,
            average_citations_per_opinion: avg_citations,
            opinions_this_month: this_month,
            opinions_last_month: last_month_count,
            most_cited_opinions: Vec::new(), // Would need external tracking
        })
    }

    fn get_citation_statistics(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> ApiResult<CitationStatistics> {
        let opinions = self.find_published_opinions(start_date, end_date)?;
        
        let mut total_citations = 0;
        let mut case_citations: HashMap<String, usize> = HashMap::new();
        let mut treatment_counts: HashMap<String, usize> = HashMap::new();
        
        for opinion in &opinions {
            for citation in &opinion.legal_citations {
                total_citations += 1;
                *case_citations.entry(citation.case_name.clone()).or_insert(0) += 1;
                
                let treatment_name = format!("{:?}", citation.treatment);
                *treatment_counts.entry(treatment_name).or_insert(0) += 1;
            }
        }
        
        // Get most cited cases
        let mut cited_cases: Vec<_> = case_citations.into_iter()
            .map(|(name, count)| CaseCitation {
                case_name: name,
                citation: String::new(), // Would need to track this
                times_cited: count,
                positive_treatments: 0, // Would need to track this
                negative_treatments: 0, // Would need to track this
            })
            .collect();
        
        cited_cases.sort_by(|a, b| b.times_cited.cmp(&a.times_cited));
        cited_cases.truncate(10); // Top 10 most cited
        
        Ok(CitationStatistics {
            total_citations,
            unique_cases_cited: cited_cases.len(),
            citations_by_treatment: treatment_counts.into_iter().collect(),
            most_cited_cases: cited_cases,
            citation_trends: Vec::new(), // Would need time-series tracking
        })
    }
}

impl SpinKvDocumentRepository {
    fn update_order_indices(&self, order: &JudicialOrder) -> ApiResult<()> {        
        // Update main order index
        let mut order_ids: Vec<String> = self.store.get_json("index:orders")
            .map_err(|e| ApiError::Internal(format!("Failed to get order index: {}", e)))?
            .unwrap_or_default();
        
        if !order_ids.contains(&order.id) {
            order_ids.push(order.id.clone());
            self.store.set_json("index:orders", &order_ids)
                .map_err(|e| ApiError::Internal(format!("Failed to update order index: {}", e)))?;
        }
        
        // Update case index
        let case_index_key = format!("index:orders:case:{}", order.case_id);
        let mut case_orders: Vec<String> = self.store.get_json(&case_index_key)
            .map_err(|e| ApiError::Internal(format!("Failed to get case order index: {}", e)))?
            .unwrap_or_default();
        
        if !case_orders.contains(&order.id) {
            case_orders.push(order.id.clone());
            self.store.set_json(&case_index_key, &case_orders)
                .map_err(|e| ApiError::Internal(format!("Failed to update case order index: {}", e)))?;
        }
        
        Ok(())
    }

    fn remove_order_from_indices(&self, order_id: &str) -> ApiResult<()> {        
        // Remove from main index
        let mut order_ids: Vec<String> = self.store.get_json("index:orders")
            .map_err(|e| ApiError::Internal(format!("Failed to get order index: {}", e)))?
            .unwrap_or_default();
        
        order_ids.retain(|id| id != order_id);
        self.store.set_json("index:orders", &order_ids)
            .map_err(|e| ApiError::Internal(format!("Failed to update order index: {}", e)))?;
        
        Ok(())
    }

    fn update_opinion_indices(&self, opinion: &JudicialOpinion) -> ApiResult<()> {        
        // Update main opinion index
        let mut opinion_ids: Vec<String> = self.store.get_json("index:opinions")
            .map_err(|e| ApiError::Internal(format!("Failed to get opinion index: {}", e)))?
            .unwrap_or_default();
        
        if !opinion_ids.contains(&opinion.id) {
            opinion_ids.push(opinion.id.clone());
            self.store.set_json("index:opinions", &opinion_ids)
                .map_err(|e| ApiError::Internal(format!("Failed to update opinion index: {}", e)))?;
        }
        
        // Update case index
        let case_index_key = format!("index:opinions:case:{}", opinion.case_id);
        let mut case_opinions: Vec<String> = self.store.get_json(&case_index_key)
            .map_err(|e| ApiError::Internal(format!("Failed to get case opinion index: {}", e)))?
            .unwrap_or_default();
        
        if !case_opinions.contains(&opinion.id) {
            case_opinions.push(opinion.id.clone());
            self.store.set_json(&case_index_key, &case_opinions)
                .map_err(|e| ApiError::Internal(format!("Failed to update case opinion index: {}", e)))?;
        }
        
        Ok(())
    }

    fn remove_opinion_from_indices(&self, opinion_id: &str) -> ApiResult<()> {        
        // Remove from main index
        let mut opinion_ids: Vec<String> = self.store.get_json("index:opinions")
            .map_err(|e| ApiError::Internal(format!("Failed to get opinion index: {}", e)))?
            .unwrap_or_default();
        
        opinion_ids.retain(|id| id != opinion_id);
        self.store.set_json("index:opinions", &opinion_ids)
            .map_err(|e| ApiError::Internal(format!("Failed to update opinion index: {}", e)))?;
        
        Ok(())
    }
}