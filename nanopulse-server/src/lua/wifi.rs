use std::sync::OnceLock;
use std::time::Duration;

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result, Table};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::Error};
use tracing::info;

use crate::config;

#[derive(Deserialize)]
pub struct Bssid {
    pub bssid: [u8; 6],
    pub rssi: i16,
}

impl FromLua for Bssid {
    fn from_lua(value: mlua::prelude::LuaValue, lua: &Lua) -> Result<Self> {
        lua.from_value(value)
    }
}

#[derive(Serialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
}

impl IntoLua for Location {
    fn into_lua(self, lua: &Lua) -> Result<mlua::prelude::LuaValue> {
        lua.to_value(&self)
    }
}

static CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> Client {
    CLIENT
        .get_or_init(|| {
            Client::builder()
                .user_agent(concat!("nanopulse", "/", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(5))
                .use_rustls_tls()
                .build()
                .unwrap()
        })
        .clone()
}

pub fn wifi_resolve_location(l: &Lua, t: &Table) -> Result<()> {
    t.set(
        "wifi_resolve_location",
        l.create_async_function(resolve_location)?,
    )
}

async fn resolve_location(_l: Lua, buffer: mlua::LuaString) -> Result<Location> {
    let buffer = buffer.as_bytes();

    if buffer.is_empty() || !buffer.len().is_multiple_of(7) {
        return Err(mlua::Error::custom("invalid input data"));
    }

    let bssids: Vec<Bssid> = buffer
        .as_chunks::<7>()
        .0
        .iter()
        .map(|v| Bssid {
            bssid: v[0..6].try_into().unwrap(),
            rssi: -(v[6] as i16),
        })
        .collect();

    let conf = config::get();

    info!(wifi_backend = %conf.geolocation.wifi_backend, "resolve location");

    match conf.geolocation.wifi_backend.as_str() {
        "beacondb" => beacondb::resolve_location(&bssids)
            .await
            .map_err(mlua::Error::custom),
        "google" => google::resolve_location(&bssids, &conf.geolocation.google.api_key)
            .await
            .map_err(mlua::Error::custom),
        _ => Err(mlua::Error::custom(format!(
            "unexpected wifi geolocation backend: {}",
            conf.geolocation.wifi_backend
        ))),
    }
}

mod beacondb {
    use super::{Bssid, Deserialize, Location, Serialize, get_client};
    use anyhow::{Result, anyhow};

    #[derive(Serialize)]
    struct Request {
        #[serde(rename = "wifiAccessPoints")]
        pub wifi_access_points: Vec<WifiAp>,
    }

    #[derive(Serialize)]
    struct WifiAp {
        #[serde(rename = "macAddress")]
        pub mac_address: String,
        #[serde(rename = "signalStrength")]
        pub signal_strength: i16,
    }

    #[derive(Default, Debug, Deserialize)]
    #[serde(default)]
    struct Response {
        pub location: Option<LocationResult>,
        pub accuracy: Option<f32>,
        pub errors: Option<serde_json::Value>,
        pub fallback: Option<String>,
    }

    #[derive(Default, Debug, Deserialize)]
    #[serde(default)]
    struct LocationResult {
        pub lat: f64,
        pub lng: f64,
    }

    impl From<&Bssid> for WifiAp {
        fn from(value: &Bssid) -> Self {
            WifiAp {
                mac_address: hex::encode(value.bssid),
                signal_strength: value.rssi,
            }
        }
    }

    impl TryFrom<Response> for Location {
        type Error = anyhow::Error;

        fn try_from(value: Response) -> Result<Self, Self::Error> {
            if let Some(e) = value.errors {
                Err(anyhow!("error: {:?}", e))
            } else if value.fallback.is_some() {
                // we do not want to fallback onto IP based geolocation, as this would provide the
                // location of the server, not of the device
                Err(anyhow!("could not resolve location"))
            } else if let Some(loc) = value.location {
                Ok(Location {
                    latitude: loc.lat,
                    longitude: loc.lng,
                    accuracy: value.accuracy.unwrap_or_default(),
                })
            } else {
                Err(anyhow!("could not resolve location"))
            }
        }
    }

    pub async fn resolve_location(aps: &[Bssid]) -> Result<Location> {
        let req = Request {
            wifi_access_points: aps.iter().map(WifiAp::from).collect(),
        };
        let client = get_client();
        let res = client
            .post("https://api.beacondb.net/v1/geolocate")
            .json(&req)
            .send()
            .await?
            .json::<Response>()
            .await?;

        res.try_into()
    }
}

mod google {
    use super::{Bssid, Deserialize, Location, Serialize, get_client};
    use anyhow::{Result, anyhow};

    #[derive(Serialize)]
    struct Request {
        #[serde(rename = "wifiAccessPoints")]
        pub wifi_access_points: Vec<WifiAp>,
    }

    #[derive(Serialize)]
    struct WifiAp {
        #[serde(rename = "macAddress")]
        pub mac_address: String,
        #[serde(rename = "signalStrength")]
        pub signal_strength: i16,
    }

    #[derive(Default, Debug, Deserialize)]
    #[serde(default)]
    struct Response {
        pub location: Option<LocationResult>,
        pub accuracy: Option<f32>,
    }

    #[derive(Default, Debug, Deserialize)]
    #[serde(default)]
    struct LocationResult {
        pub lat: f64,
        pub lng: f64,
    }

    impl From<&Bssid> for WifiAp {
        fn from(value: &Bssid) -> Self {
            WifiAp {
                mac_address: value
                    .bssid
                    .iter()
                    .map(|v| format!("{:02x}", v))
                    .collect::<Vec<_>>()
                    .join(":"),
                signal_strength: value.rssi,
            }
        }
    }

    impl TryFrom<Response> for Location {
        type Error = anyhow::Error;

        fn try_from(value: Response) -> std::prelude::v1::Result<Self, Self::Error> {
            if let Some(loc) = value.location {
                Ok(Location {
                    latitude: loc.lat,
                    longitude: loc.lng,
                    accuracy: value.accuracy.unwrap_or_default(),
                })
            } else {
                Err(anyhow!("could not resolve location"))
            }
        }
    }

    pub async fn resolve_location(aps: &[Bssid], api_key: &str) -> Result<Location> {
        let req = Request {
            wifi_access_points: aps.iter().map(WifiAp::from).collect(),
        };
        let client = get_client();
        let res = client
            .post("https://www.googleapis.com/geolocation/v1/geolocate")
            .header("X-Goog-Api-Key", api_key)
            .json(&req)
            .send()
            .await?
            .json::<Response>()
            .await?;

        res.try_into()
    }
}
