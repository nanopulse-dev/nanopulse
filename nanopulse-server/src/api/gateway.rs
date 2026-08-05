use std::time::Duration;

use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use super::{ApiError, ListResponse};
use crate::errors::Error;
use crate::region;
use crate::storage::{gateway, workspace};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct GatewayIdParam {
    /// Gateway ID.
    pub gateway_name: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct WorkspaceParam {
    /// workspace name.
    pub workspace_name: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct GatewayIdParams {
    /// workspace ID.
    pub workspace_name: String,

    /// Gateway ID.
    pub gateway_name: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListParams {
    /// The offset in the result set.
    #[param(default = 0)]
    pub offset: Option<usize>,

    /// The number of items to return.
    #[param(default = 10)]
    pub limit: Option<usize>,
}

#[derive(Deserialize, ToSchema)]
pub struct GatewayAllowKeyExchangeRequest {
    /// Duration in seconds.
    pub duration_sec: usize,
}

/// Create gateway.
#[utoipa::path(
    method(post),
    tag = "gateways",
    path = "/api/workspaces/{workspace_name}/gateways",
    params(
        WorkspaceParam,
    ),
    request_body = gateway::Gateway,
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn gateway_create(
    path: axum::extract::Path<WorkspaceParam>,
    Json(mut body): Json<gateway::Gateway>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let w = workspace::get_by_name(&path.workspace_name).await?;
    body.workspace_id = w.id;

    let r = region::get(&body.region_module)?;
    let gateway_name = body.name.clone();

    gateway::create(body).await?;
    crate::gateway::sync_configuration(&gateway_name, r)
        .await
        .map_err(Error::Anyhow)?;

    Ok(StatusCode::OK)
}

/// Get gateway.
#[utoipa::path(
    method(get),
    tag = "gateways",
    path = "/api/workspaces/{workspace_name}/gateways/{gateway_name}",
    params (
        GatewayIdParams,
    ),
    responses(
        (status = OK, description = "Success", body = gateway::Gateway),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn gateway_get(
    path: axum::extract::Path<GatewayIdParams>,
) -> Result<Json<gateway::Gateway>, (StatusCode, Json<ApiError>)> {
    Ok(Json(gateway::get_by_name(&path.gateway_name).await?))
}

/// Update gateway.
#[utoipa::path(
    method(put),
    tag = "gateways",
    path = "/api/workspaces/{workspace_name}/gateways/{gateway_name}",
    params(
        GatewayIdParams,
    ),
    request_body = gateway::Gateway,
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn gateway_update(
    path: axum::extract::Path<GatewayIdParams>,
    Json(body): Json<gateway::Gateway>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let mut g = gateway::get_by_name(&path.gateway_name).await?;
    g.description = body.description.clone();
    g.region_module = body.region_module.clone();

    let r = region::get(&g.region_module)?;
    let gateway_name = g.name.clone();

    gateway::update(g).await?;

    crate::gateway::sync_configuration(&gateway_name, r)
        .await
        .map_err(Error::Anyhow)?;

    Ok(StatusCode::OK)
}

/// Delete gateway.
#[utoipa::path(
    method(delete),
    tag = "gateways",
    path = "/api/workspaces/{workspace_name}/gateways/{gateway_name}",
    params(
        GatewayIdParams,
    ),
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn gateway_delete(
    path: axum::extract::Path<GatewayIdParams>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let g = gateway::get_by_name(&path.gateway_name).await?;
    gateway::delete(g.id).await?;
    Ok(StatusCode::OK)
}

/// List gateways.
#[utoipa::path(
    method(get),
    tag = "gateways",
    path = "/api/workspaces/{workspace_name}/gateways",
    params (
        WorkspaceParam,
        ListParams,
    ),
    responses(
        (status = OK, description = "Success", body = ListResponse<gateway::Gateway>),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn gateway_list(
    path: axum::extract::Path<WorkspaceParam>,
    query: axum::extract::Query<ListParams>,
) -> Result<Json<ListResponse<gateway::Gateway>>, (StatusCode, Json<ApiError>)> {
    let wid = workspace::get_id_for_name(&path.workspace_name).await?;

    Ok(Json(ListResponse {
        total_count: gateway::get_count(wid).await? as usize,
        result: gateway::list(
            wid,
            query.limit.unwrap_or_default() as i64,
            query.offset.unwrap_or_default() as i64,
        )
        .await?,
    }))
}

/// Set allow key exchange through gateway.
#[utoipa::path(
    method(post),
    tag = "gateways",
    path = "/api/workspaces/{workspace_name}/gateways/{gateway_name}/allow-key-exchange",
    params (
        GatewayIdParams,
    ),
    request_body = GatewayAllowKeyExchangeRequest,
    responses(
        (status = OK, description = "Success"),
        (status = 400, description = "Invalid parameters", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn gateway_allow_key_exchange(
    path: axum::extract::Path<GatewayIdParams>,
    Json(body): Json<GatewayAllowKeyExchangeRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    gateway::allow_key_exchange(
        &path.gateway_name,
        Duration::from_secs(body.duration_sec as u64),
    )
    .await?;

    Ok(StatusCode::OK)
}
