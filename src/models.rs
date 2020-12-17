use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    id: u32,
    pub name: String,
    authors: Vec<Author>,
    pub website_url: String,

    #[serde(rename = "gameVersionLatestFiles")]
    pub game_files: Vec<GameFile>,
}

#[derive(Debug, Deserialize)]
pub struct Author {
    name: String,
    url: String,
    id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFile {
    pub game_version: String,
    project_file_id: u32,
    project_file_name: String,
    file_type: u8,
}

