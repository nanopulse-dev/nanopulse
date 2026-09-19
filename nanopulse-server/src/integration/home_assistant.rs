use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use rumqttc::mqttbytes::v5::{ConnectReturnCode, Publish, SubscribeReasonCode};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, PublishOptions, mqttbytes::QoS};
use serde::Serialize;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info, trace};

use super::{
    ActivationEvent, Integration as IntegrationTrait, KeyExchangeEvent, StateEvent, TelemetryEvent,
    handlers::set_device_state_desired,
};
use crate::config;
use crate::profile::{Component, ComponentType, LocationSourceType, SensorType, TemperatureUnit};

pub struct Integration {
    client: AsyncClient,
    qos: QoS,
    ha_discovery_prefix: String,
}

impl Integration {
    pub async fn new(conf: &config::HomeAssistant) -> Result<Integration> {
        info!("initializing Home Assistant integration");

        let client_id = format!("nanopulse-{:x}", getrandom::u32()?);
        let qos = QoS::AtMostOnce;
        let (subscribe_tx, mut subscribe_rx) = mpsc::channel(2);

        let mut mqtt_opts =
            MqttOptions::parse_url(format!("{}?client_id={}", conf.mqtt_server, client_id))?;
        mqtt_opts.set_clean_start(true);
        mqtt_opts.set_keep_alive(15);
        if !conf.mqtt_username.is_empty() || !conf.mqtt_password.is_empty() {
            mqtt_opts.set_credentials(&conf.mqtt_username, conf.mqtt_password.clone());
        }

        let (client, mut eventloop) = AsyncClient::builder(mqtt_opts).capacity(100).build();

        let i = Integration {
            client,
            qos,
            ha_discovery_prefix: "homeassistant".into(),
        };

        info!(server_uri = %conf.mqtt_server, client_id = %client_id, "connecting to MQTT broker");

        // (re)subscribe loop
        tokio::spawn({
            let client = i.client.clone();
            let qos = i.qos;
            let topic = "nanopulse/workspace/+/device/+/command/+".to_string();

            async move {
                while let Some(()) = subscribe_rx.recv().await {
                    loop {
                        info!(topic = %topic, "subscribing to command topic");
                        if let Err(e) = client.subscribe(&topic, qos).await {
                            error!(topic = %topic, error = %e, "subscribe error");
                        } else {
                            break;
                        }
                    }

                    sleep(Duration::from_secs(1)).await;
                }
            }
        });

        // eventloop
        tokio::spawn({
            async move {
                loop {
                    match eventloop.poll().await {
                        Ok(v) => {
                            trace!(event = ?v, "mqtt event");

                            match v {
                                Event::Incoming(Incoming::Publish(p)) => message_callback(p).await,
                                Event::Incoming(Incoming::ConnAck(v)) => {
                                    if v.code == ConnectReturnCode::Success {
                                        if let Err(e) = subscribe_tx.try_send(()) {
                                            error!(error = %e, "send to subscribe channel error");
                                        }
                                    } else {
                                        error!(error = ?v.code, "connection error");
                                        sleep(Duration::from_secs(1)).await
                                    }
                                }
                                Event::Incoming(Incoming::SubAck(v)) => {
                                    let errors: Vec<SubscribeReasonCode> = v
                                        .return_codes
                                        .iter()
                                        .filter(|v| !matches!(v, SubscribeReasonCode::Success(_)))
                                        .cloned()
                                        .collect();

                                    if !errors.is_empty() {
                                        error!(errors = ?errors, "Subscribe ack returned errors");

                                        if let Err(e) = subscribe_tx.try_send(()) {
                                            error!(error = %e, "send to subscribe channel error");
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "mqtt error");
                            sleep(Duration::from_secs(1)).await
                        }
                    }
                }
            }
        });

        Ok(i)
    }

    fn get_discovery_topic(&self, device_name: &str) -> String {
        format!(
            "{}/device/nanopulse/{}/config",
            self.ha_discovery_prefix, device_name
        )
    }

    fn get_telemetry_topic(&self, workspace_name: &str, device_name: &str) -> String {
        format!(
            "nanopulse/workspace/{}/device/{}/event/telemetry",
            workspace_name, device_name
        )
    }
}

#[async_trait]
impl IntegrationTrait for Integration {
    async fn key_exchange(&self, _pl: &KeyExchangeEvent) -> Result<()> {
        Ok(())
    }

    async fn activation(&self, pl: &ActivationEvent) -> Result<()> {
        let state_topic = format!(
            "nanopulse/workspace/{}/device/{}/event/telemetry",
            pl.device_info.workspace_name, pl.device_info.device_name
        );
        let command_topic = format!(
            "nanopulse/workspace/{}/device/{}/command/state",
            pl.device_info.workspace_name, pl.device_info.device_name
        );
        let topic = self.get_discovery_topic(&pl.device_info.device_name);
        let pub_options = PublishOptions::new(QoS::AtLeastOnce);
        info!(device_name = %pl.device_info.device_name, topic = %topic, "sending discovery payload");

        let mut components = components_from_schemas(
            &pl.device_info.device_name,
            &pl.telemetry_schema,
            &state_topic,
        );
        components.extend(components_from_schemas(
            &pl.device_info.device_name,
            &pl.state_schema,
            &state_topic,
        ));

        let pl = DiscoveryPayload {
            components,
            state_topic,
            command_topic,
            device: DiscoveryPayloadDevice {
                ids: pl.device_info.device_name.clone(),
                name: pl.device_info.device_name.clone(),
                manufacturer: pl.device_info.vendor_name.clone(),
            },
            origin: Default::default(),
        };

        self.client
            .publish(topic, serde_json::to_vec(&pl)?, pub_options)
            .await?;

        Ok(())
    }

    async fn telemetry(&self, pl: &TelemetryEvent) -> Result<()> {
        let topic =
            self.get_telemetry_topic(&pl.device_info.workspace_name, &pl.device_info.device_name);
        let pub_options = PublishOptions::new(QoS::AtLeastOnce);
        info!(device_name = %pl.device_info.device_name, topic = %topic, "sending telemetry payload");

        self.client
            .publish(topic, serde_json::to_vec(&pl.telemetry)?, pub_options)
            .await?;

        Ok(())
    }

    async fn state(&self, pl: &StateEvent) -> Result<()> {
        let topic =
            self.get_telemetry_topic(&pl.device_info.workspace_name, &pl.device_info.device_name);
        let pub_options = PublishOptions::new(QoS::AtLeastOnce);
        info!(device_name = %pl.device_info.device_name, topic = %topic, "sending state payload");

        self.client
            .publish(
                topic,
                serde_json::to_vec(&bools_to_strings(&pl.state))?,
                pub_options,
            )
            .await?;
        Ok(())
    }
}

#[derive(Serialize)]
struct DiscoveryPayload {
    pub device: DiscoveryPayloadDevice,
    pub origin: DiscoveryPayloadOrigin,
    pub components: HashMap<String, DiscoveryComponent>,
    pub state_topic: String,
    pub command_topic: String,
}

#[derive(Serialize)]
struct DiscoveryPayloadDevice {
    pub ids: String,
    pub name: String,
    pub manufacturer: String,
}

#[derive(Serialize)]
struct DiscoveryPayloadOrigin {
    pub name: String,
    pub sw: String,
    pub url: String,
}

impl Default for DiscoveryPayloadOrigin {
    fn default() -> Self {
        DiscoveryPayloadOrigin {
            name: env!("CARGO_PKG_NAME").into(),
            sw: env!("CARGO_PKG_VERSION").into(),
            url: env!("CARGO_PKG_HOMEPAGE").into(),
        }
    }
}

#[derive(Default, Serialize)]
#[serde(default)]
struct DiscoveryComponent {
    pub platform: String,
    pub unique_id: String,
    pub value_template: String,
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub device_class: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub unit_of_measurement: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub command_template: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub source_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub json_attributes_topic: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub json_attributes_template: String,
}

fn components_from_schemas(
    device_name: &str,
    schema: &HashMap<String, Component>,
    state_topic: &str,
) -> HashMap<String, DiscoveryComponent> {
    let mut out = HashMap::new();

    for (component_name, component) in schema.iter() {
        let mut dc = DiscoveryComponent {
            unique_id: format!("{}_{}", device_name, component_name),
            name: component.name.clone(),
            ..Default::default()
        };

        match &component.component {
            ComponentType::Sensor(sensor_type) => {
                dc.platform = "sensor".into();
                dc.value_template = format!("{{{{value_json.{}}}}}", component_name);

                match sensor_type {
                    SensorType::None => {
                        dc.device_class = "None".into();
                    }
                    SensorType::Temperature(props) => {
                        dc.device_class = "temperature".into();
                        dc.unit_of_measurement = match props.unit {
                            TemperatureUnit::C => "°C".into(),
                            TemperatureUnit::F => "°F".into(),
                        };
                    }
                }
            }
            ComponentType::Tracker(props) => {
                dc.platform = "device_tracker".into();
                dc.json_attributes_topic = state_topic.to_string();
                dc.json_attributes_template = "{{ {'latitude': value_json.location.latitude, 'longitude': value_json.location.longitude, 'gps_accuracy': value_json.location.accuracy} | tojson }}".to_string();
                dc.source_type = match props.source_type {
                    LocationSourceType::Wifi => "gps",
                }
                .into();
            }
            ComponentType::Switch(_) => {
                dc.platform = "switch".into();
                dc.value_template = format!("{{{{value_json.{}}}}}", component_name);
                dc.command_template = format!("{{\"{}\": \"{{{{value}}}}\"}}", component_name);
            }
        }

        out.insert(component_name.into(), dc);
    }

    out
}

fn bools_to_strings(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Bool(true) => serde_json::Value::String("ON".to_string()),
        serde_json::Value::Bool(false) => serde_json::Value::String("OFF".to_string()),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(bools_to_strings).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in obj {
                new_map.insert(k.to_string(), bools_to_strings(v));
            }
            serde_json::Value::Object(new_map)
        }
        _ => value.clone(),
    }
}

fn strings_to_bools(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(v) => match v.as_ref() {
            "ON" => serde_json::Value::Bool(true),
            "OFF" => serde_json::Value::Bool(false),
            _ => serde_json::Value::String(v),
        },
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(strings_to_bools).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in obj {
                new_map.insert(k, strings_to_bools(v));
            }
            serde_json::Value::Object(new_map)
        }
        _ => value,
    }
}

async fn message_callback(p: Publish) {
    let topic = String::from_utf8_lossy(&p.topic);
    let parts: Vec<_> = topic.split("/").collect();

    let err = || -> Result<()> {
        if parts.len() != 7 {
            return Err(anyhow!("invalid topic expected 7 parts, topic: {}", topic));
        }
        if topic.ends_with("/state") {
            let v: serde_json::Value = serde_json::from_slice(p.payload.as_ref())?;
            let v = strings_to_bools(v);
            tokio::spawn(set_device_state_desired(parts[4].to_string(), v));
        }

        Ok(())
    }();

    if let Err(e) = err {
        error!(error = %e, topic = %topic, "handle message error");
    }
}
