use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::Result;
use serde::{Deserialize, Serialize};

static CONFIG: LazyLock<Mutex<Arc<Configuration>>> =
    LazyLock::new(|| Mutex::new(Arc::new(Default::default())));

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub logging: Logging,
    pub keypair: Keypair,
    pub database: Database,
    pub api: Api,
    pub mqtt: Mqtt,
    pub integration: Integration,
    pub geolocation: Geolocation,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Logging {
    pub level: String,
}

impl Default for Logging {
    fn default() -> Self {
        Logging {
            level: "info".into(),
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Keypair {
    #[serde(with = "hex_encode")]
    pub public_key: [u8; 32],
    #[serde(with = "hex_encode")]
    pub secret_key: [u8; 32],
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Database {
    pub path: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Api {
    pub bind: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Mqtt {
    pub server: String,
    pub client_id: String,
    pub username: String,
    pub password: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Integration {
    pub home_assistant: HomeAssistant,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Geolocation {
    pub wifi_backend: String,
    pub google: GeolocationGoogle,
}

impl Default for Geolocation {
    fn default() -> Self {
        Geolocation {
            wifi_backend: "beacondb".into(),
            google: Default::default(),
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GeolocationGoogle {
    pub api_key: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeAssistant {
    pub enabled: bool,
    pub mqtt_server: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
}

pub fn load(path: PathBuf) -> Result<()> {
    let c = config::Config::builder()
        .add_source(config::File::with_name(path.to_str().unwrap()))
        .add_source(config::Environment::default().separator("__"))
        .build()?;

    let conf: Configuration = c.try_deserialize()?;

    let mut config_mx = CONFIG.lock().unwrap();
    *config_mx = Arc::new(conf);

    Ok(())
}

#[cfg(test)]
pub fn set(conf: Configuration) {
    let mut config_mx = CONFIG.lock().unwrap();
    *config_mx = Arc::new(conf);
}

pub fn get() -> Arc<Configuration> {
    CONFIG.lock().unwrap().clone()
}

mod hex_encode {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(b: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(b))
    }

    pub fn deserialize<'a, D, O>(deserializer: D) -> Result<O, D::Error>
    where
        D: Deserializer<'a>,
        O: std::convert::TryFrom<std::vec::Vec<u8>>,
    {
        let s: String = serde::de::Deserialize::deserialize(deserializer)?;

        // HEX encoded values may start with 0x prefix, we must strip this.
        let s = s.trim_start_matches("0x");

        hex::decode(s)
            .map_err(serde::de::Error::custom)?
            .try_into()
            .map_err(|_| serde::de::Error::custom("value does not fit"))
    }
}
