use std::str::FromStr;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use regex::Regex;
use sqlx::Sqlite;
use sqlx::migrate::Migrator;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use tokio::sync::RwLock;
use tracing::info;

use crate::config;

pub mod device;
pub mod fields;
pub mod gateway;
pub mod workspace;

static DATABASE_POOL: LazyLock<RwLock<Option<SqlitePool>>> = LazyLock::new(|| RwLock::new(None));
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9\-]{3,100}$").unwrap());

pub async fn setup() -> Result<()> {
    let conf = config::get();
    info!(database_path = %conf.database.path, "Setting up storage");
    let opts = SqliteConnectOptions::from_str(&conf.database.path)?
        .create_if_missing(true)
        .pragma("journal_mode", "WAL")
        .pragma("busy_timeout", "1000")
        .pragma("foreign_keys", "on");
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(opts)
        .await?;

    info!("Running database migrations");
    MIGRATOR.run(&pool).await?;

    let mut pool_w = DATABASE_POOL.write().await;
    *pool_w = Some(pool);

    Ok(())
}

pub async fn get_conn() -> Result<PoolConnection<Sqlite>> {
    Ok(DATABASE_POOL
        .read()
        .await
        .as_ref()
        .ok_or_else(|| anyhow!("Database pool is not setup"))?
        .acquire()
        .await?)
}

pub fn json_value(v: &serde_json::Value) -> sea_query::Value {
    sea_query::Value::Json(Some(Box::new(v.clone())))
}
