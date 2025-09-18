//! REST API handlers for judicial orders
//!
//! This module provides HTTP endpoints for managing judicial orders,
//! templates, and electronic signatures.

use crate::domain::order::{
    JudicialOrder, OrderTemplate, OrderType, OrderStatus, ElectronicSignature,
    ServiceRecord, ServiceMethod, ServiceStatus, TemplateVariable
};
use crate::error::{ApiError, ApiResult};
use crate::ports::document_repository::{DocumentRepository, OrderFilter, OrderStatistics};
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

/// Request to create a new judicial order
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrderRequest {
    pub case_id: String,
    pub judge_id: String,
    pub order_type: OrderType,
    pub title: String,
    pub content: String,
    pub is_sealed: bool,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub related_motions: Vec<String>,
}

/// Request to update an order
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrderRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<OrderStatus>,
    pub is_sealed: Option<bool>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
}

/// Request to sign an order electronically
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignOrderRequest {
    pub judge_id: String,
    pub judge_name: String,
    pub certificate_id: String,
}

/// Request to add service record
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddServiceRequest {
    pub party_id: String,
    pub party_name: String,
    pub method: ServiceMethod,
    pub served_by: Option<String>,
}

/// Request to create order from template
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFromTemplateRequest {
    pub template_id: String,
    pub case_id: String,
    pub judge_id: String,
    pub variables: HashMap<String, String>,
}

/// Request to create an order template
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub order_type: OrderType,
    pub description: String,
    pub template_content: String,
    pub variables: Vec<TemplateVariable>,
    pub required_attachments: Vec<String>,
    pub default_service_method: ServiceMethod,
}

/// Response for order lists
#[derive(Debug, Serialize, ToSchema)]
pub struct OrderListResponse {
    pub orders: Vec<JudicialOrder>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

/// Create a new judicial order
#[utoipa::path(
    post,
    path = "/api/orders",
    request_body = CreateOrderRequest,
    responses(
        (status = 201, description = "Order created successfully", body = JudicialOrder),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn create_order(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateOrderRequest = serde_json::from_slice(body)?;

    let mut order = JudicialOrder::new(
        request.case_id,
        request.judge_id,
        request.order_type,
        request.title,
        request.content,
    );

    order.is_sealed = request.is_sealed;
    order.effective_date = request.effective_date;
    order.expiration_date = request.expiration_date;
    order.related_motions = request.related_motions;

    // Use tenant-specific store
    let repo = get_tenant_repo!(&req);
    let created = repo.create_order(order)?;
    
    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&created)?)
        .build())
}

/// Get an order by ID
#[utoipa::path(
    get,
    path = "/api/orders/{order_id}",
    params(
        ("order_id" = String, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Order found", body = JudicialOrder),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn get_order(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let order_id = params.get("order_id")
        .ok_or_else(|| ApiError::BadRequest("Order ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let order = repo.get_order(order_id)?
        .ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&order)?)
        .build())
}

/// Update an order
#[utoipa::path(
    patch,
    path = "/api/orders/{order_id}",
    params(
        ("order_id" = String, Path, description = "Order ID")
    ),
    request_body = UpdateOrderRequest,
    responses(
        (status = 200, description = "Order updated successfully", body = JudicialOrder),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn update_order(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let order_id = params.get("order_id")
        .ok_or_else(|| ApiError::BadRequest("Order ID required".to_string()))?;
    
    let body = req.body();
    let request: UpdateOrderRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut order = repo.get_order(order_id)?
        .ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    
    if let Some(title) = request.title {
        order.title = title;
    }
    if let Some(content) = request.content {
        order.content = content;
    }
    if let Some(status) = request.status {
        order.status = status;
    }
    if let Some(sealed) = request.is_sealed {
        order.is_sealed = sealed;
    }
    if let Some(effective) = request.effective_date {
        order.effective_date = Some(effective);
    }
    if let Some(expiration) = request.expiration_date {
        order.expiration_date = Some(expiration);
    }
    
    order.updated_at = Utc::now();
    let updated = repo.update_order(order)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Delete an order
#[utoipa::path(
    delete,
    path = "/api/orders/{order_id}",
    params(
        ("order_id" = String, Path, description = "Order ID")
    ),
    responses(
        (status = 204, description = "Order deleted successfully"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn delete_order(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let order_id = params.get("order_id")
        .ok_or_else(|| ApiError::BadRequest("Order ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    repo.delete_order(order_id)?;
    
    Ok(ResponseBuilder::new(204).build())
}

/// List orders with filters
#[utoipa::path(
    get,
    path = "/api/orders",
    params(
        ("case_id" = Option<String>, Query, description = "Filter by case ID"),
        ("judge_id" = Option<String>, Query, description = "Filter by judge ID"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("is_sealed" = Option<bool>, Query, description = "Filter by sealed status"),
        ("limit" = Option<usize>, Query, description = "Maximum number of results"),
        ("offset" = Option<usize>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List of orders", body = OrderListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn list_orders(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let filter = OrderFilter {
        case_id: query_map.get("case_id").map(|s| s.to_string()),
        judge_id: query_map.get("judge_id").map(|s| s.to_string()),
        status: query_map.get("status").and_then(|s| serde_json::from_str(s).ok()),
        is_sealed: query_map.get("is_sealed").and_then(|s| s.parse().ok()),
        limit: query_map.get("limit").and_then(|s| s.parse().ok()),
        offset: query_map.get("offset").and_then(|s| s.parse().ok()),
        ..Default::default()
    };
    
    let limit = filter.limit.unwrap_or(100);
    let offset = filter.offset.unwrap_or(0);
    
    let repo = get_tenant_repo!(&req);
    let orders = repo.list_orders(filter)?;
    
    let response = OrderListResponse {
        total: orders.len(),
        orders,
        offset,
        limit,
    };
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

/// Sign an order electronically
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/sign",
    params(
        ("order_id" = String, Path, description = "Order ID")
    ),
    request_body = SignOrderRequest,
    responses(
        (status = 200, description = "Order signed successfully", body = JudicialOrder),
        (status = 404, description = "Order not found"),
        (status = 400, description = "Order already signed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn sign_order(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let order_id = params.get("order_id")
        .ok_or_else(|| ApiError::BadRequest("Order ID required".to_string()))?;
    
    let body = req.body();
    let request: SignOrderRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut order = repo.get_order(order_id)?
        .ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    
    if order.signature.is_some() {
        return Err(ApiError::BadRequest("Order already signed".to_string()));
    }
    
    let signature = ElectronicSignature {
        judge_id: request.judge_id,
        judge_name: request.judge_name,
        signature_hash: uuid::Uuid::new_v4().to_string(),
        signed_at: Utc::now(),
        certificate_id: request.certificate_id,
        ip_address: "127.0.0.1".to_string(), // Would get from request context
    };
    
    order.sign(signature);
    let updated = repo.update_order(order)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Issue an order
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/issue",
    params(
        ("order_id" = String, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Order issued successfully", body = JudicialOrder),
        (status = 404, description = "Order not found"),
        (status = 400, description = "Order not signed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn issue_order(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let order_id = params.get("order_id")
        .ok_or_else(|| ApiError::BadRequest("Order ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let mut order = repo.get_order(order_id)?
        .ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    
    if order.signature.is_none() {
        return Err(ApiError::BadRequest("Order must be signed before issuing".to_string()));
    }
    
    order.issue();
    let updated = repo.update_order(order)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Add service record to an order
#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/service",
    params(
        ("order_id" = String, Path, description = "Order ID")
    ),
    request_body = AddServiceRequest,
    responses(
        (status = 200, description = "Service record added", body = JudicialOrder),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn add_service_record(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let order_id = params.get("order_id")
        .ok_or_else(|| ApiError::BadRequest("Order ID required".to_string()))?;
    
    let body = req.body();
    let request: AddServiceRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let mut order = repo.get_order(order_id)?
        .ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    
    let record = ServiceRecord {
        party_id: request.party_id,
        party_name: request.party_name,
        method: request.method,
        served_at: None,
        served_by: request.served_by,
        proof_of_service: None,
        status: ServiceStatus::Pending,
    };
    
    order.add_service_record(record);
    let updated = repo.update_order(order)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Get orders by case
#[utoipa::path(
    get,
    path = "/api/cases/{case_id}/orders",
    params(
        ("case_id" = String, Path, description = "Case ID")
    ),
    responses(
        (status = 200, description = "List of orders for the case", body = Vec<JudicialOrder>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn get_orders_by_case(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let case_id = params.get("case_id")
        .ok_or_else(|| ApiError::BadRequest("Case ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let orders = repo.find_orders_by_case(case_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&orders)?)
        .build())
}

/// Get orders by judge
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/orders",
    params(
        ("judge_id" = String, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "List of orders by the judge", body = Vec<JudicialOrder>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn get_orders_by_judge(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params.get("judge_id")
        .ok_or_else(|| ApiError::BadRequest("Judge ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let orders = repo.find_orders_by_judge(judge_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&orders)?)
        .build())
}

/// Get pending signatures for a judge
#[utoipa::path(
    get,
    path = "/api/judges/{judge_id}/orders/pending-signatures",
    params(
        ("judge_id" = String, Path, description = "Judge ID")
    ),
    responses(
        (status = 200, description = "List of orders pending signature", body = Vec<JudicialOrder>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn get_pending_signatures(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let judge_id = params.get("judge_id")
        .ok_or_else(|| ApiError::BadRequest("Judge ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let orders = repo.find_pending_signatures(judge_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&orders)?)
        .build())
}

/// Get expiring orders
#[utoipa::path(
    get,
    path = "/api/orders/expiring",
    params(
        ("days" = Option<i64>, Query, description = "Number of days to look ahead")
    ),
    responses(
        (status = 200, description = "List of expiring orders", body = Vec<JudicialOrder>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn get_expiring_orders(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let days = query_map.get("days")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    
    let repo = get_tenant_repo!(&req);
    let orders = repo.find_expiring_orders(days)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&orders)?)
        .build())
}

/// Create a new order template
#[utoipa::path(
    post,
    path = "/api/templates/orders",
    request_body = CreateTemplateRequest,
    responses(
        (status = 201, description = "Template created successfully", body = OrderTemplate),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Order Templates"
)]
pub fn create_template(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateTemplateRequest = serde_json::from_slice(body)?;
    
    let mut template = OrderTemplate::new(
        request.name,
        request.order_type,
        request.description,
        request.template_content,
    );
    
    template.variables = request.variables;
    template.required_attachments = request.required_attachments;
    template.default_service_method = request.default_service_method;
    
    let repo = get_tenant_repo!(&req);
    let created = repo.create_template(template)?;
    
    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&created)?)
        .build())
}

/// Get an order template
#[utoipa::path(
    get,
    path = "/api/templates/orders/{template_id}",
    params(
        ("template_id" = String, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Template found", body = OrderTemplate),
        (status = 404, description = "Template not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Order Templates"
)]
pub fn get_template(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let template_id = params.get("template_id")
        .ok_or_else(|| ApiError::BadRequest("Template ID required".to_string()))?;
    
    let repo = get_tenant_repo!(&req);
    let template = repo.get_template(template_id)?
        .ok_or_else(|| ApiError::NotFound("Template not found".to_string()))?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&template)?)
        .build())
}

/// List order templates
#[utoipa::path(
    get,
    path = "/api/templates/orders",
    params(
        ("order_type" = Option<String>, Query, description = "Filter by order type")
    ),
    responses(
        (status = 200, description = "List of templates", body = Vec<OrderTemplate>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Order Templates"
)]
pub fn list_templates(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let order_type = query_map.get("order_type")
        .and_then(|s| serde_json::from_str(s).ok());
    
    let repo = get_tenant_repo!(&req);
    let templates = repo.list_templates(order_type)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&templates)?)
        .build())
}

/// Create order from template
#[utoipa::path(
    post,
    path = "/api/orders/from-template",
    request_body = CreateFromTemplateRequest,
    responses(
        (status = 201, description = "Order created from template", body = JudicialOrder),
        (status = 404, description = "Template not found"),
        (status = 400, description = "Missing required variables"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn create_from_template(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let request: CreateFromTemplateRequest = serde_json::from_slice(body)?;
    
    let repo = get_tenant_repo!(&req);
    let template = repo.get_template(&request.template_id)?
        .ok_or_else(|| ApiError::NotFound("Template not found".to_string()))?;
    
    // Check required variables
    for var in &template.variables {
        if var.required && !request.variables.contains_key(&var.name) {
            return Err(ApiError::BadRequest(format!("Missing required variable: {}", var.name)));
        }
    }
    
    let content = template.generate_content(&request.variables);
    
    let order = JudicialOrder::new(
        request.case_id,
        request.judge_id,
        template.order_type,
        template.name,
        content,
    );
    
    let created = repo.create_order(order)?;
    
    Ok(ResponseBuilder::new(201)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&created)?)
        .build())
}

/// Get order statistics
#[utoipa::path(
    get,
    path = "/api/orders/statistics",
    params(
        ("judge_id" = Option<String>, Query, description = "Filter by judge ID")
    ),
    responses(
        (status = 200, description = "Order statistics", body = OrderStatistics),
        (status = 500, description = "Internal server error")
    ),
    tag = "Judicial Orders"
)]
pub fn get_order_statistics(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    use crate::utils::query_parser::parse_query_string;
    
    let query_str = req.query();
    let query_params = parse_query_string(query_str);
    let mut query_map: HashMap<&str, &str> = HashMap::new();
    for (key, value) in query_params {
        query_map.insert(key, value);
    }
    
    let judge_id = query_map.get("judge_id").map(|s| *s);
    
    let repo = get_tenant_repo!(&req);
    let stats = repo.get_order_statistics(judge_id)?;
    
    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&stats)?)
        .build())
}

/// Update an order template
#[utoipa::path(
    put,
    path = "/api/templates/orders/{template_id}",
    params(
        ("template_id" = String, Path, description = "Template ID")
    ),
    request_body = OrderTemplate,
    responses(
        (status = 200, description = "Template updated", body = OrderTemplate),
        (status = 404, description = "Template not found")
    ),
    tag = "Order Templates"
)]
pub fn update_template(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let body = req.body();
    let mut template: OrderTemplate = serde_json::from_slice(body)?;

    let template_id = params.get("template_id").unwrap_or("").to_string();
    template.id = template_id;

    let repo = get_tenant_repo!(&req);
    let updated = repo.update_template(template)?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&updated)?)
        .build())
}

/// Delete an order template
#[utoipa::path(
    delete,
    path = "/api/templates/orders/{template_id}",
    params(
        ("template_id" = String, Path, description = "Template ID")
    ),
    responses(
        (status = 204, description = "Template deleted"),
        (status = 404, description = "Template not found")
    ),
    tag = "Order Templates"
)]
pub fn delete_template(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let template_id = params.get("template_id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);
    repo.delete_template(&template_id)?;

    Ok(ResponseBuilder::new(204).build())
}

/// Get active order templates
#[utoipa::path(
    get,
    path = "/api/templates/orders/active",
    responses(
        (status = 200, description = "List of active templates", body = Vec<OrderTemplate>)
    ),
    tag = "Order Templates"
)]
pub fn find_active_templates(req: Request, _params: Params) -> ApiResult<impl IntoResponse> {
    let repo = get_tenant_repo!(&req);
    let templates = repo.find_active_templates()?;

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&templates)?)
        .build())
}

/// Check if order is expired
#[utoipa::path(
    get,
    path = "/api/orders/{id}/is-expired",
    params(
        ("id" = String, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Success", body = bool),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Orders"
)]
pub fn check_order_expired(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params.get("id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);
    let order = repo.get_order(&id)?
        .ok_or_else(|| ApiError::NotFound(format!("Order {} not found", id)))?;

    let is_expired = order.is_expired();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&is_expired)?)
        .build())
}

/// Check if order requires immediate attention
#[utoipa::path(
    get,
    path = "/api/orders/{id}/requires-attention",
    params(
        ("id" = String, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Success", body = bool),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Orders"
)]
pub fn check_requires_attention(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let id = params.get("id").unwrap_or("").to_string();

    let repo = get_tenant_repo!(&req);
    let order = repo.get_order(&id)?
        .ok_or_else(|| ApiError::NotFound(format!("Order {} not found", id)))?;

    let requires_attention = order.requires_immediate_attention();

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&requires_attention)?)
        .build())
}

/// Generate order content from template
#[utoipa::path(
    post,
    path = "/api/templates/{template_id}/generate-content",
    params(
        ("template_id" = String, Path, description = "Template ID")
    ),
    request_body = HashMap<String, String>,
    responses(
        (status = 200, description = "Success", body = String),
        (status = 404, description = "Template not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Order Templates"
)]
pub fn generate_template_content(req: Request, params: Params) -> ApiResult<impl IntoResponse> {
    let template_id = params.get("template_id").unwrap_or("").to_string();

    let body_bytes = req.body();
    let body_str = std::str::from_utf8(body_bytes)
        .map_err(|e| ApiError::InvalidInput(format!("Invalid UTF-8: {}", e)))?;
    let values: std::collections::HashMap<String, String> = serde_json::from_str(body_str)
        .map_err(|e| ApiError::InvalidInput(format!("Invalid JSON: {}", e)))?;

    let repo = get_tenant_repo!(&req);
    let template = repo.get_template(&template_id)?
        .ok_or_else(|| ApiError::NotFound(format!("Template {} not found", template_id)))?;

    let content = template.generate_content(&values);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&content)?)
        .build())
}