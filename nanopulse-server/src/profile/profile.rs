use std::collections::HashMap;

use anyhow::Result;
use mlua::{Function, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::lua;

#[derive(Default, Debug, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub description: String,
}

pub type ComponentSchema = HashMap<String, Component>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub component: ComponentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    #[serde(rename = "sensor")]
    Sensor(SensorType),
    #[serde(rename = "tracker")]
    Tracker(TrackerType),
    #[serde(rename = "switch")]
    Switch(NoProps),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorType {
    #[serde(rename = "None")]
    None,
    #[serde(rename = "temperature")]
    Temperature(TemperatureProperties),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerType {
    pub source_type: LocationSourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureProperties {
    pub unit: TemperatureUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemperatureUnit {
    C,
    F,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationSourceType {
    #[serde(rename = "WIFI")]
    Wifi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoProps {}

pub fn get_profile(
    vendor_id: &[u8; 4],
    profile_id: &[u8; 2],
    version_id: &[u8; 2],
) -> Result<Profile> {
    let vendor_id = hex::encode(vendor_id);
    let profile_id = hex::encode(profile_id);
    let version_id = hex::encode(version_id);
    info!(vendor_id = %vendor_id, profile_id = %profile_id, version_id = %version_id, "Loading profile information");

    let lua = lua::get()?;
    let require: Function = lua.globals().get("require")?;
    let profile: Table = require.call(format!(
        "vendors.{}.profiles.{}_{}",
        vendor_id, profile_id, version_id
    ))?;

    Ok(Profile {
        name: profile.get("name")?,
        description: profile.get("description")?,
    })
}

pub async fn decode_telemetry(profile_mod: &str, pl: &[u8]) -> Result<serde_json::Value> {
    let lua = lua::get()?;
    let function = lua::get_function(&lua, profile_mod, "decode_telemetry")?;
    let value: Value = function.call_async(pl).await?;

    Ok(serde_json::to_value(value)?)
}

pub fn encode_state(profile_mod: &str, pl: &serde_json::Value) -> Result<Vec<u8>> {
    let lua = lua::get()?;
    let function = lua::get_function(&lua, profile_mod, "encode_state")?;
    let value: Value = function.call(lua.to_value(pl)?)?;

    Ok(lua.from_value(value)?)
}

pub fn decode_state(profile_mod: &str, pl: &[u8]) -> Result<serde_json::Value> {
    let lua = lua::get()?;
    let function = lua::get_function(&lua, profile_mod, "decode_state")?;
    let value: Value = function.call(pl)?;

    Ok(serde_json::to_value(value)?)
}

pub fn get_telemetry_schema(profile_mod: &str) -> Result<ComponentSchema> {
    let lua = lua::get()?;
    let function: Function = lua::get_function(&lua, profile_mod, "telemetry_schema")?;
    let value: Value = function.call(())?;

    Ok(lua.from_value(value)?)
}

pub fn get_state_schema(profile_mod: &str) -> Result<ComponentSchema> {
    let lua = lua::get()?;
    let function: Function = lua::get_function(&lua, profile_mod, "state_schema")?;
    let value: Value = function.call(())?;

    Ok(lua.from_value(value)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test;
    use serde_json::json;

    #[tokio::test]
    async fn test_decode_telemetry_error() {
        let _guard = test::prepare().await;
        let res = decode_telemetry("vendors.00000000.profiles.0000_0000", &[]).await;
        assert!(res.is_err());
        assert!(
            res.err()
                .unwrap()
                .to_string()
                .contains("invalid payload length")
        );
    }

    #[tokio::test]
    async fn test_decode_telemetry() {
        let _guard = test::prepare().await;
        let a = decode_telemetry("vendors.00000000.profiles.0000_0000", &[150, 0])
            .await
            .unwrap();
        let b = decode_telemetry("vendors.00000000.profiles.0000_0000", &[106, 255])
            .await
            .unwrap();

        assert_eq!(json!({"temperature": 15}), a);
        assert_eq!(json!({"temperature": -15}), b);
    }

    #[tokio::test]
    async fn test_encode_state() {
        let _guard = test::prepare().await;
        assert_eq!(
            vec![0x01],
            encode_state("vendors.00000000.profiles.0000_0000", &json!({"led": true})).unwrap()
        );
    }

    #[tokio::test]
    async fn test_telemetry_schema() {
        let _guard = test::prepare().await;
        let mut s = ComponentSchema::new();
        s.insert(
            "temperature".into(),
            Component {
                name: "Temperature".into(),
                component: ComponentType::Sensor(SensorType::Temperature(TemperatureProperties {
                    unit: TemperatureUnit::C,
                })),
            },
        );

        assert_eq!(
            json!({
                "temperature": {
                    "name": "Temperature",
                    "component": {
                        "sensor": {
                            "temperature": {
                                "unit": "C"
                            },
                        }
                    }
                }
            }),
            serde_json::to_value(s).unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_telemetry_schema() {
        let _guard = test::prepare().await;
        let schema = get_telemetry_schema("vendors.00000000.profiles.0000_0000").unwrap();
        assert_eq!(
            json!({
                "temperature": {
                    "name": "Temperature",
                    "component": {
                        "sensor": {
                            "temperature": {
                                "unit": "C"
                            },
                        }
                    }
                }
            }),
            serde_json::to_value(schema).unwrap(),
        );
    }
}
