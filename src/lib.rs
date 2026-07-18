use discord_social_rpc::{Activity, ActivityType, Assets};

pub mod info;

pub use info::UserInfo;
use log::debug;

pub struct ActivityGenerator {
    #[allow(dead_code)] // Will be used later
    script_dir: String,

    assets_base_url: String,
    mii_generator_server: String,
}

impl ActivityGenerator {
    /// Create a new `ActivityGenerator`
    ///
    /// `script_dir` is the directory containing game scripts (title_id/script.lua). 
    /// Will be used later for more advanced activity generation.
    ///
    /// `assets_base_url` is the base URL for game icons, e.g.
    /// `"http://localhost:8080/imgs/"`. The final image URL will be
    /// `{assets_base_url}{title_id}/icon.png`.
    ///
    /// `mii_generator_server` is the URL of the Mii generator server, e.g.
    /// `"http://localhost:8080/miis/"`.
    pub fn new(script_dir: &str, assets_base_url: &str, mii_generator_server: &str) -> Self {
        Self {
            script_dir: script_dir.trim_end_matches('/').to_string(), // Will be used later
            assets_base_url: assets_base_url.trim_end_matches('/').to_string(),
            mii_generator_server: mii_generator_server.trim_end_matches('/').to_string(),
        }
    }

    /// Build a Discord `Activity` for the given `title_id`.
    ///
    /// If the title is found in the catalogue, the activity is populated
    /// with the game's metadata. Otherwise a fallback "Unknown game"
    /// activity is returned.
    pub async fn build_activity(
        &self,
        user_info: &info::UserInfo,
        game_info: &info::GameInfo,
    ) -> Activity {
        let image_url = format!("{}/{}/icon.png", self.assets_base_url, game_info.title_id);
        debug!("Game icon URL: {}", image_url);
        
        let mut act = Activity::new()
            .set_name(&game_info.name)
            .set_activity_type(ActivityType::Playing)
            .set_details(&game_info.publisher)
            .set_state("Via 3ds-presence.top")
            .set_assets(Assets::new().set_large_image(&image_url));

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
