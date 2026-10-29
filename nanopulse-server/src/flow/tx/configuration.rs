use std::time::Duration;

use tracing::{Instrument, Level, span, trace, warn};

use nanopulse::{
    frames,
    frames::{FrameType, configuration},
    keys,
};
use nanopulse_structs::gateway::{Timing, TimingDelay, TxCommand};

use crate::errors::Error;
use crate::flow::rx::RxFrameSet;
use crate::gateway::send_tx_command;
use crate::profile::{encode_configuration, json_diff};
use crate::storage::{device, workspace};

pub struct Flow {
    frame_set: RxFrameSet,
    device: device::Device,
    device_changeset: device::DeviceChangeSet,
    workspace: workspace::Workspace,
    config_diff: serde_json::Value,
    config_diff_pl: Vec<u8>,
}

impl Flow {
    pub fn new(
        frame_set: RxFrameSet,
        device: device::Device,
        workspace: workspace::Workspace,
    ) -> Self {
        Flow {
            frame_set,
            device,
            workspace,
            device_changeset: Default::default(),
            config_diff: Default::default(),
            config_diff_pl: vec![],
        }
    }

    pub async fn handle_response(&mut self) -> Result<(), Error> {
        let span = span!(Level::INFO, "configuration");
        self._handle_response().instrument(span).await
    }

    async fn _handle_response(&mut self) -> Result<(), Error> {
        self.get_diff_payload().await?;
        self.store_pending_request().await?;
        self.send_configuration().await?;
        self.update_device().await?;

        Ok(())
    }

    async fn get_diff_payload(&mut self) -> Result<(), Error> {
        trace!("get_diff_payload fn");

        self.config_diff = json_diff(
            &self.device.configuration,
            &self.device.configuration_desired,
        );
        self.config_diff_pl =
            encode_configuration(&self.device.lua_profile_module(), &self.config_diff)?;
        if self.config_diff_pl.is_empty() {
            warn!("encoded diff payload is empty, aborting");
            return Err(Error::Abort);
        }

        Ok(())
    }

    async fn store_pending_request(&self) -> Result<(), Error> {
        trace!("store_pending_request fn");

        device::insert_pending_request(device::DevicePendingRequest {
            device_id: self.device.id,
            request_counter: (self.device.counter_down as u16).into(),
            payload: self.config_diff.clone(),
            ..Default::default()
        })
        .await?;

        Ok(())
    }

    async fn send_configuration(&mut self) -> Result<(), Error> {
        trace!("send_configuration fn");

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
            FrameType::Configuration,
            true,
        );
        let mic_key = keys::get_mic_key(
            self.device.session_root_key.as_ref(),
            FrameType::Configuration,
            dl_ch,
            dl_dr,
        );

        let counter_down = self.device.counter_down;
        self.device_changeset.counter_down = Some(self.device.counter_down + 1);

        let mut pl = frames::Frame {
            payload: frames::Payload::Configuration(configuration::Configuration {
                is_downlink: true,
                is_resp: false,
                short_id: self.device.short_id.into(),
                counter: counter_down,
                payload: configuration::Payload::Request(
                    self.config_diff_pl.as_slice().try_into()?,
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
                        * self.frame_set.region.tx_delay_data as u64,
                )
                .as_nanos() as u64,
                context: dl_gw.context.clone(),
            }),
        };

        send_tx_command(&dl_gw.gateway_name, &tx_cmd).await?;

        Ok(())
    }

    async fn update_device(&mut self) -> Result<(), Error> {
        trace!("update device fn");

        self.device = device::update(self.device.id, &self.device_changeset).await?;

        Ok(())
    }
}
