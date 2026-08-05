use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;

use nanopulse::keypair::get_short_id;

use super::{ApiError, ListRequest, ListResponse};
use crate::storage::{device, workspace};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct DeviceParams {
    /// workspace name.
    pub workspace_name: String,

    /// Device name.
    pub device_name: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct WorkspaceParam {
    /// workspace name.
    pub workspace_name: String,
}

/// Create device.
#[utoipa::path(
    method(post),
    tag = "devices",
    path = "/api/workspaces/{workspace_name}/devices",
    params (
        WorkspaceParam,
    ),
    request_body = device::Device,
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn device_create(
    path: axum::extract::Path<WorkspaceParam>,
    Json(mut body): Json<device::Device>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let w = workspace::get_by_name(&path.workspace_name).await?;

    body.workspace_id = w.id;
    body.short_id = get_short_id(body.public_key.as_ref()).into();

    device::create(body).await?;
    Ok(StatusCode::OK)
}

/// Get device.
#[utoipa::path(
    method(get),
    tag = "devices",
    path = "/api/workspaces/{workspace_name}/devices/{device_name}",
    params (
        DeviceParams,
    ),
    responses(
        (status = OK, description = "Success", body = device::Device),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn device_get(
    path: axum::extract::Path<DeviceParams>,
) -> Result<Json<device::Device>, (StatusCode, Json<ApiError>)> {
    Ok(Json(device::get_by_name(&path.device_name).await?))
}

/// Update device.
#[utoipa::path(
    method(put),
    tag = "devices",
    path = "/api/workspaces/{workspace_name}/devices/{device_name}",
    params (
        DeviceParams,
    ),
    request_body = device::Device,
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn device_update(
    path: axum::extract::Path<DeviceParams>,
    Json(body): Json<device::Device>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let d = device::get_by_name(&path.device_name).await?;

    device::update(
        d.id,
        &device::DeviceChangeSet {
            pin: Some(body.pin),
            name: Some(body.name.clone()),
            description: Some(body.description.clone()),
            ..Default::default()
        },
    )
    .await?;

    Ok(StatusCode::OK)
}

/// List devices.
#[utoipa::path(
    method(get),
    tag = "devices",
    path = "/api/workspaces/{workspace_name}/devices",
    params (
        WorkspaceParam,
        ListRequest,
    ),
    responses(
        (status = OK, description = "Success", body = ListResponse<device::Device>),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn device_list(
    path: axum::extract::Path<WorkspaceParam>,
    query: axum::extract::Query<ListRequest>,
) -> Result<Json<ListResponse<device::Device>>, (StatusCode, Json<ApiError>)> {
    let workspace_name = workspace::get_id_for_name(&path.workspace_name).await?;

    Ok(Json(ListResponse {
        total_count: device::get_count(workspace_name).await? as usize,
        result: device::list(
            workspace_name,
            query.limit.unwrap_or_default() as i64,
            query.offset.unwrap_or_default() as i64,
        )
        .await?,
    }))
}

/// Delete device.
#[utoipa::path(
    method(delete),
    tag = "devices",
    path = "/api/workspaces/{workspace_name}/devices/{device_name}",
    params (
        DeviceParams,
    ),
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn device_delete(
    path: axum::extract::Path<DeviceParams>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let d = device::get_by_name(&path.device_name).await?;
    device::delete(d.id).await?;
    Ok(StatusCode::OK)
}
