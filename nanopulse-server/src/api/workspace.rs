use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;

use super::{ApiError, ListRequest, ListResponse};
use crate::storage::workspace;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct WorkspaceParam {
    /// Workspace name.
    pub workspace_name: String,
}

/// Get workspace.
#[utoipa::path(
    method(get),
    tag = "workspaces",
    path = "/api/workspaces/{workspace_name}",
    params(
        WorkspaceParam,
    ),
    responses(
        (status = OK, description = "Success", body = workspace::Workspace),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn workspace_get(
    params: axum::extract::Path<WorkspaceParam>,
) -> Result<Json<workspace::Workspace>, (StatusCode, Json<ApiError>)> {
    Ok(Json(workspace::get_by_name(&params.workspace_name).await?))
}

/// Create workspace.
#[utoipa::path(
    method(post),
    tag = "workspaces",
    path = "/api/workspaces",
    request_body = workspace::Workspace,
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn workspace_create(
    Json(body): Json<workspace::Workspace>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    workspace::create(body).await?;
    Ok(StatusCode::OK)
}

/// Update workspace.
#[utoipa::path(
    method(put),
    tag = "workspaces",
    path = "/api/workspaces/{workspace_name}",
    params(
        WorkspaceParam,
    ),
    request_body = workspace::Workspace,
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn workspace_update(
    params: axum::extract::Path<WorkspaceParam>,
    Json(body): Json<workspace::Workspace>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let mut p = workspace::get_by_name(&params.workspace_name).await?;
    p.description = body.description;
    workspace::update(p).await?;

    Ok(StatusCode::OK)
}

/// Delete workspace.
#[utoipa::path(
    method(delete),
    tag = "workspaces",
    path = "/api/workspaces/{workspace_name}",
    params(
        WorkspaceParam,
    ),
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn workspace_delete(
    params: axum::extract::Path<WorkspaceParam>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let p = workspace::get_by_name(&params.workspace_name).await?;
    workspace::delete(p.id).await?;
    Ok(StatusCode::OK)
}

/// List workspaces.
#[utoipa::path(
    method(get),
    tag = "workspaces",
    path = "/api/workspaces",
    params(
        ListRequest
    ),
    responses(
        (status = OK, description = "Success", body = ListResponse<workspace::Workspace>),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn workspace_list(
    query: axum::extract::Query<ListRequest>,
) -> Result<Json<ListResponse<workspace::Workspace>>, (StatusCode, Json<ApiError>)> {
    Ok(Json(ListResponse {
        total_count: workspace::get_count().await? as usize,
        result: workspace::list(
            query.limit.unwrap_or_default() as i64,
            query.offset.unwrap_or_default() as i64,
        )
        .await?,
    }))
}
