use crate::errors::Error;
use crate::flow::rx::RxFrameSet;
use crate::storage::{device, workspace};

pub mod configuration;
pub mod state;

pub async fn response_flow(
    frame_set: RxFrameSet,
    device: device::Device,
    workspace: workspace::Workspace,
) -> Result<(), Error> {
    if device.needs_configuration_sync() {
        configuration::Flow::new(frame_set, device, workspace)
            .handle_response()
            .await
    } else if device.needs_state_sync() {
        state::Flow::new(frame_set, device, workspace)
            .handle_response()
            .await
    } else {
        Ok(())
    }
}
