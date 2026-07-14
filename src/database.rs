use std::path::Path;

use log::info;

use crate::{entry::Entry, error::Error};

pub(crate) struct HashMapDatabase {
    entries: std::collections::HashMap<String, Entry>,
}

impl HashMapDatabase {
    fn new() -> Self {
        HashMapDatabase {
            entries: std::collections::HashMap::new(),
        }
    }
}

pub async fn create_database() -> HashMapDatabase {
    HashMapDatabase::new()
}

pub async fn load_game_data(db: &mut HashMapDatabase, info_dir: &str) -> Result<u64, Error> {
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
        let meta: Entry = serde_json::from_str(&content)?;

        db.entries.insert(title_id.to_string(), meta);
        count += 1;
    }

    info!("GameDatabase loaded {} games from {}", count, info_dir);
    Ok(count)
}

pub async fn find_game(db: &HashMapDatabase, title_id: &str) -> Result<Option<Entry>, Error> {
    Ok(db.entries.get(title_id).cloned())
}