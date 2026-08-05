use std::collections::HashSet;
use std::time::Duration;

use anyhow::{Result, anyhow};
use chrono::Utc;
use tracing::{Instrument, Level, error, info, span, trace, warn};

use nanopulse::frames::{self, FrameType, key_exchange_request, key_exchange_response};
use nanopulse::keypair;
use nanopulse::keys;
use nanopulse::x25519_dalek::{PublicKey, StaticSecret};
use nanopulse_structs::gateway::{Timing, TimingDelay, TxCommand};

use crate::config;
use crate::errors::Error;
use crate::flow::rx::RxFrameSet;
use crate::gateway::send_tx_command;
use crate::integration;
use crate::storage::{device, gateway, workspace};

pub struct Flow {
    frame_set: RxFrameSet,
    payload: key_exchange_request::KeyExchangeRequest,
    device: device::Device,
    workspace: workspace::Workspace,
}

impl Flow {
    pub fn new(frame_set: RxFrameSet) -> Result<Self, Error> {
        let payload = match &frame_set.frame.payload {
            frames::Payload::KeyExchangeRequest(pl) => pl.clone(),
            _ => return Err(anyhow!("unexepected payload").into()),
        };

        Ok(Flow {
            frame_set,
            payload,
            device: Default::default(),
            workspace: Default::default(),
        })
    }

    pub async fn handle(&mut self) {
        let short_id = hex::encode(keypair::get_short_id(&self.payload.device_pub_key));
        let span = span!(Level::INFO, "key_request", short_id = %short_id);

        let res = self._handle().instrument(span).await;

        // Abort is not a real error
        if let Err(Error::Abort) = res {
        } else if let Err(e) = res {
            error!(error = %e, "error handle key request");
        }
    }

    async fn _handle(&mut self) -> Result<(), Error> {
        match self.get_device().await {
            Err(Error::NotFound) => self.send_key_exchange_notification().await?,
            Err(v) => return Err(v),
            Ok(_) => {
                let span = span!(Level::INFO, "", workspace_name = %self.workspace.name, device_name = self.device.name);

                async {
                    self.get_key_exchange_completed().await?;
                    self.update_device().await?;
                    self.send_key_response().await?;
                    Ok::<(), Error>(())
                }
                .instrument(span)
                .await?;
            }
        }

        Ok(())
    }

    async fn get_device(&mut self) -> Result<(), Error> {
        trace!("get_device fn");

        self.device = device::get_by_public_key(&self.payload.device_pub_key).await?;
        self.workspace = workspace::get(self.device.workspace_id).await?;

        Ok(())
    }

    async fn get_key_exchange_completed(&self) -> Result<(), Error> {
        trace!("get_key_exchange_completed fn");

        if !self.device.root_key.is_empty() {
            info!("device already complete the Key Exchange");
            return Err(Error::Abort);
        }

        info!("device has not yet fully completed the Key Exchange");

        Ok(())
    }

    async fn update_device(&mut self) -> Result<(), Error> {
        trace!("update_device fn");
        self.device = device::update(
            self.device.id,
            &device::DeviceChangeSet {
                key_exchange_at: Some(Utc::now()),
                ..Default::default()
            },
        )
        .await?;

        Ok(())
    }

    async fn send_key_response(&self) -> Result<(), Error> {
        trace!("send_key_response fn");
        let conf = config::get();

        let dl_ch = self
            .frame_set
            .region
            .get_downlink_channel(self.frame_set.ul_ch);
        let dl_dr = self
            .frame_set
            .region
            .get_downlink_data_rate(self.frame_set.ul_dr);
        let dl_gw = self.frame_set.get_downlink_gateway();

        let resp_nonce =
            device::get_next_counter(self.device.id, device::KEY_EXCHANGE_RESPONSE_COUNTER).await?;

        let our_secret = StaticSecret::from(conf.keypair.secret_key);
        let their_pk = PublicKey::from(self.payload.device_pub_key);
        let ss = keys::get_shared_secret(&our_secret, &their_pk);

        let root = keys::get_root_key(
            &ss,
            self.device.pin.as_ref(),
            self.payload.nonce,
            resp_nonce,
        );

        device::insert_key_exchange(
            self.device.id,
            self.payload.nonce as i64,
            resp_nonce as i64,
            &root,
        )
        .await?;

        let mic_key = keys::get_mic_key(&root, FrameType::KeyExchangeResponse, dl_ch, dl_dr);
        let mut pl = frames::Frame {
            payload: frames::Payload::KeyExchangeResponse(
                key_exchange_response::KeyExchangeResponse {
                    server_pub_key: conf.keypair.public_key,
                    counter: resp_nonce,
                },
            ),
            mic: None,
        };
        pl.set_mic(&mic_key)?;

        let tx_cmd = TxCommand {
            id: 0,
            payload: pl.try_into()?,
            channel: dl_ch,
            data_rate: dl_dr,
            tx_power: self.frame_set.region.downlink_tx_power_eirp as i32,
            timing: Timing::Delay(TimingDelay {
                delay_ns: Duration::from_millis(
                    self.frame_set.region.timing_multiplier_ms as u64
                        * self.frame_set.region.tx_delay_key_response as u64,
                )
                .as_nanos() as u64,
                context: dl_gw.context.clone(),
            }),
        };

        send_tx_command(&dl_gw.gateway_name, &tx_cmd).await?;

        Ok(())
    }

    async fn send_key_exchange_notification(&self) -> Result<(), Error> {
        trace!("send_key_exchange_notification fn");
        let mut workspace_ids: HashSet<i64> = HashSet::new();
        let gateway_names: Vec<String> = self
            .frame_set
            .rx_info
            .iter()
            .map(|v| v.gateway_name.clone())
            .collect();

        for id in gateway_names {
            let gw = gateway::get_by_name(&id).await?;
            if let Some(ts) = &gw.allow_key_exchange_until
                && Utc::now().lt(ts)
            {
                workspace_ids.insert(gw.workspace_id);
            }
        }

        if workspace_ids.is_empty() {
            warn!(
                "key-exchange request ignored as no gateway has allow_key_exchange_until set to future timestamp"
            );
        }

        for workspace_id in workspace_ids {
            let w = workspace::get(workspace_id).await?;

            info!(
                workspace_name = %w.name,
                "publishing key-exchange integration notification"
            );
            integration::key_exchange_event(&integration::KeyExchangeEvent {
                timestamp: Utc::now(),
                workspace_name: w.name.clone(),
                device_short_id: hex::encode(keypair::get_short_id(&self.payload.device_pub_key)),
                device_public_key: hex::encode(self.payload.device_pub_key),
            })
            .await;
        }

        Ok(())
    }
}
