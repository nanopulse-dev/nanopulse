use std::time::Duration;

use anyhow::anyhow;
use chrono::Utc;
use nanopulse::keys;
use tracing::{Instrument, Level, debug, error, info, span, trace, warn};

use nanopulse::frames::{self, FrameType, activation_request, activation_response};
use nanopulse_structs::gateway::{Timing, TimingDelay, TxCommand};

use crate::errors::Error;
use crate::flow::rx::RxFrameSet;
use crate::gateway::send_tx_command;
use crate::storage::{device, fields, workspace};
use crate::{config, integration, profile};

pub struct Flow {
    frame_set: RxFrameSet,
    payload: activation_request::ActivationRequest,
    device: device::Device,
    device_changeset: device::DeviceChangeSet,
    workspace: workspace::Workspace,
    activation_request_payload: Option<activation_request::ProfileInfoPlain>,
    vendor: Option<profile::Vendor>,
    profile: Option<profile::Profile>,
}

impl Flow {
    pub fn new(frame_set: RxFrameSet) -> Result<Self, Error> {
        let payload = match &frame_set.frame.payload {
            frames::Payload::ActivationRequest(pl) => pl.clone(),
            _ => return Err(anyhow!("unexpected payload").into()),
        };

        Ok(Flow {
            frame_set,
            payload,
            device: Default::default(),
            device_changeset: Default::default(),
            workspace: Default::default(),
            activation_request_payload: None,
            vendor: None,
            profile: None,
        })
    }

    pub async fn handle(&mut self) {
        let short_id = hex::encode(self.payload.short_id);
        let span = span!(Level::INFO, "activation_request", short_id = %short_id);

        let res = self._handle().instrument(span).await;

        // Abort is not a real error
        if let Err(Error::Abort) = res {
        } else if let Err(e) = res {
            error!(error = %e, "error handling activation request");
        }
    }

    async fn _handle(&mut self) -> Result<(), Error> {
        self.find_device().await?;
        self.get_workspace().await?;

        let span = span!(Level::INFO, "", workspace_name = %self.workspace.name, device_name = %self.device.name);

        async {
            self.decrypt_payload().await?;
            self.get_profile().await?;
            self.compute_session_root_key();
            self.send_activation_response().await?;
            self.send_activation_event().await?;
            self.update_device().await?;
            Ok(())
        }
        .instrument(span)
        .await
    }

    async fn find_device(&mut self) -> Result<(), Error> {
        trace!("find_device fn");

        let mut devices = device::get_by_short_id(&self.payload.short_id).await?;
        if devices.is_empty() {
            warn!("no devices found matching short_id");
            return Err(Error::Abort);
        }

        info!(count = devices.len(), "devices found matching short_id");
        for device in &mut devices {
            let root_keys: Vec<fields::HexArray<32>> = if device.root_key.is_empty() {
                debug!("device has not yet completed key exchange, query key exchanges");
                device::get_key_exchanges(device.id)
                    .await?
                    .into_iter()
                    .map(|v| v.root_key)
                    .collect()
            } else {
                debug!("device has already completed key exchange, using root_key");
                vec![device.root_key]
            };

            for root_key in &root_keys {
                let mic_key = keys::get_mic_key(
                    root_key.as_ref(),
                    FrameType::ActivationRequest,
                    self.frame_set.ul_ch,
                    self.frame_set.ul_dr,
                );

                if self.frame_set.frame.validate_mic(&mic_key)? {
                    info!(id = device.id, "found matching device");

                    // This validates that the counter has incremented + immediately updates the
                    // row with the new counter to avoid concurrency issues. Other updates happen
                    // through the device_changeset.
                    self.device = device::sync_counter(
                        device.id,
                        device::Counter::ActivationRequestCounter(self.payload.counter),
                    )
                    .await?;

                    if device.root_key.is_empty() {
                        info!("finalizing key exchange by setting root_key");
                        self.device.root_key = *root_key;
                        self.device_changeset.root_key = Some(*root_key);
                    }

                    return Ok(());
                }
            }
        }

        warn!("failed to validate activation request mic");
        Err(Error::Abort)
    }

    async fn get_workspace(&mut self) -> Result<(), Error> {
        trace!("get_workspace fn");
        self.workspace = workspace::get(self.device.workspace_id).await?;
        Ok(())
    }

    async fn decrypt_payload(&mut self) -> Result<(), Error> {
        trace!("decrypt_payload fn");

        let enc_key = keys::get_encryption_key(
            self.device.root_key.as_ref(),
            FrameType::ActivationRequest,
            false,
        );
        self.payload.decrypt(&enc_key)?;

        if let activation_request::ProfileInfo::Plain(v) = &self.payload.profile_info {
            self.activation_request_payload = Some(v.clone());
        } else {
            return Err(Error::Anyhow(anyhow!("expected plain payload")));
        }
        Ok(())
    }

    async fn get_profile(&mut self) -> Result<(), Error> {
        trace!("get_profile fn");
        let pl = self.activation_request_payload.as_ref().unwrap();

        let v = profile::get_vendor(&pl.vendor_id)?;
        let p = profile::Profile::get_by_vendor_profile_version(
            &pl.vendor_id,
            &pl.profile_id,
            &pl.version_id,
        )?;

        self.vendor = Some(v);
        self.profile = Some(p);

        Ok(())
    }

    fn compute_session_root_key(&mut self) {
        trace!("compute_session_root_key fn");

        self.device.session_root_key = keys::get_session_root_key(
            self.device.root_key.as_ref(),
            self.payload.counter,
            self.device.activation_resp_counter,
        )
        .into();
        self.device_changeset.session_root_key = Some(self.device.session_root_key);
    }

    async fn update_device(&mut self) -> Result<(), Error> {
        trace!("update_device fn");
        let pl = self.activation_request_payload.as_ref().unwrap();
        let v = self.vendor.as_ref().unwrap();
        let p = self.profile.as_ref().unwrap();

        self.device_changeset.activation_at = Some(Utc::now());
        self.device_changeset.vendor_id = Some(pl.vendor_id.into());
        self.device_changeset.profile_id = Some(pl.profile_id.into());
        self.device_changeset.version_id = Some(pl.version_id.into());
        self.device_changeset.vendor_name = Some(v.name.clone());
        self.device_changeset.counter_up = Some(0);
        self.device_changeset.counter_down = Some(0);
        self.device_changeset.state = Some(p.get_default_state());
        self.device_changeset.configuration = Some(p.get_default_configuration());

        if self.device.name.is_empty() {
            self.device_changeset.name = Some(p.name.clone());
        }

        if self.device.description.is_empty() {
            self.device_changeset.description = Some(p.description.clone());
        }

        self.device = device::update(self.device.id, &self.device_changeset).await?;

        Ok(())
    }

    async fn send_activation_response(&mut self) -> Result<(), Error> {
        trace!("send_activation_response fn");

        let dl_ch = self
            .frame_set
            .region
            .get_downlink_channel(self.frame_set.ul_ch);
        let dl_dr = self
            .frame_set
            .region
            .get_downlink_data_rate(self.frame_set.ul_dr);
        let dl_gw = self.frame_set.get_downlink_gateway();

        let enc_key = keys::get_encryption_key(
            self.device.session_root_key.as_ref(),
            FrameType::ActivationResponse,
            true,
        );
        let mic_key = keys::get_mic_key(
            self.device.session_root_key.as_ref(),
            FrameType::ActivationResponse,
            dl_ch,
            dl_dr,
        );

        let conf = config::get();
        let network_defaults = conf
            .network
            .defaults
            .get(&self.frame_set.region_module)
            .cloned()
            .unwrap_or_default();

        let activation_response_counter = self.device.activation_resp_counter;
        self.device_changeset.activation_resp_counter =
            Some(self.device.activation_resp_counter + 1);

        let mut pl = frames::Frame {
            payload: frames::Payload::ActivationResponse(activation_response::ActivationResponse {
                counter: activation_response_counter,
                network_info: activation_response::NetworkInfo::Plain(
                    activation_response::NetworkInfoPlain {
                        channel_plan_id: network_defaults.channel_plan_id,
                        rx_window_delay: network_defaults.rx_window_delay,
                        dr_offset: network_defaults.dr_offset,
                    },
                ),
            }),
            mic: None,
        };
        pl.encrypt(&enc_key)?;
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

    async fn send_activation_event(&self) -> Result<(), Error> {
        trace!("send_activation_event fn");

        let p = self.profile.as_ref().unwrap();

        let pl = integration::ActivationEvent {
            timestamp: Utc::now(),
            device_info: integration::DeviceInfo::new(&self.workspace, &self.device),
            telemetry_schema: p.telemetry_schema.clone(),
            state_schema: p.state_schema.clone(),
        };
        integration::activation_event(&pl).await;

        Ok(())
    }
}
