use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use tracing::info;

use super::{
    ActivationEvent, Integration as IntegrationTrait, KeyExchangeEvent, StateEvent, TelemetryEvent,
};
use crate::errors::Error;

static CHANNELS: LazyLock<RwLock<HashMap<String, broadcast::Sender<Event>>>> =
    LazyLock::new(|| RwLock::new(HashMap::default()));

#[derive(Deserialize)]
pub struct Subscribe {
    pub workspace_name: String,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Event {
    pub log: Option<Log>,
    pub key_exchange_event: Option<KeyExchangeEvent>,
    pub activation_event: Option<ActivationEvent>,
    pub state_event: Option<StateEvent>,
    pub telemetry_event: Option<TelemetryEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Log {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub attributes: HashMap<String, String>,
    pub message: String,
}

pub struct Integration;

#[async_trait]
impl IntegrationTrait for Integration {
    async fn key_exchange(&self, pl: &KeyExchangeEvent) -> Result<()> {
        publish(
            &pl.workspace_name,
            Event {
                key_exchange_event: Some(pl.clone()),
                ..Default::default()
            },
        );
        Ok(())
    }

    async fn activation(&self, pl: &ActivationEvent) -> Result<()> {
        publish(
            &pl.device_info.workspace_name,
            Event {
                activation_event: Some(pl.clone()),
                ..Default::default()
            },
        );
        Ok(())
    }

    async fn telemetry(&self, pl: &TelemetryEvent) -> Result<()> {
        publish(
            &pl.device_info.workspace_name,
            Event {
                telemetry_event: Some(pl.clone()),
                ..Default::default()
            },
        );
        Ok(())
    }

    async fn state(&self, pl: &StateEvent) -> Result<()> {
        publish(
            &pl.device_info.workspace_name,
            Event {
                state_event: Some(pl.clone()),
                ..Default::default()
            },
        );
        Ok(())
    }
}

pub fn get_channel(workspace_name: &str) -> Result<broadcast::Receiver<Event>, Error> {
    info!(workspace_name = %workspace_name, "get channel for workspace");
    let mut c = CHANNELS.write().unwrap();
    let v = c
        .entry(workspace_name.to_string())
        .or_insert_with(|| broadcast::Sender::<Event>::new(10));

    Ok(v.subscribe())
}

pub fn publish(workspace_name: &str, n: Event) {
    let c = CHANNELS.read().unwrap();
    if let Some(sender) = c.get(workspace_name) {
        _ = sender.send(n);
    }
}
