use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub antenna_gain_dbi: i8,
    pub concentrator: Concentrator,
    pub radios: [Radio; 2],
    pub mappings: HashMap<String, Mapping>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Concentrator {
    pub clock_source: u8,
    pub full_duplex: bool,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Radio {
    pub tx_enabled: bool,
    pub radio_type: String,
    pub rssi_offset: f32,
    pub tx_gain_table: [TxGain; 16],
    pub rssi_temp_compensation: RssiTempCompensation,
    pub single_input_mode: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct TxGain {
    pub rf_power: i8,
    pub dig_gain: u8,
    pub pa_gain: u8,
    pub dac_gain: u8,
    pub mix_gain: u8,
    pub offset_i: i8,
    pub offset_q: i8,
    pub pwr_idx: u8,
}

impl Default for TxGain {
    fn default() -> Self {
        TxGain {
            rf_power: 0,
            dig_gain: 0,
            pa_gain: 0,
            dac_gain: 0,
            mix_gain: 5,
            offset_i: 0,
            offset_q: 0,
            pwr_idx: 0,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RssiTempCompensation {
    pub coeff_a: f32,
    pub coeff_b: f32,
    pub coeff_c: f32,
    pub coeff_d: f32,
    pub coeff_e: f32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Mapping {
    pub reset_chip: String,
    pub reset_pin: u32,
    pub com_dev_spi: String,
    pub com_dev_usb: String,
}

impl Default for Mapping {
    fn default() -> Self {
        Mapping {
            reset_chip: "/dev/gpiochip0".into(),
            reset_pin: 0,
            com_dev_spi: "".into(),
            com_dev_usb: "".into(),
        }
    }
}

pub fn get(config_file: &PathBuf) -> Result<Configuration> {
    info!(config_file = ?config_file, "Reading board configuration file");
    let c = config::Config::builder()
        .add_source(config::File::with_name(&config_file.to_str().unwrap()))
        .build()?;

    let conf: Configuration = c.try_deserialize()?;
    Ok(conf)
}
