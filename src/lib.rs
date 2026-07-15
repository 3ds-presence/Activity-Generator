use discord_social_rpc::Activity;
use log::info;

mod activity_builder;
mod database;
mod error;
mod entry;
pub mod user_info;

pub use user_info::UserInfo;
use error::Error;
use database::HashMapDatabase;

/// In-memory game catalogue backed by a HashMap.
///
/// Loads all `meta.json` files from the `info/` directory at startup
/// and provides a `get_activity()` method to build a Discord `Activity`
/// for a given title ID.
pub struct GameDatabase {
    db: HashMapDatabase,
    assets_base_url: String,
    mii_generator_server: String,
}

impl GameDatabase {
    /// Create a new `GameDatabase`, loading all game metadata from
    /// `info_dir` (a path like `"activity_manager/info"`) into an
    /// in-memory HashMap.
    ///
    /// `assets_base_url` is the base URL for game icons, e.g.
    /// `"http://localhost:8080/imgs/"`. The final image URL will be
    /// `{assets_base_url}{title_id}/icon.png`.
    /// 
    /// `mii_generator_server` is the URL of the Mii generator server, e.g.
    /// `"http://localhost:8080/miis/"`. 
    pub async fn new(info_dir: &str, assets_base_url: &str, mii_generator_server: &str) -> Result<Self, Error> {
        let mut db = database::create_database().await;
        let count = database::load_game_data(&mut db, info_dir).await?;
        info!("GameDatabase initialized with {} games", count);

        Ok(Self {
            db,
            assets_base_url: assets_base_url.trim_end_matches('/').to_string(),
            mii_generator_server: mii_generator_server.trim_end_matches('/').to_string(),
        })
    }

    /// Build a Discord `Activity` for the given `title_id`.
    ///
    /// If the title is found in the catalogue, the activity is populated
    /// with the game's metadata. Otherwise a fallback "Unknown game"
    /// activity is returned.
    pub async fn build_activity(&self, title_id: &str, user_info: &user_info::UserInfo) -> Activity {
        let mut act = match database::find_game(&self.db, title_id).await {
            Ok(Some(model)) => activity_builder::build_known_activity(
                title_id,
                &model.short,
                &model.long,
                &model.publisher,
                &self.assets_base_url,
            ),
            _ => activity_builder::build_unknown_activity(title_id)
        };
        if let Some(mii) = &user_info.mii {
            let assets_with_mii = act.assets()
                .set_small_image(&format!("{}/{}.png", self.mii_generator_server, mii))
                .set_small_text(&user_info.mii_name.clone().unwrap_or("Unknown Mii".into()));
            act = act.set_assets(assets_with_mii);
        }
        act
    }
}