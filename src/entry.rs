#[derive(Debug, Clone, serde::Deserialize)]
pub struct Entry {
    pub title_id: String,
    pub short: String,
    pub long: String,
    pub publisher: String,
}