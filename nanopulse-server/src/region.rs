use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::{Result, anyhow};
use mlua::{Function, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tracing::info;

use crate::errors::Error;
use crate::lua;

static REGIONS: LazyLock<Mutex<HashMap<String, Arc<Region>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Default, Debug, Clone, Deserialize)]
pub enum Modulation {
    #[default]
    #[serde(rename = "LORA")]
    Lora,
    #[serde(rename = "FSK")]
    Fsk,
}

#[derive(Default, Debug, Deserialize, Clone, Copy)]
pub enum CodingRate {
    #[default]
    #[serde(rename = "CR4_5")]
    Cr45,
}

impl From<CodingRate> for nanopulse_structs::common::CodingRate {
    fn from(val: CodingRate) -> Self {
        match val {
            CodingRate::Cr45 => nanopulse_structs::common::CodingRate::Cr45,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    pub frequency: u32,
    pub data_rates: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct DataRate {
    pub modulation: Modulation,
    pub bandwidth: u32,
    pub spreading_factor: u8,
    pub coding_rate: CodingRate,
    pub bitrate: u32,
    pub frequency_deviation: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Region {
    pub name: String,
    pub description: String,
    pub version: String,
    pub timing_multiplier_ms: usize,
    pub tx_delay_key_response: usize,
    pub tx_delay_join_accept: usize,
    pub tx_delay_data: usize,
    pub uplink_channels: HashMap<u8, Channel>,
    pub downlink_channels: HashMap<u8, Channel>,
    pub downlink_tx_power_eirp: i8,
    pub uplink_downlink_channel_mapping: HashMap<u8, u8>,
    pub data_rates: HashMap<u8, DataRate>,
    pub uplink_downlink_data_rate_mapping: HashMap<u8, u8>,
}

impl Region {
    pub fn init(name: &str) -> Result<Self> {
        let lua = lua::get()?;
        let require: Function = lua.globals().get("require")?;
        let region: Table = require.call(name)?;

        Ok(lua.from_value(Value::Table(region))?)
    }

    pub fn get_downlink_channel(&self, ul_chan: u8) -> u8 {
        self.uplink_downlink_channel_mapping
            .get(&ul_chan)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_downlink_data_rate(&self, ul_dr: u8) -> u8 {
        self.uplink_downlink_data_rate_mapping
            .get(&ul_dr)
            .cloned()
            .unwrap_or_default()
    }
}

pub fn load_all() -> Result<()> {
    info!("loading all regions");
    let mut regions = REGIONS.lock().unwrap();

    let modules = walk_module("regions")?;
    for m in modules {
        info!(region_module = %m, "loading region");
        let r = Region::init(&m)?;
        regions.insert(m, Arc::new(r));
    }

    Ok(())
}

pub fn walk_module(prefix: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();

    let lua = lua::get()?;
    let require: Function = lua.globals().get("require")?;
    let manifest: Table = match require.call(format!("{}.manifest", prefix)) {
        Ok(v) => v,
        Err(_) => {
            return Ok(vec![prefix.to_string()]);
        }
    };

    let manifests: Vec<String> = lua.from_value(Value::Table(manifest))?;
    let manifests: Vec<String> = manifests
        .iter()
        .map(|v| format!("{}.{}", prefix, v))
        .collect();

    for m in manifests {
        out.extend_from_slice(&walk_module(&m)?);
    }

    Ok(out)
}

pub fn get(name: &str) -> Result<Arc<Region>, Error> {
    info!(region = %name, "Getting region configuration");

    let mut regions = REGIONS.lock().map_err(|e| anyhow!("lock error: {}", e))?;
    if let Some(r) = regions.get(name) {
        return Ok(r.clone());
    }

    let region = Arc::new(Region::init(name)?);
    let out = region.clone();
    regions.insert(name.to_string(), region);

    Ok(out)
}

pub fn get_keys() -> Result<Vec<String>, Error> {
    let regions = REGIONS.lock().map_err(|e| anyhow!("lock error: {}", e))?;
    let mut keys: Vec<String> = regions.keys().cloned().collect();
    keys.sort();
    Ok(keys)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test;

    #[tokio::test]
    async fn test_lua() {
        let _guard = test::prepare().await;
        let region = Region::init("regions.lora.eu868").unwrap();
        assert_eq!("EU868", region.name);
        assert_eq!(8, region.uplink_channels.len());
    }
}
