use anyhow::Result;
use tracing::error;

use crate::storage::device;

pub async fn set_device_state_desired(device_name: String, state: serde_json::Value) {
    if let Err(e) = _set_device_state_desired(device_name, state).await {
        error!(error = %e, "set device state error");
    }
}

async fn _set_device_state_desired(device_name: String, state: serde_json::Value) -> Result<()> {
    let mut d = device::get_by_name(&device_name).await?;
    json_patch::merge(&mut d.state_desired, &state);
    let c = device::DeviceChangeSet {
        state_desired: Some(d.state_desired.clone()),
        ..Default::default()
    };
    device::update(d.id, &c).await?;

    Ok(())
}
