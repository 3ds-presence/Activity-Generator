use std::path::Path;

use log::info;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend,
    EntityTrait, QueryFilter, Set, Statement,
};
use sea_orm::sea_query::{SqliteQueryBuilder, TableCreateStatement};

use crate::error::Error;
use crate::models::{ActiveModel, Column, Entity, MetaJson, Model};

/// Initializes an in-memory SQLite database and creates the `game_info` table.
pub async fn create_database() -> Result<DatabaseConnection, Error> {
    let db = Database::connect("sqlite::memory:").await?;

    let stmt: TableCreateStatement =
        sea_orm::Schema::new(DbBackend::Sqlite).create_table_from_entity(Entity);
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        stmt.to_string(SqliteQueryBuilder),
    ))
    .await?;

    Ok(db)
}

/// Scans the `info_dir` directory for `meta.json` files and inserts them
/// into the database. Returns the number of games loaded.
pub async fn load_game_data(
    db: &DatabaseConnection,
    info_dir: &str,
) -> Result<u64, Error> {
    let info_path = Path::new(info_dir);
    let mut count = 0u64;

    if !info_path.is_dir() {
        return Ok(count);
    }

    let mut entries: Vec<_> = std::fs::read_dir(info_path)?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let dir_name = entry.file_name();
        let title_id = dir_name.to_string_lossy();
        let meta_path = entry.path().join("meta.json");

        if !meta_path.is_file() {
            continue;
        }

        let content = std::fs::read_to_string(&meta_path)?;
        let meta: MetaJson = serde_json::from_str(&content)?;

        let active_model = ActiveModel {
            title_id: Set(title_id.to_string()),
            short: Set(meta.short),
            long: Set(meta.long),
            publisher: Set(meta.publisher),
        };
        active_model.insert(db).await?;
        count += 1;
    }

    info!("GameDatabase loaded {} games from {}", count, info_dir);
    Ok(count)
}

/// Queries the database for a game by its title ID.
pub async fn find_game(
    db: &DatabaseConnection,
    title_id: &str,
) -> Result<Option<Model>, sea_orm::DbErr> {
    Entity::find()
        .filter(Column::TitleId.eq(title_id))
        .one(db)
        .await
}