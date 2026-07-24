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

use discord_social_rpc::{Activity, ActivityType, Assets};

mod activity_utils;
pub mod info;
mod script_runner;

use activity_utils::merge_activities;
pub use info::UserInfo;
use log::debug;
use script_runner::ScriptRunner;

pub struct ActivityGenerator {
    script_runner: ScriptRunner,

    assets_base_url: String,
    mii_generator_server: String,
}

impl ActivityGenerator {
    /// `script_dir` — directory with `<title_id>.lua` scripts.
    /// `assets_base_url` — base URL for `{title_id}/icon.png`.
    /// `mii_generator_server` — base URL for Mii images.
    /// `lua_pool_max` — Lua VM pool size (0 = default 64).
    pub fn new(
        script_dir: &str,
        assets_base_url: &str,
        mii_generator_server: &str,
        lua_pool_max: usize,
    ) -> Self {
        Self {
            script_runner: ScriptRunner::new(script_dir, lua_pool_max),
            assets_base_url: assets_base_url.trim_end_matches('/').to_string(),
            mii_generator_server: mii_generator_server.trim_end_matches('/').to_string(),
        }
    }

    /// Build a Discord Activity for the given game. Runs a Lua script if `extra_info` is set.
    pub async fn build_activity(
        &self,
        user_info: &info::UserInfo,
        game_info: &info::GameInfo,
        extra_info: &Option<String>,
    ) -> Activity {
        let image_url = format!("{}/{}/icon.png", self.assets_base_url, game_info.title_id);
        debug!("Game icon URL: {}", image_url);

        // Build the default activity first
        let default_act = Activity::new()
            .set_name(&game_info.name)
            .set_activity_type(ActivityType::Playing)
            .set_details(&game_info.publisher)
            .set_state("Via 3ds-presence.top")
            .set_assets(Assets::new().set_large_image(&image_url));

        let mut act = default_act.clone();

        // If we have extra_info, try the Lua script runner
        if let Some(extra) = extra_info {
            if !extra.is_empty() {
                if let Some(script_act) = self
                    .script_runner
                    .call_script(&game_info.title_id, game_info, extra)
                    .await
                {
                    // Merge: script values override defaults, but empty fields keep defaults
                    act = merge_activities(&script_act, &default_act);
                }
            }
        }

        // Apply Mii overlay (small image) if available
        if let Some(mii) = &user_info.mii {
            let assets_with_mii = act
                .assets()
                .set_small_image(&format!("{}{}", self.mii_generator_server, mii))
                .set_small_text(&user_info.mii_name.clone().unwrap_or("Unknown Mii".into()));
            debug!("Mii image URL: {}", assets_with_mii.small_image());
            act = act.set_assets(assets_with_mii);
        }

        act
    }
}
