use std::path::PathBuf;

use anyhow::{Result, anyhow};
use tracing::info;

use crate::config;
use crate::traits::Radio;

mod jitqueue;
#[cfg(feature = "semtech_sx1302")]
mod semtech_helpers;
#[cfg(feature = "semtech_sx1302")]
mod semtech_sx1302;

pub async fn get(
    chipset_vendor: &str,
    chipset: &str,
    board_mapping: &str,
    board_config_file: &PathBuf,
) -> Result<Box<dyn Radio>> {
    info!(chipset_vendor = %chipset_vendor, chipset = %chipset, board_config_file = %board_config_file.display(), "Initializing radio");

    let radio = match chipset_vendor {
        "semtech" => match chipset {
            #[cfg(feature = "semtech_sx1302")]
            "sx1302" => {
                let board_config = config::semtech_sx1302::get(board_config_file)?;
                semtech_sx1302::new(&board_config, board_mapping)?
            }
            _ => return Err(anyhow!("unknown chipset: {}", chipset)),
        },
        _ => return Err(anyhow!("unknown chipset vendor: {}", chipset_vendor)),
    };

    Ok(radio)
}
