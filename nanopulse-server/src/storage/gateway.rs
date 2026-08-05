use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_query::{Asterisk, Expr, ExprTrait, Order, Query, SqliteQueryBuilder, enum_def};
use sea_query_sqlx::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tracing::info;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::errors::Error;
use crate::region;
use crate::storage::{NAME_REGEX, get_conn};

#[derive(
    Default, Debug, Clone, ToSchema, Deserialize, Serialize, FromRow, Validate, PartialEq, Eq,
)]
#[enum_def]
#[serde(default)]
pub struct Gateway {
    #[serde(skip)]
    pub id: i64,
    #[serde(skip)]
    pub workspace_id: i64,

    /// Name.
    /// Valid characters are a-z, 0-9 and -. Example: test-gateway-123.
    #[validate(regex(path = *NAME_REGEX))]
    pub name: String,

    /// Created-at timestamp.
    pub created_at: DateTime<Utc>,

    /// Updated-at timestamp.
    pub updated_at: DateTime<Utc>,

    /// Last heartbeat timestamp.
    pub last_heartbeat_at: Option<DateTime<Utc>>,

    /// Description.
    pub description: String,

    /// Region module.
    #[validate(custom(function = "validate_region_module"))]
    pub region_module: String,

    /// Allow key-exchange requests until given timestamp.
    pub allow_key_exchange_until: Option<DateTime<Utc>>,
}

fn validate_region_module(region_mod: &str) -> Result<(), ValidationError> {
    if region::get(region_mod).is_err() {
        return Err(ValidationError::new("region_module does not exist"));
    }

    Ok(())
}

pub async fn create(g: Gateway) -> Result<Gateway, Error> {
    g.validate()?;

    let (sql, values) = Query::insert()
        .into_table(GatewayIden::Table)
        .columns([
            GatewayIden::WorkspaceId,
            GatewayIden::Name,
            GatewayIden::CreatedAt,
            GatewayIden::UpdatedAt,
            GatewayIden::Description,
            GatewayIden::RegionModule,
        ])
        .values_panic([
            g.workspace_id.into(),
            g.name.into(),
            Utc::now().into(),
            Utc::now().into(),
            g.description.into(),
            g.region_module.into(),
        ])
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let g: Gateway = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(gateway_name = g.name, "gateway created");

    Ok(g)
}

pub async fn get(id: i64) -> Result<Gateway, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(GatewayIden::Table)
        .and_where(Expr::col(GatewayIden::Id).is(id))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn get_by_name(name: &str) -> Result<Gateway, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(GatewayIden::Table)
        .and_where(Expr::col(GatewayIden::Name).is(name))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn update(g: Gateway) -> Result<Gateway, Error> {
    g.validate()?;

    let (sql, values) = Query::update()
        .table(GatewayIden::Table)
        .values([
            (GatewayIden::Description, g.description.into()),
            (GatewayIden::RegionModule, g.region_module.into()),
        ])
        .and_where(Expr::col(GatewayIden::Id).is(g.id))
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let g: Gateway = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(gateway_name = g.name, "gateway updated");

    Ok(g)
}

pub async fn delete(id: i64) -> Result<(), Error> {
    let (sql, values) = Query::delete()
        .from_table(GatewayIden::Table)
        .and_where(Expr::col(GatewayIden::Id).is(id))
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let g: Gateway = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(gateway_name = g.name, "gateway deleted");

    Ok(())
}

pub async fn get_count(workspace_id: i64) -> Result<i64, Error> {
    let (sql, values) = Query::select()
        .from(GatewayIden::Table)
        .expr(Expr::count(Expr::cust("*")))
        .and_where(Expr::col(GatewayIden::WorkspaceId).is(workspace_id))
        .build_sqlx(SqliteQueryBuilder);

    let row: (i64,) = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    Ok(row.0)
}

pub async fn list(workspace_id: i64, limit: i64, offset: i64) -> Result<Vec<Gateway>, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(GatewayIden::Table)
        .and_where(Expr::col(GatewayIden::WorkspaceId).is(workspace_id))
        .order_by(GatewayIden::Name, Order::Asc)
        .limit(limit as u64)
        .offset(offset as u64)
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_all(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn heartbeat_update(name: &str) -> Result<(), Error> {
    let (sql, values) = Query::update()
        .table(GatewayIden::Table)
        .values([(GatewayIden::LastHeartbeatAt, Utc::now().into())])
        .and_where(Expr::col(GatewayIden::Name).is(name))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .execute(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(gateway_name = %name, "Gateway updated");

    Ok(())
}

pub async fn allow_key_exchange(name: &str, duration: Duration) -> Result<(), Error> {
    let ts = Utc::now() + duration;

    let (sql, values) = Query::update()
        .table(GatewayIden::Table)
        .values([(GatewayIden::AllowKeyExchangeUntil, ts.into())])
        .and_where(Expr::col(GatewayIden::Name).is(name))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .execute(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(gateway_name = %name, until = %ts, "Set allow key exchange");

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{storage, test::prepare};

    pub async fn create_gateway() -> Gateway {
        let w = storage::workspace::test::create_workspace().await;

        create(Gateway {
            workspace_id: w.id,
            name: "test-gateway".into(),
            region_module: "regions.lora.eu868".into(),
            ..Default::default()
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_validation() {
        let _guard = prepare().await;
        assert!(Gateway::default().validate().is_err());
        assert!(
            Gateway {
                name: "test-gateway-123".into(),
                region_module: "regions.lora.eu868".into(),
                ..Default::default()
            }
            .validate()
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_storage() {
        let _guard = prepare().await;

        let w = storage::workspace::test::create_workspace().await;

        let g = Gateway {
            workspace_id: w.id,
            name: "test-gateway-123".into(),
            description: "Test gateway".into(),
            region_module: "regions.lora.eu868".into(),
            ..Default::default()
        };
        let g = create(g).await.unwrap();

        let g_get = get(g.id).await.unwrap();
        assert_eq!(g, g_get);

        let g_get = get_by_name(&g.name).await.unwrap();
        assert_eq!(g, g_get);

        assert_eq!(1, get_count(w.id).await.unwrap());
        assert_eq!(vec![g.clone()], list(w.id, 10, 0).await.unwrap());

        let mut g_update = g.clone();
        g_update.description = "New description".into();
        let _ = update(g_update.clone()).await.unwrap();
        let g_get = get(g_update.id).await.unwrap();
        assert_eq!(&g_update.description, &g_get.description);

        delete(g.id).await.unwrap();
        assert!(delete(w.id).await.is_err());
    }
}
