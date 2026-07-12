use discord_social_rpc::Activity;
use log::info;
use sea_orm::DatabaseConnection;

mod activity_builder;
mod database;
mod error;
pub mod models;

pub use error::Error;

/// In-memory game catalogue backed by SQLite.
///
/// Loads all `meta.json` files from the `info/` directory at startup
/// and provides a `get_activity()` method to build a Discord `Activity`
/// for a given title ID.
pub struct GameDatabase {
    db: DatabaseConnection,
    assets_base_url: String,
}

impl GameDatabase {
    /// Create a new `GameDatabase`, loading all game metadata from
    /// `info_dir` (a path like `"activity_manager/info"`) into an
    /// in-memory SQLite database.
    ///
    /// `assets_base_url` is the base URL for game icons, e.g.
    /// `"http://localhost:8080/imgs/"`. The final image URL will be
    /// `{assets_base_url}{title_id}/icon.png`.
    pub async fn new(info_dir: &str, assets_base_url: &str) -> Result<Self, Error> {
        let db = database::create_database().await?;
        let count = database::load_game_data(&db, info_dir).await?;
        info!("GameDatabase initialized with {} games", count);

        Ok(Self {
            db,
            assets_base_url: assets_base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Build a Discord `Activity` for the given `title_id`.
    ///
    /// If the title is found in the catalogue, the activity is populated
    /// with the game's metadata. Otherwise a fallback "Unknown game"
    /// activity is returned.
    pub async fn build_activity(&self, title_id: &str) -> Activity {
        match database::find_game(&self.db, title_id).await {
            Ok(Some(model)) => activity_builder::build_known_activity(
                title_id,
                &model.short,
                &model.long,
                &model.publisher,
                &self.assets_base_url,
            ),
            _ => activity_builder::build_unknown_activity(title_id),
        }
    }
}