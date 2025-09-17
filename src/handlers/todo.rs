use crate::domain;
use crate::error::{ApiError, ApiResult, validation};

use serde::{Deserialize, Serialize};
use spin_sdk::http::{
    conversions::IntoBody, IntoResponse, Params, Request, Response, ResponseBuilder,
};
use uuid::Uuid;

use utoipa::ToSchema;

/// Query parameters for pagination
#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: usize,
    /// Number of items per page
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Filter by completion status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
}

fn default_page() -> usize { 1 }
fn default_limit() -> usize { 20 }

/// Paginated response wrapper
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    /// The items for the current page
    pub items: Vec<T>,
    /// Total number of items
    pub total: usize,
    /// Current page number
    pub page: usize,
    /// Number of items per page
    pub limit: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_previous: bool,
}

/// Get all ToDo items with pagination
///
/// Returns a paginated list of active (non-deleted) ToDo items.
#[utoipa::path(
    get,
    path = "/api/todos",
    tags = ["todos"],
    params(
        ("page" = Option<usize>, Query, description = "Page number (1-indexed)", minimum = 1, example = 1),
        ("limit" = Option<usize>, Query, description = "Number of items per page", minimum = 1, maximum = 100, example = 20),
        ("completed" = Option<bool>, Query, description = "Filter by completion status", example = false)
    ),
    description = "Retrieve all active ToDo items with pagination and filtering",
    responses(
        (status = 200, description = "Paginated list of ToDo items", body = PaginatedResponse<ToDoModel>),
        (status = 400, description = "Bad Request - Invalid pagination parameters", body = crate::error::ErrorResponse),
        (status = 500, description = "Internal Server Error", body = crate::error::ErrorResponse)
    )
)]
pub(crate) fn get_all(req: Request, _p: Params) -> ApiResult<impl IntoResponse> {
    // Parse query parameters
    let query_string = req.query();
    let params = parse_pagination_params(query_string)?;

    // Validate pagination parameters
    if params.page < 1 {
        return Err(ApiError::BadRequest("Page number must be >= 1".to_string()));
    }
    if params.limit < 1 || params.limit > 100 {
        return Err(ApiError::BadRequest("Limit must be between 1 and 100".to_string()));
    }

    let mut todos = domain::ToDo::get_all()?;

    // Filter out deleted items
    todos.retain(|t| !t.is_deleted);

    // Apply completion filter if specified
    if let Some(completed) = params.completed {
        todos.retain(|t| t.is_completed == completed);
    }

    // Calculate pagination
    let total = todos.len();
    let total_pages = (total + params.limit - 1) / params.limit;
    let start = (params.page - 1) * params.limit;
    let _end = std::cmp::min(start + params.limit, total);

    // Get the page items
    let items: Vec<ToDoModel> = todos
        .into_iter()
        .skip(start)
        .take(params.limit)
        .map(ToDoModel::from)
        .collect();

    let response = PaginatedResponse {
        items,
        total,
        page: params.page,
        limit: params.limit,
        total_pages,
        has_next: params.page < total_pages,
        has_previous: params.page > 1,
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&response)?)
        .build())
}

fn parse_pagination_params(query: &str) -> ApiResult<PaginationParams> {
    let mut params = PaginationParams {
        page: 1,
        limit: 20,
        completed: None,
    };

    if query.is_empty() {
        return Ok(params);
    }

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() != 2 || parts[1].is_empty() {
            continue;
        }

        match parts[0] {
            "page" => {
                params.page = parts[1].parse()
                    .map_err(|_| ApiError::BadRequest("Invalid page number".to_string()))?;
            }
            "limit" => {
                params.limit = parts[1].parse()
                    .map_err(|_| ApiError::BadRequest("Invalid limit value".to_string()))?;
            }
            "completed" => {
                if !parts[1].is_empty() {
                    params.completed = Some(parts[1].parse()
                        .map_err(|_| ApiError::BadRequest("Invalid completed value (use true/false)".to_string()))?);
                }
            }
            _ => {} // Ignore unknown parameters
        }
    }

    Ok(params)
}

/// Get a single ToDo item by ID
///
/// Retrieves a specific ToDo item using its UUID identifier.
#[utoipa::path(
    get,
    path = "/api/todos/{id}",
    tags = ["todos"],
    description = "Retrieve a ToDo item using its identifier",
    params(
        ("id" = Uuid, Path, description = "ToDo identifier")
    ),
    responses(
        (status = 200, description = "Desired ToDo item", body = ToDoModel),
        (status = 400, description = "Bad Request - Invalid UUID format"),
        (status = 404, description = "ToDo item was not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub(crate) fn get_by_id(_req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id)?;

    match domain::ToDo::get_by_id(id)? {
        Some(todo) => Ok(ResponseBuilder::new(200)
            .header("content-type", "application/json")
            .body(ToDoModel::from(todo))
            .build()),
        None => Err(ApiError::NotFound(format!("ToDo item with id {} not found", id))),
    }
}

/// Delete a ToDo item
///
/// Marks a ToDo item as deleted (soft delete). The item is not physically removed
/// from storage but marked with a deleted flag.
#[utoipa::path(
    delete,
    path = "/api/todos/{id}",
    tags = ["todos"],
    description = "Delete a ToDo item using its identifier",
    params(
        ("id" = Uuid, Path, description = "ToDo identifier")
    ),
    responses(
        (status = 204, description = "ToDo item successfully deleted"),
        (status = 400, description = "Bad Request - Invalid UUID format"),
        (status = 404, description = "ToDo item was not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub(crate) fn delete_by_id(_req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id)?;

    let mut todo = domain::ToDo::get_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("ToDo item with id {} not found", id)))?;

    todo.is_deleted = true;
    todo.save()?;

    Ok(Response::new(204, ()))
}

/// Toggle ToDo completion status
///
/// Changes the completion status of a ToDo item to its opposite state.
/// If the item is completed, it will be marked as incomplete and vice versa.
#[utoipa::path(
    post,
    path = "/api/todos/{id}/toggle",
    tags = ["todos"],
    description = "Toggle the completion status of a ToDo item",
    params(
        ("id" = Uuid, Path, description = "ToDo identifier")
    ),
    responses(
        (status = 204, description = "ToDo item status successfully toggled"),
        (status = 400, description = "Bad Request - Invalid UUID format"),
        (status = 404, description = "ToDo item was not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub(crate) fn toggle_by_id(_req: Request, p: Params) -> ApiResult<impl IntoResponse> {
    let id = p.get("id")
        .ok_or_else(|| ApiError::Internal("Missing path parameter 'id'".to_string()))?;

    let id = Uuid::parse_str(id)?;

    let mut todo = domain::ToDo::get_by_id(id)?
        .ok_or_else(|| ApiError::NotFound(format!("ToDo item with id {} not found", id)))?;

    todo.is_completed = !todo.is_completed;
    todo.save()?;

    Ok(Response::new(204, ()))
}

/// Create a new ToDo item
///
/// Creates a new ToDo item with the provided contents. The item will be
/// created in an incomplete state with a new UUID identifier.
#[utoipa::path(
    post,
    path = "/api/todos",
    tags = ["todos"],
    description = "Create a new ToDo item",
    request_body(
        content = CreateToDoModel,
        description = "ToDo item to create",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "ToDo item successfully created", body = ToDoModel,
         headers(
             ("location" = String, description = "URL of the created ToDo item")
         )
        ),
        (status = 400, description = "Bad Request - Invalid request body"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub(crate) fn create_todo(req: Request, _p: Params) -> ApiResult<impl IntoResponse> {
    let model: CreateToDoModel = serde_json::from_slice(req.body())?;

    // Validate the input
    validation::validate_todo_content(&model.contents)?;

    let new_todo = domain::ToDo::new(model.contents);
    new_todo.save()?;

    Ok(ResponseBuilder::new(201)
        .header("location", format!("/api/todos/{}", new_todo.id))
        .header("content-type", "application/json")
        .body(ToDoModel::from(new_todo))
        .build())
}

/// Request model for creating a new ToDo item
#[derive(Deserialize, ToSchema)]
#[schema(example = json!({ "contents": "Buy groceries" }))]
pub struct CreateToDoModel {
    /// The content/description of the ToDo item
    pub contents: String,
}


/// Response model for a ToDo item
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({ "id": "059c7906-ce72-4433-94df-441beb14d96a", "contents": "Buy Milk", "isCompleted": false}))]
pub struct ToDoModel {
    /// Unique identifier of the ToDo item
    id: Uuid,
    /// The content/description of the ToDo item
    contents: String,
    /// Indicates whether the ToDo item has been completed
    is_completed: bool,
}

impl IntoBody for ToDoModel {
    fn into_body(self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Error while serializing ToDoModel")
    }
}

impl From<domain::ToDo> for ToDoModel {
    fn from(value: domain::ToDo) -> Self {
        Self {
            id: value.id,
            contents: value.contents.clone(),
            is_completed: value.is_completed,
        }
    }
}
