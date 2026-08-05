use std::sync::LazyLock;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use serde::Serialize;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::config;
use crate::profile::ComponentSchema;
use crate::storage::{device, workspace};

mod handlers;
mod home_assistant;
pub mod internal;

static GLOBAL_INTEGRATIONS: LazyLock<RwLock<Vec<Box<dyn Integration + Sync + Send>>>> =
    LazyLock::new(|| RwLock::new(vec![]));

pub async fn setup() -> Result<()> {
    info!("setting up integrations");
    let conf = config::get();
    let mut ints = GLOBAL_INTEGRATIONS.write().await;

    // internal integration used for websockets
    ints.push(Box::new(internal::Integration {}));

    if conf.integration.home_assistant.enabled {
        info!(mqtt_server = %conf.integration.home_assistant.mqtt_server, "setting up Home Assistant integration");
        let ha_int = home_assistant::Integration::new(&conf.integration.home_assistant).await?;
        ints.push(Box::new(ha_int));
    }

    Ok(())
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub device_name: String,
    pub device_short_id: String,
    pub workspace_name: String,
    pub vendor_name: String,
}

impl DeviceInfo {
    pub fn new(w: &workspace::Workspace, d: &device::Device) -> Self {
        DeviceInfo {
            device_name: d.name.clone(),
            device_short_id: d.short_id.to_string(),
            workspace_name: w.name.clone(),
            vendor_name: d.vendor_name.clone(),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct KeyExchangeEvent {
    pub timestamp: DateTime<Utc>,
    pub workspace_name: String,
    pub device_short_id: String,
    pub device_public_key: String,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct ActivationEvent {
    pub timestamp: DateTime<Utc>,
    pub device_info: DeviceInfo,
    pub telemetry_schema: ComponentSchema,
    pub state_schema: ComponentSchema,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct TelemetryEvent {
    pub timestamp: DateTime<Utc>,
    pub device_info: DeviceInfo,
    pub telemetry: serde_json::Value,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct StateEvent {
    pub timestamp: DateTime<Utc>,
    pub device_info: DeviceInfo,
    pub state: serde_json::Value,
}

#[async_trait]
pub trait Integration {
    async fn key_exchange(&self, pl: &KeyExchangeEvent) -> Result<()>;
    async fn activation(&self, pl: &ActivationEvent) -> Result<()>;
    async fn telemetry(&self, pl: &TelemetryEvent) -> Result<()>;
    async fn state(&self, pl: &StateEvent) -> Result<()>;
}

pub async fn key_exchange_event(pl: &KeyExchangeEvent) {
    tokio::spawn({
        let pl = pl.clone();

        async move {
            if let Err(e) = _key_exchange_event(&pl).await {
                warn!(device_short_id = %pl.device_short_id, error = %e, "key-exchange integration event error");
            }
        }
    });
}

async fn _key_exchange_event(pl: &KeyExchangeEvent) -> Result<()> {
    let ints = GLOBAL_INTEGRATIONS.read().await;
    let mut futures = Vec::new();

    for i in ints.iter() {
        futures.push(i.key_exchange(pl));
    }

    for e in join_all(futures).await {
        e?;
    }

    Ok(())
}

pub async fn activation_event(pl: &ActivationEvent) {
    tokio::spawn({
        let pl = pl.clone();

        async move {
            if let Err(e) = _activation_event(&pl).await {
                warn!(device_name = %pl.device_info.device_name, device_short_id = %pl.device_info.device_short_id, error = %e, "activation integration event error");
            }
        }
    });
}

async fn _activation_event(pl: &ActivationEvent) -> Result<()> {
    let ints = GLOBAL_INTEGRATIONS.read().await;
    let mut futures = Vec::new();

    for i in ints.iter() {
        futures.push(i.activation(pl));
    }

    for e in join_all(futures).await {
        e?;
    }

    Ok(())
}

pub async fn telemetry_event(pl: &TelemetryEvent) {
    tokio::spawn({
        let pl = pl.clone();

        async move {
            if let Err(e) = _telemetry_event(&pl).await {
                warn!(device_name = %pl.device_info.device_name, device_short_id = %pl.device_info.device_short_id, error = %e, "telemetry integration event error");
            }
        }
    });
}

async fn _telemetry_event(pl: &TelemetryEvent) -> Result<()> {
    let ints = GLOBAL_INTEGRATIONS.read().await;
    let mut futures = Vec::new();

    for i in ints.iter() {
        futures.push(i.telemetry(pl));
    }

    for e in join_all(futures).await {
        e?;
    }

    Ok(())
}

pub async fn state_event(pl: &StateEvent) {
    tokio::spawn({
        let pl = pl.clone();

        async move {
            if let Err(e) = _state_event(&pl).await {
                warn!(device_name = %pl.device_info.device_name, device_short_id = %pl.device_info.device_short_id, error = %e, "state integration event error");
            }
        }
    });
}

async fn _state_event(pl: &StateEvent) -> Result<()> {
    let ints = GLOBAL_INTEGRATIONS.read().await;
    let mut futures = Vec::new();

    for i in ints.iter() {
        futures.push(i.state(pl));
    }

    for e in join_all(futures).await {
        e?;
    }

    Ok(())
}
