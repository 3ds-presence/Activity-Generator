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

use std::time::Duration;

use discord_social_rpc::Activity;
use log::{debug, warn};
use mlua::{Lua, LuaOptions, StdLib};

use crate::info::GameInfo;
use crate::script_reader::ScriptReader;

mod converter;
mod environment;
mod executor;

use executor::Executor;

/// Maximum execution time for a single Lua script (500 ms).
const LUA_TIMEOUT: Duration = Duration::from_millis(500);

/// Executes game-specific activity scripts.
///
/// A fresh Lua VM is created for every script invocation, then dropped
/// automatically — no pooling, no recycling bugs.
pub struct ScriptRunner;

impl ScriptRunner {
    /// Create a new `ScriptRunner`.
    pub const fn new() -> Self {
        Self
    }

    /// Run the Lua script for `title_id` and return an `Activity`.
    ///
    /// `script_dir` — directory containing `<title_id>/script.lua` scripts.
    ///
    /// Returns `None` if the script does not exist, fails, triggers fallback,
    /// or exceeds the 500ms timeout.
    pub async fn call_script(
        &self,
        script_reader: &ScriptReader,
        title_id: &str,
        game_info: &GameInfo,
        extra_info: &str,
    ) -> Option<Activity> {
        let script_content = script_reader.read_lua_script(title_id)?;
        let executor = Executor::new(script_reader.dir(), title_id);

        let lua = Self::acquire();
        let script_content_clone = script_content;
        let game_info_clone = game_info.clone();
        let extra_info_clone = extra_info.to_string();

        let result = tokio::time::timeout(
            LUA_TIMEOUT,
            tokio::task::spawn_blocking(move || {
                executor.run_build(&lua, &script_content_clone, &game_info_clone, &extra_info_clone)
            }),
        )
        .await;

        match result {
            Ok(Ok(Some(activity))) => Some(activity),
            Ok(Ok(None)) => None,
            Ok(Err(err)) => {
                warn!("evt=lua_script_panicked title_id={title_id} error={err:?}");
                None
            }
            Err(_) => {
                warn!(
                    "evt=lua_script_timeout title_id={title_id} timeout_ms={}",
                    LUA_TIMEOUT.as_millis()
                );
                None
            }
        }
    }

    /// Create a fresh sandboxed Lua VM.
    fn acquire() -> Lua {
        debug!("evt=lua_vm_created");
        Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::COROUTINE | StdLib::UTF8,
            LuaOptions::default(),
        )
        .expect("Failed to create sandboxed Lua VM")
    }
}