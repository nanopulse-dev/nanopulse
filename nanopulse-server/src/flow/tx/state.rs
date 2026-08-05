use std::time::Duration;

use tracing::{Instrument, Level, span, trace, warn};

use nanopulse::{
    frames,
    frames::{FrameType, state},
    keys,
};
use nanopulse_structs::gateway::{Timing, TimingDelay, TxCommand};

use crate::errors::Error;
use crate::flow::rx::RxFrameSet;
use crate::gateway::send_tx_command;
use crate::profile::{encode_state, json_diff};
use crate::storage::{device, workspace};

pub struct Flow {
    frame_set: RxFrameSet,
    device: device::Device,
    workspace: workspace::Workspace,
    payload: Vec<u8>,
    state_down_counter: u32,
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
            payload: vec![],
            state_down_counter: 0,
        }
    }

    pub async fn handle_response(&mut self) -> Result<(), Error> {
        let span = span!(Level::INFO, "tx_state");
        self._handle_response().instrument(span).await
    }

    async fn _handle_response(&mut self) -> Result<(), Error> {
        self.get_diff_payload().await?;
        self.update_device().await?;
        self.send_state().await?;
        Ok(())
    }

    async fn get_diff_payload(&mut self) -> Result<(), Error> {
        trace!("get_diff_payload fn");

        let state_diff = json_diff(&self.device.state, &self.device.state_desired);
        let pl = encode_state(&self.device.lua_profile_module(), &state_diff)?;
        if pl.is_empty() {
            warn!("encoded diff payload is empty, aborting");
            return Err(Error::Abort);
        }

        self.payload = pl;

        Ok(())
    }

    async fn update_device(&mut self) -> Result<(), Error> {
        trace!("update_device fn");

        self.state_down_counter =
            device::get_next_counter(self.device.id, device::STATE_DOWN_COUNTER).await?;

        Ok(())
    }

    async fn send_state(&self) -> Result<(), Error> {
        trace!("send_state fn");

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
            FrameType::State,
            true,
        );
        let mic_key = keys::get_mic_key(
            self.device.session_root_key.as_ref(),
            FrameType::State,
            dl_ch,
            dl_dr,
        );

        let mut pl = frames::Frame {
            payload: frames::Payload::State(state::State {
                is_downlink: true,
                short_id: self.device.short_id.into(),
                counter: self.state_down_counter,
                payload: self.payload.as_slice().try_into()?,
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
}
