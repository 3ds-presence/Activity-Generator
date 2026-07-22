use std::path::PathBuf;

use discord_social_rpc::Activity;
use log::{debug, warn};
use mlua::{Function, Lua, Value};

use crate::info::GameInfo;

use super::converter;
use super::environment;
use super::environment::is_fallback_error;

/// Handles the full lifecycle of a single script execution.
pub struct Executor {
    script_path: PathBuf,
}

impl Executor {
    /// Create a new executor for the given `title_id` in `script_dir`.
    pub fn new(script_dir: &PathBuf, title_id: &str) -> Self {
        Self {
            script_path: script_dir.join(format!("{}.lua", title_id)),
        }
    }

    /// Read the script file from disk. Returns `None` if not found or unreadable.
    pub fn read_script(&self) -> Option<String> {
        if !self.script_path.exists() {
            debug!("No Lua script found at {:?}, using fallback", self.script_path);
            return None;
        }
        match std::fs::read_to_string(&self.script_path) {
            Ok(c) => Some(c),
            Err(e) => {
                warn!("Failed to read Lua script {:?}: {}", self.script_path, e);
                None
            }
        }
    }

    /// Full pipeline: prepare env, load script, call build, convert result.
    pub fn run_build(
        &self,
        lua: &Lua,
        script_content: &str,
        game_info: &GameInfo,
        extra_info: &str,
    ) -> Option<Activity> {
        if !environment::prepare(lua, game_info, extra_info) {
            return None;
        }
        if !self.load_script(lua, script_content) {
            return None;
        }
        let value = self.call_build(lua)?;
        converter::value_to_activity(value, &self.script_path)
    }

    /// Load and execute the script content. Returns `true` on success.
    fn load_script(&self, lua: &Lua, script_content: &str) -> bool {
        match lua.load(script_content).exec() {
            Ok(_) => true,
            Err(e) => {
                if is_fallback_error(&e) {
                    debug!("Script {} requested fallback", self.script_path.display());
                } else {
                    warn!("Lua script {} execution error: {}", self.script_path.display(), e);
                }
                false
            }
        }
    }

    /// Call the `build` function and return its value.
    fn call_build(&self, lua: &Lua) -> Option<Value> {
        let build_fn: Function = match lua.globals().get("build") {
            Ok(f) => f,
            Err(e) => {
                warn!("Lua script {} has no `build` function: {}", self.script_path.display(), e);
                return None;
            }
        };

        let game_table: mlua::Table = lua.globals().get("game_info").unwrap();
        let extra_table: mlua::Table = lua.globals().get("extra_info").unwrap();

        match build_fn.call::<Value>((game_table, extra_table)) {
            Ok(val) => Some(val),
            Err(e) => {
                if is_fallback_error(&e) {
                    debug!("Script {} requested fallback", self.script_path.display());
                } else {
                    warn!("Lua build() call failed for {}: {}", self.script_path.display(), e);
                }
                None
            }
        }
    }
}