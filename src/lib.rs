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
use log::debug;

mod activity_utils;
pub mod available_titles;
pub mod info;
mod script_runner;

use activity_utils::merge_activities;
use available_titles::AvailableTitles;
pub use info::UserInfo;
use script_runner::ScriptRunner;

pub struct ActivityGenerator {
    script_runner: ScriptRunner,

    assets_base_url: String,
    mii_generator_server: String,
    available_titles: AvailableTitles,
}

impl ActivityGenerator {
    /// Create a new `ActivityGenerator`.
    ///
    /// `script_dir` — directory with `<title_id>.lua` scripts.
    /// `assets_base_url` — base URL for game icon images.
    /// `mii_generator_server` — base URL for Mii images.
    ///
    /// On construction, this fetches `{assets_base_url}/available_titles.json`
    /// to know which title IDs have icons. If the fetch fails, a warning is logged
    /// and every title will get the fallback `3ds_logo.png`.
    #[must_use]
    pub async fn new(
        script_dir: &str,
        assets_base_url: &str,
        mii_generator_server: &str,
    ) -> Self {
        Self {
            script_runner: ScriptRunner::new(script_dir),
            assets_base_url: assets_base_url.trim_end_matches('/').to_string(),
            mii_generator_server: mii_generator_server.trim_end_matches('/').to_string(),
            available_titles: AvailableTitles::load(assets_base_url).await,
        }
    }

    fn get_image_url(&self, title_id: &str) -> String {
        if title_id == "0000000000000000" {
            format!("{}/specials/home_menu.png", self.assets_base_url)
        } else if self.available_titles.contains(title_id) {
            format!("{}/{}/icon.png", self.assets_base_url, title_id)
        } else {
            format!("{}/specials/3ds_logo.png", self.assets_base_url)
        }
    }

    /// Build a Discord Activity for the given game. Runs a Lua script if `extra_info` is set.
    pub async fn build_activity(
        &self,
        user_info: &info::UserInfo,
        game_info: &info::GameInfo,
        extra_info: &Option<String>,
    ) -> Activity {
        let image_url = self.get_image_url(&game_info.title_id);
        debug!("evt=activity_icon_url url={image_url}");

        // Build the default activity first
        let default_act = Activity::new()
            .set_name(&game_info.name)
            .set_activity_type(ActivityType::Playing)
            .set_details(&game_info.publisher)
            .set_state("Via 3ds-presence.top") // Credit to the project, please don't remove it as part of the AGPL license
            .set_assets(Assets::new().set_large_image(&image_url));

        // If we have extra_info, try the Lua script runner; otherwise use default
        let mut act = if let Some(extra) = extra_info
            && !extra.is_empty()
            && let Some(script_act) = self
                .script_runner
                .call_script(&game_info.title_id, game_info, extra)
                .await
        {
            // Merge: script values override defaults, but empty fields keep defaults
            merge_activities(&script_act, &default_act)
        } else {
            default_act
        };

        // Apply Mii overlay (small image) if available and non-empty
        if let Some(mii) = &user_info.mii
            && !mii.is_empty()
            && act.assets().small_image().is_empty()
        {
            let assets_with_mii = act
                .assets()
                .set_small_image(&format!("{}/{}", self.mii_generator_server, mii))
                .set_small_text(
                    &user_info
                        .mii_name
                        .clone()
                        .unwrap_or_else(|| "Unknown Mii".into()),
                );
            debug!("evt=activity_mii_url url={}", assets_with_mii.small_image());
            act = act.set_assets(assets_with_mii);
        }

        act
    }
}