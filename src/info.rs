#[derive(Debug, Clone, Default)]
pub struct UserInfo {
    pub mii_name: Option<String>,
    pub mii: Option<String>,
}

pub struct GameInfo {
    pub title_id: String,
    pub name: String,
    pub publisher: String,
}