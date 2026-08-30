// 3DS Presence — Discord Rich Presence for Nintendo 3DS
// Copyright (C) 2026 3DS Presence - LeonLeBreton
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::path::{Path, PathBuf};

use discord_social_rpc::Activity;
use log::{debug, warn};
use mlua::{Function, HookTriggers, Lua, Value};

use crate::info::GameInfo;

use super::converter;
use super::environment;
use super::environment::is_fallback_error;

/// Max VM instructions per script. The 500ms timeout cannot interrupt `spawn_blocking`,
/// so this hook (fires once at the limit) is the real guard against infinite loops.
const LUA_INSTRUCTION_LIMIT: u32 = 2_000_000;

fn install_instruction_hook(lua: &Lua) -> mlua::Result<()> {
    lua.set_hook(
        HookTriggers::new().every_nth_instruction(LUA_INSTRUCTION_LIMIT),
        |_lua, _debug| Err(mlua::Error::runtime("script instruction limit exceeded")),
    )
}

/// Handles the full lifecycle of a single script execution.
pub struct Executor {
    script_path: PathBuf,
}

impl Executor {
    /// Create a new executor for the given `title_id` in `script_dir`.
    pub fn new(script_dir: &Path, title_id: &str) -> Self {
        Self {
            script_path: script_dir.join(format!("{title_id}/script.lua")),
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
            warn!(
                "evt=lua_prepare_failed path={}",
                self.script_path.display()
            );
            return None;
        }

        // Install an instruction-count hook so an infinite loop (e.g. `while true`) is
        // stopped even though tokio::time::timeout cannot interrupt spawn_blocking.
        if let Err(e) = install_instruction_hook(lua) {
            warn!(
                "evt=lua_hook_install_failed path={} error={e}",
                self.script_path.display()
            );
            return None;
        }

        if !self.load_script(lua, script_content) {
            return None;
        }
        let value = self.call_build(lua)?;
        converter::value_to_activity(value, &self.script_path)
    }

    /// Log a script error, silently if it's a fallback request.
    fn log_script_error(&self, stage: &str, e: &mlua::Error) {
        if is_fallback_error(e) {
            debug!(
                "evt=lua_fallback_requested path={} stage={stage}",
                self.script_path.display()
            );
        } else {
            warn!(
                "evt=lua_script_error path={} stage={stage} error={}",
                self.script_path.display(),
                e
            );
        }
    }

    /// Load and execute the script content. Returns `true` on success.
    fn load_script(&self, lua: &Lua, script_content: &str) -> bool {
        match lua.load(script_content).exec() {
            Ok(()) => true,
            Err(e) => {
                self.log_script_error("execution", &e);
                false
            }
        }
    }

    /// Call the `build` function and return its value.
    fn call_build(&self, lua: &Lua) -> Option<Value> {
        let build_fn: Function = match lua.globals().get("build") {
            Ok(f) => f,
            Err(e) => {
                warn!(
                    "evt=lua_build_missing path={} error={e}",
                    self.script_path.display()
                );
                return None;
            }
        };

        let game_table: mlua::Table = lua.globals().get("game_info").unwrap();
        let extra_table: mlua::Table = lua.globals().get("extra_info").unwrap();

        match build_fn.call::<Value>((game_table, extra_table)) {
            Ok(val) => Some(val),
            Err(e) => {
                self.log_script_error("build()", &e);
                None
            }
        }
    }
}
