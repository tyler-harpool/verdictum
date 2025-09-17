use crate::domain;

use super::JsonResponse;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{
    conversions::IntoBody, IntoResponse, Params, Request, Response, ResponseBuilder,
};
use uuid::Uuid;

use utoipa::ToSchema;

/// Get all ToDo items
///
/// Returns a list of all active (non-deleted) ToDo items.
#[utoipa::path(
    get,
    path = "/api/todos",
    tags = ["todos"],
    description = "Retrieve all active ToDo items",
    responses(
        (status = 200, description = "List of ToDo items", body = Vec<ToDoModel>),
        (status = 500, description = "Internal Server Error")
    )
)]
pub(crate) fn get_all(_req: Request, _p: Params) -> Result<impl IntoResponse> {
    let todos = domain::ToDo::get_all()?;
    let models = ToDoListModel::from(
        todos
            .into_iter()
            .filter(|i| !i.is_deleted)
            .map(ToDoModel::from)
            .collect::<Vec<_>>(),
    );

    JsonResponse::from(models)
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
pub(crate) fn get_by_id(_req: Request, p: Params) -> Result<impl IntoResponse> {
    let id = p.get("id").expect("router guarantees id is set");
    let Ok(id) = Uuid::parse_str(id) else {
        return Ok(Response::new(400, "Bad Request"));
    };
    match domain::ToDo::get_by_id(id)? {
        Some(todo) => JsonResponse::from(ToDoModel::from(todo)),
        None => Ok(Response::new(404, "Not Found")),
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
pub(crate) fn delete_by_id(_req: Request, p: Params) -> Result<impl IntoResponse> {
    let id = p.get("id").expect("router guarantees id is set");
    let Ok(id) = Uuid::parse_str(id) else {
        return Ok(Response::new(400, "Bad Request"));
    };
    let Some(mut found) = domain::ToDo::get_by_id(id)? else {
        return Ok(Response::new(404, "Not Found"));
    };
    found.is_deleted = true;
    found.save()?;
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
pub(crate) fn toggle_by_id(_req: Request, p: Params) -> Result<impl IntoResponse> {
    let id = p.get("id").expect("router guarantees id is set");
    let Ok(id) = Uuid::parse_str(id) else {
        return Ok(Response::new(400, "Bad Request"));
    };
    let Some(mut found) = domain::ToDo::get_by_id(id)? else {
        return Ok(Response::new(404, "Not Found"));
    };
    found.is_completed = !found.is_completed;
    found.save()?;
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
pub(crate) fn create_todo(req: Request, _p: Params) -> Result<impl IntoResponse> {
    let Ok(model) = serde_json::from_slice::<CreateToDoModel>(req.body()) else {
        return Ok(Response::new(400, "Bad Request"));
    };
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

/// Response model for a list of ToDo items
struct ToDoListModel {
    items: Vec<ToDoModel>,
}

impl From<Vec<ToDoModel>> for ToDoListModel {
    fn from(value: Vec<ToDoModel>) -> Self {
        Self { items: value }
    }
}

impl IntoBody for ToDoListModel {
    fn into_body(self) -> Vec<u8> {
        serde_json::to_vec(&self.items).expect("Error while serializing ToDoListModel")
    }
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
