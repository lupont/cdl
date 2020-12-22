use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub authors: Vec<Author>,

    #[serde(rename = "summary")]
    pub description: String,

    pub categories: Vec<Category>,

    #[serde(rename = "gameVersionLatestFiles")]
    pub game_files: Vec<GameFile>,
    pub id: u32,
    pub name: String,
    pub website_url: String,
}

impl SearchResult {
    pub fn get_file_by_version(&self, version: &str) -> Option<&GameFile> {
        self.game_files.iter().find(|f| f.game_version == version)
    }

    pub fn author_names(&self) -> String {
        // TODO: implement join() for Vec<Author>
        self.authors
            .iter()
            .fold(String::new(), |mut a, c| {
                a.push_str(&c.name);
                a.push_str(", ");
                a
            })
            .strip_suffix(", ")
            .expect("expected author string to end with a semicolon")
            .into()
    }

    pub fn is_fabric(&self) -> bool {
        self.categories
            .iter()
            .any(|c| c.category_id == Category::fabric_id())
    }
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub category_id: u32,
    pub name: String,
    pub url: String,
}

impl Category {
    pub fn fabric_id() -> u32 {
        4780
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFile {
    pub game_version: String,
    pub project_file_id: u32,
    pub project_file_name: String,
    pub file_type: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInfo {
    pub id: u32,
    pub display_name: String,
    pub file_name: String,
    pub download_url: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub addon_id: u32,
    #[serde(rename = "type")]
    pub dep_type: u32,
}
