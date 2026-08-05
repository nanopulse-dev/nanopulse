use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[cfg(feature = "semtech_sx1302")]
pub mod semtech_sx1302;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub gateway: Gateway,
    pub mqtt: Mqtt,
}

impl Configuration {
    pub fn get_board_config_file_path(&self, config_dir: &Path) -> PathBuf {
        config_dir
            .join(&self.gateway.chipset_vendor)
            .join(&self.gateway.chipset)
            .join(&self.gateway.board_vendor)
            .join(format!("{}.toml", &self.gateway.board_model))
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Gateway {
    pub name: String,
    pub chipset_vendor: String,
    pub chipset: String,
    pub board_vendor: String,
    pub board_model: String,
    pub board_mapping: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Mqtt {
    pub server: String,
    pub client_id: String,
    pub username: String,
    pub password: String,
}

pub fn get(config_dir: &Path) -> Result<Configuration> {
    let conf_file = config_dir.join("nanopulse-gateway.toml");
    info!(config_file = ?conf_file, "reading configuration file");
    let c = config::Config::builder()
        .add_source(config::File::with_name(&conf_file.to_str().unwrap()))
        .add_source(
            config::Environment::with_prefix("NP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    let conf: Configuration = c.try_deserialize()?;
    Ok(conf)
}
