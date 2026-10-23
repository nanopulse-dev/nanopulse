use std::fmt::Display;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_query::{Asterisk, Expr, ExprTrait, Order, Query, SqliteQueryBuilder, enum_def};
use sea_query_sqlx::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::{info, warn};
use utoipa::ToSchema;
use validator::Validate;

use crate::errors::Error;
use crate::storage::{NAME_REGEX, fields, get_conn, json_value};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow, Validate, PartialEq, Eq)]
#[enum_def]
#[serde(default)]
pub struct Device {
    #[serde(skip)]
    pub id: i64,

    #[serde(skip)]
    pub workspace_id: i64,

    /// Name.
    /// Valid input is a-z, 0-9 and -, e.g. test-device-123.
    #[validate(regex(path = *NAME_REGEX))]
    pub name: String,

    /// Public key of the device.
    pub public_key: fields::HexArray<32>,

    /// Short ID of the device.
    pub short_id: fields::HexArray<4>,

    /// Created-at timestamp.
    pub created_at: DateTime<Utc>,

    /// Updated-at timestamp.
    pub updated_at: DateTime<Utc>,

    /// Last key-exchange timestamp.
    pub key_exchange_at: Option<DateTime<Utc>>,

    /// Last activation at timestamp.
    pub activation_at: Option<DateTime<Utc>>,

    /// Last seen at timestamp.
    pub last_seen_at: Option<DateTime<Utc>>,

    /// Device PIN.
    pub pin: fields::HexArray<4>,

    /// Description of the device.
    pub description: String,

    /// Vendor ID.
    pub vendor_id: fields::HexArray<4>,

    /// Profile ID.
    pub profile_id: fields::HexArray<2>,

    /// Version ID.
    pub version_id: fields::HexArray<2>,

    /// Vendor name.
    pub vendor_name: String,

    /// Last reported telemetry.
    pub telemetry: serde_json::Value,

    /// Device state (current).
    pub state: serde_json::Value,

    /// Device state (desired).
    pub state_desired: serde_json::Value,

    /// Device configuration (current).
    pub configuration: serde_json::Value,

    /// Device configuration (desired).
    pub configuration_desired: serde_json::Value,

    /// MAC configuration of the device (current).
    pub mac_configuration: serde_json::Value,

    /// MAC configuration of the device (desired).
    pub mac_configuration_desired: serde_json::Value,

    #[serde(skip)]
    pub root_key: fields::HexArray<32>,

    #[serde(skip)]
    pub session_root_key: fields::HexArray<32>,

    #[serde(skip)]
    pub key_exchange_resp_counter: u32,
    #[serde(skip)]
    pub activation_req_counter: u32,
    #[serde(skip)]
    pub activation_resp_counter: u32,
    #[serde(skip)]
    pub counter_up: u32,
    #[serde(skip)]
    pub counter_down: u32,
}

pub enum Counter {
    ActivationRequestCounter(u32),
    CounterUp(u32),
}

impl Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Counter::ActivationRequestCounter(v) => write!(f, "activation_req_counter: {}", v),
            Counter::CounterUp(v) => write!(f, "counter_up: {}", v),
        }
    }
}

impl Device {
    pub fn lua_profile_module(&self) -> String {
        format!(
            "vendors.{}.profiles.{}_{}",
            self.vendor_id, self.profile_id, self.version_id
        )
    }

    pub fn needs_state_sync(&self) -> bool {
        let state = match self.state.as_object() {
            Some(v) => v,
            None => return false,
        };

        let state_desired = match self.state_desired.as_object() {
            Some(v) => v,
            None => return false,
        };

        !state_desired.iter().all(|(k, v)| state.get(k) == Some(v))
    }

    pub fn needs_configuration_sync(&self) -> bool {
        let configuration = match self.configuration.as_object() {
            Some(v) => v,
            None => return false,
        };

        let configuration_desired = match self.configuration_desired.as_object() {
            Some(v) => v,
            None => return false,
        };

        !configuration_desired
            .iter()
            .all(|(k, v)| configuration.get(k) == Some(v))
    }
}

impl Default for Device {
    fn default() -> Self {
        let now = Utc::now();

        Device {
            id: 0,
            workspace_id: 0,
            name: "".into(),
            public_key: fields::HexArray::default(),
            short_id: fields::HexArray::default(),
            created_at: now,
            updated_at: now,
            key_exchange_at: None,
            activation_at: None,
            last_seen_at: None,
            pin: fields::HexArray::default(),
            description: "".into(),
            vendor_id: fields::HexArray::default(),
            profile_id: fields::HexArray::default(),
            version_id: fields::HexArray::default(),
            vendor_name: "".into(),
            telemetry: serde_json::Value::default(),
            state: serde_json::Value::default(),
            state_desired: serde_json::Value::default(),
            configuration: serde_json::Value::default(),
            configuration_desired: serde_json::Value::default(),
            mac_configuration: serde_json::Value::default(),
            mac_configuration_desired: serde_json::Value::default(),
            root_key: fields::HexArray::default(),
            session_root_key: fields::HexArray::default(),
            key_exchange_resp_counter: 0,
            activation_req_counter: 0,
            activation_resp_counter: 0,
            counter_up: 0,
            counter_down: 0,
        }
    }
}

#[derive(Default)]
pub struct DeviceChangeSet {
    pub pin: Option<fields::HexArray<4>>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub root_key: Option<fields::HexArray<32>>,
    pub session_root_key: Option<fields::HexArray<32>>,
    pub key_exchange_at: Option<DateTime<Utc>>,
    pub activation_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub vendor_id: Option<fields::HexArray<4>>,
    pub profile_id: Option<fields::HexArray<2>>,
    pub version_id: Option<fields::HexArray<2>>,
    pub vendor_name: Option<String>,
    pub telemetry: Option<serde_json::Value>,
    pub state: Option<serde_json::Value>,
    pub state_desired: Option<serde_json::Value>,
    pub configuration: Option<serde_json::Value>,
    pub configuration_desired: Option<serde_json::Value>,
    pub key_exchange_resp_counter: Option<u32>,
    pub activation_req_counter: Option<u32>,
    pub activation_resp_counter: Option<u32>,
    pub counter_up: Option<u32>,
    pub counter_down: Option<u32>,
}

#[derive(Debug, FromRow)]
#[enum_def]
pub struct DeviceKeyExchange {
    pub id: i64,
    pub device_id: i64,
    pub created_at: DateTime<Utc>,
    pub request_nonce: i64,
    pub response_nonce: i64,
    pub root_key: fields::HexArray<32>,
}

pub async fn get(id: i64) -> Result<Device, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(DeviceIden::Table)
        .and_where(Expr::col(DeviceIden::Id).is(id))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn get_by_public_key(public_key: &[u8; 32]) -> Result<Device, Error> {
    let public_key = public_key.as_slice();

    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(DeviceIden::Table)
        .and_where(Expr::col(DeviceIden::PublicKey).is(public_key))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn get_by_name(name: &str) -> Result<Device, Error> {
    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(DeviceIden::Table)
        .and_where(Expr::col(DeviceIden::Name).is(name))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn get_by_short_id(short_id: &[u8; 4]) -> Result<Vec<Device>, Error> {
    let short_id = short_id.as_slice();

    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(DeviceIden::Table)
        .and_where(Expr::col(DeviceIden::ShortId).is(short_id))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_all(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn create(d: Device) -> Result<Device, Error> {
    let (sql, values) = Query::insert()
        .into_table(DeviceIden::Table)
        .columns([
            DeviceIden::WorkspaceId,
            DeviceIden::Name,
            DeviceIden::PublicKey,
            DeviceIden::ShortId,
            DeviceIden::CreatedAt,
            DeviceIden::UpdatedAt,
            DeviceIden::Pin,
            DeviceIden::Description,
            DeviceIden::VendorName,
            DeviceIden::KeyExchangeRespCounter,
            DeviceIden::ActivationReqCounter,
            DeviceIden::ActivationRespCounter,
            DeviceIden::CounterUp,
            DeviceIden::CounterDown,
        ])
        .values_panic([
            d.workspace_id.into(),
            d.name.into(),
            d.public_key.as_vec().into(),
            d.short_id.as_vec().into(),
            d.created_at.into(),
            d.updated_at.into(),
            d.pin.as_vec().into(),
            d.description.into(),
            d.vendor_name.into(),
            d.key_exchange_resp_counter.into(),
            d.activation_req_counter.into(),
            d.activation_resp_counter.into(),
            d.counter_up.into(),
            d.counter_down.into(),
        ])
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let d: Device = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(device_name = d.name, "device created");

    Ok(d)
}

pub async fn update(id: i64, changes: &DeviceChangeSet) -> Result<Device, Error> {
    let mut values = vec![];

    if let Some(v) = &changes.pin {
        values.push((DeviceIden::Pin, v.as_vec().into()));
    }
    if let Some(v) = &changes.name {
        values.push((DeviceIden::Name, v.into()));
    }
    if let Some(v) = &changes.description {
        values.push((DeviceIden::Description, v.into()));
    }
    if let Some(v) = &changes.root_key {
        values.push((DeviceIden::RootKey, v.as_vec().into()));
    }
    if let Some(v) = &changes.session_root_key {
        values.push((DeviceIden::SessionRootKey, v.as_vec().into()));
    }
    if let Some(v) = &changes.key_exchange_at {
        values.push((DeviceIden::KeyExchangeAt, (*v).into()));
    }
    if let Some(v) = &changes.activation_at {
        values.push((DeviceIden::ActivationAt, (*v).into()));
    }
    if let Some(v) = &changes.last_seen_at {
        values.push((DeviceIden::LastSeenAt, (*v).into()));
    }
    if let Some(v) = &changes.vendor_id {
        values.push((DeviceIden::VendorId, v.as_vec().into()));
    }
    if let Some(v) = &changes.vendor_name {
        values.push((DeviceIden::VendorName, v.into()));
    }
    if let Some(v) = &changes.profile_id {
        values.push((DeviceIden::ProfileId, v.as_vec().into()));
    }
    if let Some(v) = &changes.version_id {
        values.push((DeviceIden::VersionId, v.as_vec().into()));
    }
    if let Some(v) = &changes.telemetry {
        values.push((DeviceIden::Telemetry, json_value(v).into()));
    }
    if let Some(v) = &changes.state {
        values.push((DeviceIden::State, json_value(v).into()));
    }
    if let Some(v) = &changes.state_desired {
        values.push((DeviceIden::StateDesired, json_value(v).into()));
    }
    if let Some(v) = &changes.configuration {
        values.push((DeviceIden::Configuration, json_value(v).into()));
    }
    if let Some(v) = &changes.configuration_desired {
        values.push((DeviceIden::ConfigurationDesired, json_value(v).into()));
    }
    if let Some(v) = &changes.key_exchange_resp_counter {
        values.push((DeviceIden::KeyExchangeRespCounter, v.into()));
    }
    if let Some(v) = &changes.activation_req_counter {
        values.push((DeviceIden::ActivationReqCounter, v.into()));
    }
    if let Some(v) = &changes.activation_resp_counter {
        values.push((DeviceIden::ActivationRespCounter, v.into()));
    }
    if let Some(v) = &changes.counter_up {
        values.push((DeviceIden::CounterUp, v.into()));
    }
    if let Some(v) = &changes.counter_down {
        values.push((DeviceIden::CounterDown, v.into()));
    }

    if values.is_empty() {
        return Err(Error::EmptyUpdate);
    }

    let (sql, values) = Query::update()
        .table(DeviceIden::Table)
        .values(values)
        .and_where(Expr::col(DeviceIden::Id).is(id))
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let d: Device = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(device_name = d.name, "device updated");

    Ok(d)
}

pub async fn delete(id: i64) -> Result<(), Error> {
    let (sql, values) = Query::delete()
        .from_table(DeviceIden::Table)
        .and_where(Expr::col(DeviceIden::Id).is(id))
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let d: Device = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!(device_name = d.name, "device deleted");

    Ok(())
}

pub async fn get_count(workspace_id: i64) -> Result<i64, Error> {
    let (sql, values) = Query::select()
        .from(DeviceIden::Table)
        .expr(Expr::count(Expr::cust("*")))
        .and_where(Expr::col(DeviceIden::WorkspaceId).is(workspace_id))
        .build_sqlx(SqliteQueryBuilder);

    let row: (i64,) = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    Ok(row.0)
}

pub async fn sync_counter(id: i64, counter: Counter) -> Result<Device, Error> {
    let mut q = Query::update()
        .table(DeviceIden::Table)
        .and_where(Expr::col(DeviceIden::Id).eq(id))
        .to_owned();

    match counter {
        Counter::ActivationRequestCounter(v) => {
            q = q
                .value(DeviceIden::ActivationReqCounter, v + 1)
                .and_where(Expr::col(DeviceIden::ActivationReqCounter).lte(v))
                .to_owned();
        }
        Counter::CounterUp(v) => {
            q = q
                .value(DeviceIden::CounterUp, v + 1)
                .and_where(Expr::col(DeviceIden::CounterUp).lte(v))
                .to_owned();
        }
    }

    let (sql, values) = q
        .returning(Query::returning().all())
        .build_sqlx(SqliteQueryBuilder);

    let d: Device = sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_one(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
        .map_err(|e| {
            if let Error::NotFound = e {
                warn!(counter = %counter, "counter value is invalid");
            }

            e
        })?;

    info!(
        device_name = d.name,
        counter = %counter,
        "synchronized counter"
    );

    Ok(d)
}

pub async fn list(workspace_id: i64, limit: i64, offset: i64) -> Result<Vec<Device>, Error> {
    let (sql, values) = Query::select()
        .from(DeviceIden::Table)
        .column(Asterisk)
        .and_where(Expr::col(DeviceIden::WorkspaceId).is(workspace_id))
        .order_by(DeviceIden::Name, Order::Asc)
        .limit(limit as u64)
        .offset(offset as u64)
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_all(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

pub async fn insert_key_exchange(
    device_id: i64,
    req_nonce: i64,
    resp_nonce: i64,
    root_key: &[u8; 32],
) -> Result<(), Error> {
    let (sql, values) = Query::insert()
        .into_table(DeviceKeyExchangeIden::Table)
        .columns([
            DeviceKeyExchangeIden::CreatedAt,
            DeviceKeyExchangeIden::DeviceId,
            DeviceKeyExchangeIden::RequestNonce,
            DeviceKeyExchangeIden::ResponseNonce,
            DeviceKeyExchangeIden::RootKey,
        ])
        .values_panic([
            Utc::now().into(),
            device_id.into(),
            req_nonce.into(),
            resp_nonce.into(),
            root_key.as_ref().into(),
        ])
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .execute(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)?;

    info!("inserted key-exchange");

    Ok(())
}

pub async fn get_key_exchanges(device_id: i64) -> Result<Vec<DeviceKeyExchange>, Error> {
    let (sql, values) = Query::select()
        .from(DeviceKeyExchangeIden::Table)
        .columns([
            DeviceKeyExchangeIden::Id,
            DeviceKeyExchangeIden::CreatedAt,
            DeviceKeyExchangeIden::DeviceId,
            DeviceKeyExchangeIden::RequestNonce,
            DeviceKeyExchangeIden::ResponseNonce,
            DeviceKeyExchangeIden::RootKey,
        ])
        .and_where(Expr::col(DeviceKeyExchangeIden::DeviceId).is(device_id))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with(sqlx::AssertSqlSafe(sql.as_str()), values)
        .fetch_all(&mut *get_conn().await?)
        .await
        .map_err(Error::from_sqlx)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage;
    use crate::test::prepare;

    #[test]
    fn test_validation() {
        assert!(Device::default().validate().is_err());
        assert!(
            Device {
                name: "test-device-123".into(),
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

        let d = Device {
            workspace_id: w.id,
            name: "test-device-123".into(),
            description: "Test device".into(),
            ..Default::default()
        };
        let d = create(d).await.unwrap();

        let d_get = get(d.id).await.unwrap();
        assert_eq!(d, d_get);

        let d_get = get_by_name(&d.name).await.unwrap();
        assert_eq!(d, d_get);

        let d_get = get_by_public_key(d.public_key.as_ref()).await.unwrap();
        assert_eq!(d, d_get);

        let d_get = get_by_short_id(d.short_id.as_ref()).await.unwrap();
        assert_eq!(vec![d.clone()], d_get);

        assert_eq!(1, get_count(w.id).await.unwrap());
        assert_eq!(vec![d.clone()], list(w.id, 10, 0).await.unwrap());

        let _ = update(
            d.id,
            &DeviceChangeSet {
                description: Some("Updated description".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let d_get = get(d.id).await.unwrap();
        assert_eq!("Updated description", &d_get.description);

        assert!(sync_counter(d.id, Counter::CounterUp(0)).await.is_ok());
        assert!(sync_counter(d.id, Counter::CounterUp(1)).await.is_ok());
        assert!(sync_counter(d.id, Counter::CounterUp(1)).await.is_err());

        insert_key_exchange(d.id, 1, 2, &[0u8; 32]).await.unwrap();
        insert_key_exchange(d.id, 2, 3, &[0u8; 32]).await.unwrap();

        let key_exchanges = get_key_exchanges(d.id).await.unwrap();
        assert_eq!(2, key_exchanges.len());

        delete(d.id).await.unwrap();
        assert!(delete(d.id).await.is_err());
    }
}
