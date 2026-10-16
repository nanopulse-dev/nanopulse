use std::collections::HashMap;

use anyhow::{Context, Result};
use mlua::{Function, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

use crate::errors::Error;
use crate::lua;

#[derive(Default, Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Profile {
    #[serde(skip)]
    lua_module: String,
    pub name: String,
    pub description: String,
    pub telemetry_schema: HashMap<String, Component>,
    pub state_schema: HashMap<String, Component>,
    pub configuration_schema: HashMap<String, Configuration>,
}

impl Profile {
    pub fn get_by_vendor_profile_version(
        vendor_id: &[u8; 4],
        profile_id: &[u8; 2],
        version_id: &[u8; 2],
    ) -> Result<Profile, Error> {
        let vendor_id = hex::encode(vendor_id);
        let profile_id = hex::encode(profile_id);
        let version_id = hex::encode(version_id);

        let lua_module = format!(
            "vendors.{}.profiles.{}_{}",
            vendor_id, profile_id, version_id
        );

        Ok(Self::get_by_mod(&lua_module)?)
    }

    pub fn get_by_mod(lua_module: &str) -> Result<Profile, Error> {
        info!(profile_mod = %lua_module, "loading profile information");

        let lua = lua::get()?;
        let require: Function = lua.globals().get("require")?;
        let profile: Table = require.call(lua_module)?;

        Ok(Profile {
            name: profile.get("name")?,
            description: profile.get("description").context("get description")?,
            telemetry_schema: lua.from_value(
                profile
                    .get::<Function>("telemetry_schema")
                    .context("get telemetry_schema")?
                    .call(())
                    .context("call telemetry_schema")?,
            )?,
            state_schema: lua.from_value(
                profile
                    .get::<Function>("state_schema")
                    .context("get state_schema")?
                    .call(())
                    .context("call state_schema")?,
            )?,
            configuration_schema: lua.from_value(
                profile
                    .get::<Function>("configuration_schema")
                    .context("get configuration_schema")?
                    .call(())
                    .context("call configuration_schema")?,
            )?,
            lua_module: lua_module.into(),
        })
    }

    pub fn get_lua_module(&self) -> String {
        self.lua_module.clone()
    }

    pub fn get_default_state(&self) -> serde_json::Value {
        let mut out = serde_json::Map::new();

        for (k, v) in self.state_schema.iter() {
            out.insert(
                k.into(),
                match &v.component {
                    ComponentType::Switch(v) => serde_json::Value::Bool(v.default),
                    _ => continue,
                },
            );
        }

        serde_json::Value::Object(out)
    }

    pub fn get_default_configuration(&self) -> serde_json::Value {
        let mut out = serde_json::Map::new();

        for (k, v) in self.configuration_schema.iter() {
            out.insert(
                k.into(),
                match &v.configuration {
                    ConfigurationType::Interval(v) => serde_json::Value::Number(v.default.into()),
                },
            );
        }

        serde_json::Value::Object(out)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Component {
    pub name: String,
    pub component: ComponentType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ComponentType {
    #[serde(rename = "sensor")]
    Sensor(SensorType),
    #[serde(rename = "tracker")]
    Tracker(TrackerType),
    #[serde(rename = "switch")]
    Switch(SwitchType),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum SensorType {
    #[serde(rename = "None")]
    None,
    #[serde(rename = "temperature")]
    Temperature(TemperatureProperties),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrackerType {
    pub source_type: LocationSourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemperatureProperties {
    pub unit: TemperatureUnit,
    pub min: isize,
    pub max: isize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum TemperatureUnit {
    C,
    F,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum LocationSourceType {
    #[serde(rename = "WIFI")]
    Wifi,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SwitchType {
    pub default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Configuration {
    pub name: String,
    pub configuration: ConfigurationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ConfigurationType {
    #[serde(rename = "interval")]
    Interval(IntervalConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IntervalConfig {
    pub unit: String,
    pub default: usize,
    pub min: usize,
    pub max: usize,
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

pub fn encode_configuration(profile_mod: &str, pl: &serde_json::Value) -> Result<Vec<u8>> {
    let lua = lua::get()?;
    let function = lua::get_function(&lua, profile_mod, "encode_configuration")?;
    let value: Value = function.call(lua.to_value(pl)?)?;

    Ok(lua.from_value(value)?)
}

pub fn decode_configuration(profile_mod: &str, pl: &[u8]) -> Result<serde_json::Value> {
    let lua = lua::get()?;
    let function = lua::get_function(&lua, profile_mod, "decode_configuration")?;
    let value: Value = function.call(pl)?;

    Ok(serde_json::to_value(value)?)
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
        let mut s: HashMap<String, Component> = HashMap::new();
        s.insert(
            "temperature".into(),
            Component {
                name: "Temperature".into(),
                component: ComponentType::Sensor(SensorType::Temperature(TemperatureProperties {
                    unit: TemperatureUnit::C,
                    min: -15,
                    max: 80,
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
                                "unit": "C",
                                "min": -15,
                                "max": 80,
                            },
                        }
                    }
                }
            }),
            serde_json::to_value(s).unwrap()
        );
    }

    #[tokio::test]
    async fn test_configuration_schema() {
        let _guard = test::prepare().await;
        let mut s: HashMap<String, Configuration> = HashMap::new();
        s.insert(
            "telemetry_interval".into(),
            Configuration {
                name: "Telemetry interval".into(),
                configuration: ConfigurationType::Interval(IntervalConfig {
                    unit: "s".into(),
                    default: 30,
                    min: 5,
                    max: 300,
                }),
            },
        );

        assert_eq!(
            json!({
                "telemetry_interval": {
                    "name": "Telemetry interval",
                    "configuration": {
                        "interval": {
                            "unit": "s",
                            "default": 30,
                            "min": 5,
                            "max": 300,
                        }
                    }
                }
            }),
            serde_json::to_value(s).unwrap()
        );
    }

    #[tokio::test]
    async fn test_profile() {
        let _guard = test::prepare().await;
        let p = Profile::get_by_mod("vendors.00000000.profiles.0000_0000").unwrap();

        assert_eq!("Temperature sensor", p.name);
        assert_eq!("Sensor that reports the temperature", p.description);

        assert_eq!(
            json!({
                "temperature": {
                    "name": "Temperature",
                    "component": {
                        "sensor": {
                            "temperature": {
                                "unit": "C",
                                "min": -15,
                                "max": 80,
                            },
                        }
                    }
                }
            }),
            serde_json::to_value(&p.telemetry_schema).unwrap(),
        );

        assert_eq!(
            json!({
                "telemetry_interval": {
                    "name": "Telemetry interval",
                    "configuration": {
                        "interval": {
                            "unit": "s",
                            "default": 30,
                            "min": 5,
                            "max": 300,
                        }
                    }
                }
            }),
            serde_json::to_value(&p.configuration_schema).unwrap(),
        );

        assert_eq!(
            json!({
                "telemetry_interval": 30,
            }),
            serde_json::to_value(p.get_default_configuration()).unwrap(),
        );
    }
}
