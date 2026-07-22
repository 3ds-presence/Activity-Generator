use std::path::PathBuf;
use std::sync::Arc;

use discord_social_rpc::Activity;
use log::debug;
use mlua::Lua;
use tokio::sync::Mutex;

use crate::info::GameInfo;

mod converter;
mod environment;
mod executor;

use executor::Executor;

/// Default maximum number of Lua VMs in the pool.
const DEFAULT_POOL_MAX: usize = 64;

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
    /// Returns `None` if the script does not exist, fails, or triggers fallback.
    pub async fn call_script(
        &self,
        title_id: &str,
        game_info: &GameInfo,
        extra_info: &str,
    ) -> Option<Activity> {
        let executor = Executor::new(&self.script_dir, title_id);
        let script_content = executor.read_script()?;

        let lua = self.acquire().await;
        let activity = executor.run_build(&lua, &script_content, game_info, extra_info);
        self.recycle(lua).await;

        activity
    }

    /// Acquire a Lua VM from the pool, or create a fresh one.
    async fn acquire(&self) -> Lua {
        let mut pool = self.pool.lock().await;
        if let Some(state) = pool.pop() {
            debug!("Reusing Lua VM from pool ({} remaining)", pool.len());
            state
        } else {
            debug!("Creating new Lua VM (pool empty)");
            Lua::new()
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