use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use rumqttc::mqttbytes::QoS;
use rumqttc::mqttbytes::v5::{ConnectReturnCode, Publish};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, PublishOptions};
use tokio::{sync::OnceCell, sync::mpsc, task, time::sleep};
use tracing::{Instrument, Level, error, info, span, trace};

use nanopulse_structs::common::CodingRate;
use nanopulse_structs::gateway::{
    ChannelConfiguration, ConfigCommand, DataRateConfiguration, FskDataRateConfiguration,
    HeartbeatEvent, LoraDataRateConfiguration, RxEvent, TxCommand,
};

use crate::config;
use crate::flow;
use crate::region::{Modulation, Region};

static CLIENT: OnceCell<AsyncClient> = OnceCell::const_new();

pub async fn setup() -> Result<()> {
    info!("Setting up gw backend");

    let conf = config::get();
    let client_id = if conf.mqtt.client_id.is_empty() {
        format!("nanopulse-{:x}", getrandom::u32()?)
    } else {
        conf.mqtt.client_id.clone()
    };

    let mut opts = MqttOptions::parse_url(format!("{}?client_id={}", conf.mqtt.server, client_id))
        .context("parse mqtt server url")?;
    opts.set_clean_start(true);
    opts.set_keep_alive(15);
    if !conf.mqtt.username.is_empty() || !conf.mqtt.password.is_empty() {
        opts.set_credentials(&conf.mqtt.username, conf.mqtt.password.clone());
    }

    let (client, mut eventloop) = AsyncClient::builder(opts).capacity(100).build();
    CLIENT.set(client.clone())?;

    // Create connect channel
    // We need to re-subscribe on (re)connect to be sure we have a subscription. Even
    // in case of a persistent MQTT session, there is no guarantee that the MQTT persisted the
    // session and that a re-connect would recover the subscription.
    let (connect_tx, mut connect_rx) = mpsc::channel(10);

    task::spawn({
        let subscribe_topic = "nanopulse/gateway/+/event/+".to_string();

        async move {
            while let Some(shared_sub_support) = connect_rx.recv().await {
                let subscribe_topic = if shared_sub_support {
                    format!("$share/nanopulse/{}", subscribe_topic)
                } else {
                    subscribe_topic.clone()
                };

                info!(topic = %subscribe_topic, "Subscribing to gateway topic");
                if let Err(e) = client.subscribe(&subscribe_topic, QoS::AtLeastOnce).await {
                    error!(topic = %subscribe_topic, error = %e, "Subscribe error");
                }
            }
        }
    });

    task::spawn(async move {
        info!("Starting event loop");
        loop {
            match eventloop.poll().await {
                Ok(v) => {
                    trace!(event = ?v, "MQTT event");

                    match v {
                        Event::Incoming(Incoming::Publish(p)) => {
                            task::spawn(async {
                                if let Err(e) = message_callback(p).await {
                                    error!(error = %e, "Error handling message");
                                }
                            });
                        }
                        Event::Incoming(Incoming::ConnAck(v)) => {
                            if v.code == ConnectReturnCode::Success {
                                // Per specification:
                                // A value of 1 means Shared Subscriptions are supported. If not present, then Shared Subscriptions are supported.
                                let shared_sub_support = v
                                    .properties
                                    .map(|v| {
                                        v.shared_subscription_available
                                            .map(|v| v == 1)
                                            .unwrap_or(true)
                                    })
                                    .unwrap_or(true);

                                if let Err(e) = connect_tx.try_send(shared_sub_support) {
                                    error!(error = %e, "Send to subscribe channel error");
                                }
                            } else {
                                error!(code = ?v.code, "Connection error");
                                sleep(Duration::from_secs(1)).await
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    error!(error = %e, "MQTT error");
                    sleep(Duration::from_secs(1)).await
                }
            }
        }
    });

    Ok(())
}

async fn message_callback(p: Publish) -> Result<()> {
    let topic = String::from_utf8_lossy(&p.topic);
    let gateway_name = get_gateway_gid(&topic)?;

    if topic.ends_with("event/rx") {
        let rx_pl: RxEvent = minicbor::decode(p.payload.as_ref())?;

        let span = span!(Level::INFO, "rx", gateway_name = %gateway_name, rx_id = rx_pl.rx_id);
        flow::rx::deduplicate_and_handle(&gateway_name, &rx_pl)
            .instrument(span)
            .await?;
    } else if topic.ends_with("event/heartbeat") {
        let hb_pl: HeartbeatEvent = minicbor::decode(p.payload.as_ref())?;
        let span = span!(Level::INFO, "heartbeat", gateway_name = %gateway_name);

        flow::heartbeat::handle(&gateway_name, hb_pl)
            .instrument(span)
            .await;
    }

    Ok(())
}

fn get_gateway_gid(topic: &str) -> Result<String> {
    let parts: Vec<&str> = topic.split("/").collect();
    if parts.len() != 5 {
        return Err(anyhow!("invalid topic"));
    }

    Ok(parts[2].to_lowercase())
}

pub async fn send_config_command(gateway_name: &str, pl: &ConfigCommand) -> Result<()> {
    let topic = format!("nanopulse/gateway/{}/command/config", gateway_name);
    let pub_options = PublishOptions::new(QoS::AtLeastOnce);
    info!(topic = %topic, "Sending config command");
    let client = CLIENT.get().ok_or_else(|| anyhow!("CLIENT not set"))?;
    client
        .publish(topic, minicbor::to_vec(pl)?, pub_options)
        .await?;

    Ok(())
}

pub async fn send_tx_command(gateway_name: &str, pl: &TxCommand) -> Result<()> {
    let topic = format!("nanopulse/gateway/{}/command/tx", gateway_name);
    let pub_options = PublishOptions::new(QoS::AtLeastOnce);
    info!(topic = %topic, "Sending tx command");
    let client = CLIENT.get().ok_or_else(|| anyhow!("CLIENT not set"))?;
    client
        .publish(topic, minicbor::to_vec(pl)?, pub_options)
        .await?;

    Ok(())
}

pub async fn sync_configuration(gateway_name: &str, region: Arc<Region>) -> Result<()> {
    info!(gateway_name = %gateway_name, "syncing gateway configuration");

    let config_cmd = ConfigCommand {
        version: region.version.clone(),
        uplink_channels: region
            .uplink_channels
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    ChannelConfiguration {
                        freq_hz: v.frequency,
                        data_rates: v.data_rates.clone(),
                    },
                )
            })
            .collect(),
        downlink_channels: region
            .downlink_channels
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    ChannelConfiguration {
                        freq_hz: v.frequency,
                        data_rates: v.data_rates.clone(),
                    },
                )
            })
            .collect(),
        data_rates: region
            .data_rates
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    match v.modulation {
                        Modulation::Lora => {
                            DataRateConfiguration::Lora(LoraDataRateConfiguration {
                                bandwidth: v.bandwidth,
                                spreading_factor: v.spreading_factor,
                                coding_rate: Into::<CodingRate>::into(v.coding_rate),
                            })
                        }
                        Modulation::Fsk => DataRateConfiguration::Fsk(FskDataRateConfiguration {
                            frequency_deviation: v.frequency_deviation,
                            bitrate: v.bitrate,
                        }),
                    },
                )
            })
            .collect(),
    };

    send_config_command(gateway_name, &config_cmd).await
}
