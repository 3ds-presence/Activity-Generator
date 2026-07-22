use mlua::{Lua, Result as LuaResult, Table};

use crate::info::GameInfo;

/// Signal used internally to request a graceful fallback to the default activity.
const FALLBACK_SIGNAL: &str = "__FALLBACK__";

/// Returns `true` if the error is a normal fallback signal.
pub fn is_fallback_error(err: &mlua::Error) -> bool {
    err.to_string().contains(FALLBACK_SIGNAL)
}

/// Inject `get()`, `optional()`, `hex_to_num()` and `fallback()` helpers into the Lua globals.
pub fn inject_helpers(lua: &Lua) -> LuaResult<()> {
    // `get(key)` — get a value from extra_info or trigger a clean fallback.
    let get_fn = lua.create_function(|ctx, key: String| {
        let extra: Table = ctx.globals().get("extra_info")?;
        match extra.get::<String>(key.clone()) {
            Ok(val) => Ok(val),
            Err(_) => Err(mlua::Error::runtime(format!(
                "{FALLBACK_SIGNAL}: missing required extra_info key '{key}'"
            ))),
        }
    })?;
    lua.globals().set("get", get_fn)?;

    // `optional(key)` — get a value or return nil if the key is missing.
    let optional_fn = lua.create_function(|ctx, key: String| {
        let extra: Table = ctx.globals().get("extra_info")?;
        match extra.get::<String>(key) {
            Ok(val) => Ok(Some(val)),
            Err(_) => Ok(None::<String>),
        }
    })?;
    lua.globals().set("optional", optional_fn)?;

    // `fallback()` — explicitly request the default activity.
    // Call this when the script detects that extra_info data
    // is insufficient to build a custom presence.
    lua.globals().set(
        "fallback",
        lua.create_function(|_lua, _: ()| -> Result<(), mlua::Error> {
            Err(mlua::Error::runtime(format!(
                "{FALLBACK_SIGNAL}: script called fallback()"
            )))
        })?,
    )?;

    // `hex_to_num(hex_str)` — convert a hex string to a number.
    // Example: hex_to_num(get("004FE6E0")) or hex_to_num("1C") → 28.
    let hex_to_num_fn = lua.create_function(|_ctx, hex_str: String| {
        let num = i64::from_str_radix(&hex_str, 16).map_err(|e| {
            mlua::Error::runtime(format!(
                "__FALLBACK__: invalid hex value '{hex_str}': {e}"
            ))
        })?;
        Ok(num)
    })?;
    lua.globals().set("hex_to_num", hex_to_num_fn)?;

    Ok(())
}

/// Parse `extra_info` (key=value&key=value...) into a Lua table and set it as `extra_info`.
pub fn inject_extra_info(lua: &Lua, extra_info: &str) -> LuaResult<()> {
    let table = lua.create_table()?;

    for pair in extra_info.split('&') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        if let Some(eq_pos) = pair.find('=') {
            let key = &pair[..eq_pos];
            let value = &pair[eq_pos + 1..];
            table.set(key.to_string(), value.to_string())?;
        } else {
            table.set(pair.to_string(), String::new())?;
        }
    }

    lua.globals().set("extra_info", table)?;
    Ok(())
}

/// Inject `game_info` as a Lua table.
pub fn inject_game_info(lua: &Lua, game_info: &GameInfo) -> LuaResult<()> {
    let table = lua.create_table()?;
    table.set("title_id", game_info.title_id.clone())?;
    table.set("name", game_info.name.clone())?;
    table.set("publisher", game_info.publisher.clone())?;
    lua.globals().set("game_info", table)?;
    Ok(())
}

/// Prepare the Lua VM with helpers and data. Returns `true` on success.
pub fn prepare(lua: &Lua, game_info: &GameInfo, extra_info: &str) -> bool {
    inject_helpers(lua)
        .and(inject_extra_info(lua, extra_info))
        .and(inject_game_info(lua, game_info))
        .is_ok()
}