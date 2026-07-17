#[derive(Debug, Clone, serde::Deserialize)]
pub struct Entry {
    pub short: String,
    pub publisher: String,
}