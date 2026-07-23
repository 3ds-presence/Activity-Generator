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


use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use discord_social_rpc::Activity;
use log::{debug, warn};
use mlua::{Lua, LuaOptions, StdLib};
use tokio::sync::Mutex;

use crate::info::GameInfo;

mod converter;
mod environment;
mod executor;

use executor::Executor;

/// Default maximum number of Lua VMs in the pool.
const DEFAULT_POOL_MAX: usize = 64;

/// Maximum execution time for a single Lua script (500 ms).
const LUA_TIMEOUT: Duration = Duration::from_millis(500);

/// Pool of Lua VMs for executing game-specific activity scripts.
///
/// VMs are recycled after use (globals cleared) to prevent memory leaks.
/// Capped at `max_pool`; creates temporary VMs when the pool is empty.
pub struct ScriptRunner {
    script_dir: PathBuf,
    pool: Arc<Mutex<Vec<Lua>>>,
    max_pool: usize,
}

impl ScriptRunner {
    /// Create a new `ScriptRunner`.
    ///
    /// `script_dir` — directory containing `<title_id>.lua` scripts.
    /// `pool_max` — maximum Lua VMs to keep in the pool. Use 0 for default (64).
    pub fn new(script_dir: &str, pool_max: usize) -> Self {
        let max = if pool_max > 0 { pool_max } else { DEFAULT_POOL_MAX };
        Self {
            script_dir: PathBuf::from(script_dir),
            pool: Arc::new(Mutex::new(Vec::with_capacity(max))),
            max_pool: max,
        }
    }

    /// Run the Lua script for `title_id` and return an `Activity`.
    ///
    /// Returns `None` if the script does not exist, fails, triggers fallback,
    /// or exceeds the 500ms timeout.
    pub async fn call_script(
        &self,
        title_id: &str,
        game_info: &GameInfo,
        extra_info: &str,
    ) -> Option<Activity> {
        let executor = Executor::new(&self.script_dir, title_id);
        let script_content = executor.read_script()?;

        let lua = self.acquire().await;
        let lua_clone = lua.clone();
        let executor_clone = executor;
        let game_info_clone = game_info.clone();
        let extra_info_clone = extra_info.to_string();
        let script_content_clone = script_content;

        // Run the Lua build in spawn_blocking (mlua is synchronous) with a timeout.
        let result = tokio::time::timeout(LUA_TIMEOUT, tokio::task::spawn_blocking(move || {
            executor_clone.run_build(&lua_clone, &script_content_clone, &game_info_clone, &extra_info_clone)
        })).await;

        match result {
            Ok(Ok(Some(activity))) => {
                // Success — recycle the VM and return the activity
                self.recycle(lua).await;
                Some(activity)
            }
            Ok(Ok(None)) => {
                // Script returned nil or fallback — recycle the VM
                self.recycle(lua).await;
                None
            }
            Ok(Err(err)) => {
                // spawn_blocking panicked — log and discard the VM (may be in a bad state)
                warn!("Lua spawn_blocking panicked for {}: {:?}", title_id, err);
                None
            }
            Err(_) => {
                // Timeout exceeded — discard the VM (may be stuck in an infinite loop)
                warn!("Lua script timeout ({}ms) for {}", LUA_TIMEOUT.as_millis(), title_id);
                None
            }
        }
    }

    /// Acquire a Lua VM from the pool, or create a fresh one with a whitelist
    /// of safe standard libraries.
    async fn acquire(&self) -> Lua {
        let mut pool = self.pool.lock().await;
        if let Some(state) = pool.pop() {
            debug!("Reusing Lua VM from pool ({} remaining)", pool.len());
            state
        } else {
            debug!("Creating new Lua VM (pool empty)");
            Lua::new_with(
                StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::COROUTINE | StdLib::UTF8,
                LuaOptions::default(),
            )
            .expect("Failed to create sandboxed Lua VM")
        }
    }

    /// Reset globals and return a Lua VM to the pool.
    async fn recycle(&self, lua: Lua) {
        if let Err(e) = lua.globals().clear() {
            log::warn!("Failed to clear Lua globals: {}", e);
        }
        let mut pool = self.pool.lock().await;
        if pool.len() < self.max_pool {
            pool.push(lua);
        }
    }
}