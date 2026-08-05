use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use rumqttc::mqttbytes::v5::{ConnectReturnCode, Publish};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, PublishOptions, mqttbytes::QoS};
use tokio::sync::{Mutex, OnceCell, mpsc};
use tokio::time::sleep;
use tracing::{error, info, trace};

use nanopulse_structs::gateway::{
    ConfigCommand, HeartbeatEvent, TxAckEvent, TxAckStatus, TxCommand,
};

use crate::config::Mqtt;
use crate::traits::Radio;

static STATE: OnceCell<Mutex<State>> = OnceCell::const_new();

struct State {
    radio: Box<dyn Radio>,
    mqtt_topic_prefix: String,
    mqtt_client: AsyncClient,
    heartbeat_state: HeartbeatEvent,
}

impl State {
    async fn publish_tx_ack_event(&self, pl: &TxAckEvent) -> Result<()> {
        let tx_ack_topic = format!("{}/event/txack", self.mqtt_topic_prefix);
        let pub_options = PublishOptions::new(QoS::AtLeastOnce);
        let b = minicbor::to_vec(pl)?;

        info!(
            tx_id = pl.id,
            status = ?pl.status,
            topic = %tx_ack_topic,
            bytes_len = b.len(),
            "publishing tx ack event",
        );

        self.mqtt_client
            .publish(&tx_ack_topic, b, pub_options)
            .await?;
        Ok(())
    }
}

pub async fn start(gateway_name: &str, radio: Box<dyn Radio>, mqtt_config: &Mqtt) -> Result<()> {
    info!(gateway_name = %gateway_name, "Initializing gateway");

    let client_id = if mqtt_config.client_id.is_empty() {
        gateway_name.to_string()
    } else {
        mqtt_config.client_id.clone()
    };

    let mqtt_topic_prefix = format!("nanopulse/gateway/{}", gateway_name);

    let mut mqtt_opts =
        MqttOptions::parse_url(format!("{}?client_id={}", mqtt_config.server, client_id))?;
    mqtt_opts.set_clean_start(true);
    mqtt_opts.set_keep_alive(15);
    if !mqtt_config.username.is_empty() || !mqtt_config.password.is_empty() {
        mqtt_opts.set_credentials(&mqtt_config.username, mqtt_config.password.clone());
    }

    info!(server = %mqtt_config.server, "Connecting to MQTT broker");
    let (connect_tx, mut connect_rx) = mpsc::channel::<()>(1);
    let (mqtt_client, mut mqtt_eventloop) =
        AsyncClient::builder(mqtt_opts.clone()).capacity(10).build();

    let state = State {
        radio,
        mqtt_topic_prefix: mqtt_topic_prefix.clone(),
        mqtt_client: mqtt_client.clone(),
        heartbeat_state: HeartbeatEvent::default(),
    };
    STATE
        .set(Mutex::new(state))
        .map_err(|_| anyhow!("set STATE error"))?;

    // (re)subscribe loop
    tokio::spawn({
        let client = mqtt_client.clone();
        let hb_topic = format!("{}/event/heartbeat", mqtt_topic_prefix);
        let subscribe_topic = format!("{}/command/+", mqtt_topic_prefix);
        let pub_options = PublishOptions::new(QoS::AtLeastOnce);

        async move {
            while connect_rx.recv().await.is_some() {
                info!(topic = %subscribe_topic, "subscribing to command topic");
                if let Err(e) = client.subscribe(&subscribe_topic, QoS::AtLeastOnce).await {
                    error!(topic = %subscribe_topic, error = %e, "subscribe error");
                }

                let mut hb_state = STATE.get().unwrap().lock().await.heartbeat_state.clone();
                hb_state.gw_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                let hb_b = match minicbor::to_vec(&hb_state) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(error = %e, "encode error");
                        continue;
                    }
                };
                info!(topic = %hb_topic, bytes_len = hb_b.len(), "publishing heartbeat event");
                if let Err(e) = client.publish(&hb_topic, hb_b, pub_options.clone()).await {
                    error!(topic = %hb_topic, error = %e, "publish error");
                }
            }
        }
    });

    // Eventloop
    tokio::spawn({
        async move {
            info!("starting MQTT event loop");

            loop {
                match mqtt_eventloop.poll().await {
                    Ok(v) => {
                        trace!(event = ?v, "MQTT event");

                        match v {
                            Event::Incoming(Incoming::Publish(p)) => {
                                tokio::task::spawn(async {
                                    if let Err(e) = handle_mqtt_command(p).await {
                                        error!(error = %e, "handle MQTT event error");
                                    }
                                });
                            }
                            Event::Incoming(Incoming::ConnAck(v)) => {
                                if v.code == ConnectReturnCode::Success {
                                    if let Err(e) = connect_tx.try_send(()) {
                                        error!(error = %e, "send to subscribe channel error");
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
        }
    });

    // heartbeat loop
    tokio::spawn({
        let heartbeat_interval = Duration::from_secs(5 * 60);
        let heartbeat_topic = format!("{}/event/heartbeat", mqtt_topic_prefix);
        let pub_options = PublishOptions::new(QoS::AtLeastOnce);

        async move {
            info!("starting heartbeat loop");

            loop {
                sleep(heartbeat_interval).await;
                {
                    let state = STATE.get().unwrap().lock().await;
                    let mut hb_state = state.heartbeat_state.clone();
                    hb_state.gw_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;

                    let b = match minicbor::to_vec(&hb_state) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(error = %e, "encode error");
                            continue;
                        }
                    };
                    info!(topic = %heartbeat_topic, bytes_len = b.len(), "sending heartbeat event");

                    if let Err(e) = state
                        .mqtt_client
                        .publish(&heartbeat_topic, b, pub_options.clone())
                        .await
                    {
                        error!(topic = %heartbeat_topic, error =%e, "publish heartbeat event error");
                    }
                }
            }
        }
    });

    tokio::spawn({
        let rx_topic = format!("{}/event/rx", mqtt_topic_prefix);
        let pub_options = PublishOptions::new(QoS::AtLeastOnce);

        async move {
            info!("starting RX loop");
            loop {
                sleep(Duration::from_millis(10)).await;
                {
                    let state = STATE.get().unwrap().lock().await;
                    match state.radio.receive() {
                        Ok(rx) => {
                            for v in rx {
                                let b = match minicbor::to_vec(&v) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        error!(rx_id = v.rx_id, error = %e, "encode error");
                                        continue;
                                    }
                                };
                                info!(topic = %rx_topic, rx_id = v.rx_id, bytes_len = b.len(), "publising RX event");

                                if let Err(e) = state
                                    .mqtt_client
                                    .publish(&rx_topic, b, pub_options.clone())
                                    .await
                                {
                                    error!(topic = %rx_topic, error = %e, "Publish RX event error");
                                }
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Receive error");
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

async fn handle_mqtt_command(p: Publish) -> Result<()> {
    let topic = String::from_utf8(p.topic.to_vec())?;
    let qos = p.qos;
    let b = p.payload.to_vec();

    info!(topic = %topic, qos = ?qos, bytes_len = b.len(), "received command");

    if topic.ends_with("command/tx") {
        let tx: TxCommand = minicbor::decode(b.as_slice())?;
        handle_tx(tx).await?;
    } else if topic.ends_with("command/config") {
        let config: ConfigCommand = minicbor::decode(b.as_slice())?;
        handle_config(config).await?;
    }

    Ok(())
}

async fn handle_config(cmd: ConfigCommand) -> Result<()> {
    let mut state = STATE.get().unwrap().lock().await;
    state.radio.stop()?;
    state.radio.configure_data_rates(&cmd.data_rates)?;
    state
        .radio
        .configure_channels(&cmd.uplink_channels, &cmd.downlink_channels)?;
    state.radio.start()?;
    state.heartbeat_state.config_version = cmd.version.clone();

    Ok(())
}

async fn handle_tx(cmd: TxCommand) -> Result<()> {
    let mut state = STATE.get().unwrap().lock().await;
    let res = state.radio.send(&cmd);
    let status = match res {
        Err(e) => e,
        Ok(()) => TxAckStatus::Ok,
    };

    let pl = TxAckEvent { id: cmd.id, status };

    state.publish_tx_ack_event(&pl).await
}
