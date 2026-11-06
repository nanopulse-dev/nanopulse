use anyhow::anyhow;
use chrono::Utc;
use tracing::{Instrument, Level, error, info, span, trace, warn};

use nanopulse::frames::{self, FrameType, configuration};
use nanopulse::keys;

use crate::errors::Error;
use crate::flow::rx::RxFrameSet;
use crate::flow::tx;
use crate::integration;
use crate::profile::decode_configuration_resp;
use crate::storage::{device, workspace};

pub struct Flow {
    frame_set: RxFrameSet,
    payload: configuration::Configuration,
    device: device::Device,
    workspace: workspace::Workspace,
    configuration_req: serde_json::Value,
    configuration_errors: Option<Vec<String>>,
    device_changeset: device::DeviceChangeSet,
}

impl Flow {
    pub fn new(frame_set: RxFrameSet) -> Result<Self, Error> {
        let payload = match &frame_set.frame.payload {
            frames::Payload::Configuration(pl) => pl.clone(),
            _ => return Err(anyhow!("unexpected payload").into()),
        };

        Ok(Flow {
            frame_set,
            payload,
            device: Default::default(),
            workspace: Default::default(),
            configuration_req: Default::default(),
            configuration_errors: None,
            device_changeset: device::DeviceChangeSet::default(),
        })
    }

    pub async fn handle(&mut self) {
        let short_id = hex::encode(self.payload.short_id);
        let span = span!(Level::INFO, "configuration", short_id = %short_id);

        let res = self._handle().instrument(span).await;

        // Abort is not a real error
        if let Err(Error::Abort) = res {
        } else if let Err(e) = res {
            error!(error = %e, "error handling configuration");
        }
    }

    async fn _handle(&mut self) -> Result<(), Error> {
        self.get_device().await?;
        self.get_workspace().await?;

        let span = span!(Level::INFO, "", workspace_name = %self.workspace.name, device_name = %self.device.name);

        async {
            self.decrypt_payload()?;
            self.handle_response().await?;
            self.update_device().await?;
            self.send_to_integration().await?;
            self.start_tx_flow().await?;
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
                FrameType::Configuration,
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
                    device::Counter::CounterUp(self.payload.counter),
                )
                .await?;

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
            FrameType::Configuration,
            false,
        );
        self.payload.decrypt(&enc_key)?;

        Ok(())
    }

    async fn handle_response(&mut self) -> Result<(), Error> {
        trace!("handle_response fn");

        if let configuration::Payload::Response(pl) = &self.payload.payload {
            let pending_req =
                device::get_pending_request(self.device.id, pl.request_counter as i32).await?;
            self.configuration_req = pending_req.payload.clone();

            if !pl.payload.is_empty() {
                self.configuration_errors = Some(decode_configuration_resp(
                    &self.device.lua_profile_module(),
                    &pl.payload,
                )?);
            } else {
                json_patch::merge(&mut self.device.configuration, &pending_req.payload);
                self.device_changeset.configuration = Some(self.device.configuration.clone());
            }
        } else {
            warn!("configuration payload is not a response");
            return Err(Error::Abort);
        }

        Ok(())
    }

    async fn update_device(&mut self) -> Result<(), Error> {
        trace!("update_device fn");
        self.device_changeset.last_seen_at = Some(Utc::now());
        self.device = device::update(self.device.id, &self.device_changeset).await?;
        Ok(())
    }

    async fn send_to_integration(&self) -> Result<(), Error> {
        trace!("send_to_integration fn");

        if let Some(errors) = &self.configuration_errors {
            let pl = integration::ErrorEvent {
                timestamp: Utc::now(),
                device_info: integration::DeviceInfo::new(&self.workspace, &self.device),
                request: Some(self.configuration_req.clone()),
                errors: errors.clone(),
            };
            integration::error_event(&pl).await;
        } else {
            let pl = integration::ConfigurationEvent {
                timestamp: Utc::now(),
                device_info: integration::DeviceInfo::new(&self.workspace, &self.device),
                configuration: self.configuration_req.clone(),
            };
            integration::configuration_event(&pl).await;
        }

        Ok(())
    }

    async fn start_tx_flow(&self) -> Result<(), Error> {
        trace!("start_tx_flow fn");

        tx::response_flow(
            self.frame_set.clone(),
            self.device.clone(),
            self.workspace.clone(),
        )
        .await
    }
}
