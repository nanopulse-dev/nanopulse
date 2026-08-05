use anyhow::Result;
use mlua::{Function, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tracing::info;

use crate::lua;

#[derive(Default, Debug, Deserialize, Clone)]
pub struct Vendor {
    pub name: String,
}

impl Vendor {
    pub fn load(id: &[u8; 4]) -> Result<Self> {
        let id = hex::encode(id);
        info!(id = %id, "Loading vendor information");

        let lua = lua::get()?;
        let require: Function = lua.globals().get("require")?;
        let vendor: Table = require.call(format!("vendors.{}", id))?;

        Ok(lua.from_value(Value::Table(vendor))?)
    }
}

pub fn get_vendor(id: &[u8; 4]) -> Result<Vendor> {
    Vendor::load(id)
}
