use sea_orm::entity::prelude::*;
use serde::Deserialize;

/// SeaORM entity for the in-memory game catalogue.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "game_info")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub title_id: String,
    #[sea_orm(column_type = "Text")]
    pub short: String,
    #[sea_orm(column_type = "Text")]
    pub long: String,
    #[sea_orm(column_type = "Text")]
    pub publisher: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// JSON structure of each `meta.json` file.
#[derive(Debug, Deserialize)]
pub struct MetaJson {
    pub short: String,
    pub long: String,
    pub publisher: String,
}