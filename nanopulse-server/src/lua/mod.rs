use std::sync::LazyLock;
use std::sync::Mutex;

use anyhow::{Result, anyhow};
use mlua::{Function, Lua, Table};

mod helpers;
mod wifi;

static LUA: LazyLock<Mutex<Option<Lua>>> = LazyLock::new(|| Mutex::new(None));

pub fn setup(paths: &[String]) -> Result<()> {
    let lua = Lua::new();
    let np_table = lua.create_table()?;

    helpers::decode_u16_le(&np_table)?;
    helpers::decode_i16_le(&np_table)?;
    helpers::decode_u32_le(&np_table)?;
    helpers::decode_i32_le(&np_table)?;

    wifi::wifi_resolve_location(&lua, &np_table)?;

    lua.globals().set("np", np_table)?;

    let package: Table = lua.globals().get("package")?;
    let paths: Vec<String> = paths
        .iter()
        .map(|v| format!("{}/?.lua;{}/?/init.lua", v, v))
        .collect();

    package.set("path", paths.join(";"))?;
    package.set("cpath", "")?;

    *(LUA.lock().unwrap()) = Some(lua);
    Ok(())
}

pub fn get() -> Result<Lua> {
    LUA.lock()
        .unwrap()
        .clone()
        .ok_or_else(|| anyhow!("LUA is not set"))
}

pub fn get_function(lua: &Lua, module: &str, fn_name: &str) -> Result<Function> {
    let require: Function = lua.globals().get("require")?;
    let module: Table = require.call(module)?;
    let function: Function = module.get(fn_name)?;
    Ok(function)
}
