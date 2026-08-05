use anyhow::anyhow;
use chrono::Utc;
use tracing::{Instrument, Level, error, info, span, trace, warn};

use nanopulse::frames::{self, FrameType, state};
use nanopulse::keys;

use crate::errors::Error;
use crate::flow::rx::RxFrameSet;
use crate::integration;
use crate::profile::decode_state;
use crate::storage::{device, workspace};

pub struct Flow {
    frame_set: RxFrameSet,
    payload: state::State,
    device: device::Device,
    workspace: workspace::Workspace,
    state: Option<serde_json::Value>,
    device_changeset: device::DeviceChangeSet,
}

impl Flow {
    pub fn new(frame_set: RxFrameSet) -> Result<Self, Error> {
        let payload = match &frame_set.frame.payload {
            frames::Payload::State(pl) => pl.clone(),
            _ => return Err(anyhow!("unexpected payload").into()),
        };

        Ok(Flow {
            frame_set,
            payload,
            device: Default::default(),
            workspace: Default::default(),
            state: None,
            device_changeset: device::DeviceChangeSet::default(),
        })
    }

    pub async fn handle(&mut self) {
        let short_id = hex::encode(self.payload.short_id);
        let span = span!(Level::INFO, "state", short_id = %short_id);

        let res = self._handle().instrument(span).await;

        // Abort is not a real error
        if let Err(Error::Abort) = res {
        } else if let Err(e) = res {
            error!(error = %e, "error handling state");
        }
    }

    async fn _handle(&mut self) -> Result<(), Error> {
        self.get_device().await?;
        self.get_workspace().await?;

        let span = span!(Level::INFO, "", workspace_name = %self.workspace.name, device_name = %self.device.name);

        async {
            self.decrypt_payload()?;
            self.decode_state()?;
            self.update_device().await?;
            self.send_to_integration().await?;
            Ok(())
        }
        .instrument(span)
        .await
    }

    async fn get_device(&mut self) -> Result<(), Error> {
        trace!("get_device fn");

        let mut devices = device::get_by_short_id(&self.payload.short_id).await?;
        if devices.is_empty() {
            warn!("no devices found matching short_id");
            return Err(Error::Abort);
        }

        info!(count = devices.len(), "devices found matching short_id");
        for device in &mut devices {
            let mic_key = keys::get_mic_key(
                device.session_root_key.as_ref(),
                FrameType::State,
                self.frame_set.ul_ch,
                self.frame_set.ul_dr,
            );

            if self.frame_set.frame.validate_mic(&mic_key)? {
                info!(id = device.id, "found matching device");

                device::sync_counter(device.id, device::STATE_UP_COUNTER, self.payload.counter)
                    .await?;
                self.device = device.clone();
                return Ok(());
            }
        }

        warn!("no matching device found");
        Err(Error::Abort)
    }

    async fn get_workspace(&mut self) -> Result<(), Error> {
        trace!("get_workspace fn");
        self.workspace = workspace::get(self.device.workspace_id).await?;
        Ok(())
    }

    fn decrypt_payload(&mut self) -> Result<(), Error> {
        trace!("decrypt_payload fn");

        let enc_key = keys::get_encryption_key(
            self.device.session_root_key.as_ref(),
            FrameType::State,
            false,
        );
        self.payload.decrypt(&enc_key)?;

        Ok(())
    }

    fn decode_state(&mut self) -> Result<(), Error> {
        trace!("decode_state fn");
        let state = decode_state(&self.device.lua_profile_module(), &self.payload.payload)?;

        json_patch::merge(&mut self.device.state, &state);
        self.state = Some(state);
        self.device_changeset.state = Some(self.device.state.clone());

        Ok(())
    }

    async fn update_device(&mut self) -> Result<(), Error> {
        trace!("update_device fn");
        self.device = device::update(self.device.id, &self.device_changeset).await?;
        Ok(())
    }

    async fn send_to_integration(&self) -> Result<(), Error> {
        trace!("send_to_integration fn");
        let s = self.state.as_ref().unwrap();

        let pl = integration::StateEvent {
            timestamp: Utc::now(),
            device_info: integration::DeviceInfo::new(&self.workspace, &self.device),
            state: s.clone(),
        };

        integration::state_event(&pl).await;

        Ok(())
    }
}
