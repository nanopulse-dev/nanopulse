use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_query::{Asterisk, Expr, ExprTrait, Order, Query, SqliteQueryBuilder, enum_def};
use sea_query_sqlx::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::info;
use utoipa::ToSchema;
use validator::Validate;

use crate::errors::Error;
use crate::storage::{NAME_REGEX, get_conn};

#[derive(
    Default, Debug, Clone, Serialize, Deserialize, FromRow, ToSchema, Validate, PartialEq, Eq,
)]
#[enum_def]
#[serde(default)]
pub struct Workspace {
    #[serde(skip)]
    pub id: i64,

    /// Name.
    /// Valid characters are a-z, 0-9 and -. Example: test-workspace-123.
    #[validate(regex(path = *NAME_REGEX))]
    pub name: String,

    /// Created-at timestamp.
    pub created_at: DateTime<Utc>,

    /// Updated-at timestamp.
    pub updated_at: DateTime<Utc>,

    /// Description.
    pub description: String,
}

pub async fn get_id_for_name(name: &str) -> Result<i64, Error> {
    let (sql, values) = Query::select()
        .column(WorkspaceIden::Id)
        .from(WorkspaceIden::Table)
        .and_where(Expr::col(WorkspaceIden::Name).is(name))
        .build_sqlx(SqliteQueryBuilder);

    let row: (i64,) = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;
    Ok(row.0)
}

pub async fn get_by_name(name: &str) -> Result<Workspace, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(WorkspaceIden::Table)
        .and_where(Expr::col(WorkspaceIden::Name).is(name))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn get(id: i64) -> Result<Workspace, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(WorkspaceIden::Table)
        .and_where(Expr::col(WorkspaceIden::Id).is(id))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn create(w: Workspace) -> Result<Workspace, Error> {
    w.validate()?;

    let (sql, values) = Query::insert()
        .into_table(WorkspaceIden::Table)
        .columns([
            WorkspaceIden::Name,
            WorkspaceIden::CreatedAt,
            WorkspaceIden::UpdatedAt,
            WorkspaceIden::Description,
        ])
        .values_panic([
            w.name.into(),
            Utc::now().into(),
            Utc::now().into(),
            w.description.into(),
        ])
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let w: Workspace = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(workspace_name = w.name, "workspace created");

    Ok(w)
}

pub async fn update(w: Workspace) -> Result<Workspace, Error> {
    w.validate()?;

    let (sql, values) = Query::update()
        .table(WorkspaceIden::Table)
        .values([
            (WorkspaceIden::UpdatedAt, Utc::now().into()),
            (WorkspaceIden::Description, w.description.into()),
        ])
        .and_where(Expr::col(WorkspaceIden::Id).is(w.id))
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let w: Workspace = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(workspace_name = w.name, "workspace updated");

    Ok(w)
}

pub async fn delete(id: i64) -> Result<(), Error> {
    let (sql, values) = Query::delete()
        .from_table(WorkspaceIden::Table)
        .and_where(Expr::col(WorkspaceIden::Id).is(id))
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let w: Workspace = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(workspace_name = w.name, "workspace deleted");

    Ok(())
}

pub async fn get_count() -> Result<i64, Error> {
    let (sql, values) = Query::select()
        .from(WorkspaceIden::Table)
        .expr(Expr::count(Expr::cust("*")))
        .build_sqlx(SqliteQueryBuilder);

    let row: (i64,) = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    Ok(row.0)
}

pub async fn list(limit: i64, offset: i64) -> Result<Vec<Workspace>, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(WorkspaceIden::Table)
        .order_by(WorkspaceIden::Name, Order::Asc)
        .limit(limit as u64)
        .offset(offset as u64)
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_ref()), values)
        .fetch_all(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::test::prepare;

    pub async fn create_workspace() -> Workspace {
        create(Workspace {
            name: "test-workspace".into(),
            ..Default::default()
        })
        .await
        .unwrap()
    }

    #[test]
    fn test_validation() {
        assert!(Workspace::default().validate().is_err());
        assert!(
            Workspace {
                name: "foo-bar-123".into(),
                ..Default::default()
            }
            .validate()
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_storage() {
        let _guard = prepare().await;

        let w = Workspace {
            name: "test-workspace".into(),
            description: "Test workspace".into(),
            ..Default::default()
        };

        let w = create(w).await.unwrap();

        let w_get = get(w.id).await.unwrap();
        assert_eq!(w, w_get);

        let w_get = get_by_name(&w.name).await.unwrap();
        assert_eq!(w, w_get);

        assert_eq!(1, get_count().await.unwrap());
        assert_eq!(vec![w.clone()], list(10, 0).await.unwrap());

        let mut w_update = w.clone();
        w_update.description = "New description".into();
        let _ = update(w_update.clone()).await.unwrap();
        let w_get = get(w_update.id).await.unwrap();
        assert_eq!(&w_update.description, &w_get.description);

        delete(w.id).await.unwrap();
        assert!(delete(w.id).await.is_err());
    }
}
