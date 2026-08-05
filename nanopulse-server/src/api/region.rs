use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use super::{ApiError, ListResponse};
use crate::region;

#[derive(Debug, Serialize, ToSchema)]
pub struct RegionListItem {
    /// Module.
    pub module: String,

    /// Name.
    pub name: String,
}

/// List all regions.
#[utoipa::path(
    method(get),
    tag = "regions",
    path = "/api/regions",
    responses(
        (status = OK, description = "Success", body = ListResponse<RegionListItem>),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn region_list()
-> Result<Json<ListResponse<RegionListItem>>, (StatusCode, Json<ApiError>)> {
    let region_keys = region::get_keys()?;
    let mut regions = vec![];

    for key in &region_keys {
        let r = region::get(key)?;
        regions.push(RegionListItem {
            module: key.clone(),
            name: r.name.clone(),
        })
    }

    Ok(Json(ListResponse {
        total_count: regions.len(),
        result: regions,
    }))
}
