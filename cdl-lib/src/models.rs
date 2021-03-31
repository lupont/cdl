use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModLoader {
    Forge,
    Fabric,
    Both,
}

impl FromStr for ModLoader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "forge" => Ok(Self::Forge),
            "fabric" => Ok(Self::Fabric),
            "both" => Ok(Self::Both),

            s => Err(format!("'{}' not a valid mod loader", s)),
        }
    }
}

impl ToString for ModLoader {
    fn to_string(&self) -> String {
        match self {
            Self::Forge => "Forge".into(),
            Self::Fabric => "Fabric".into(),
            Self::Both => "Forge/Fabric".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SortType {
    TotalDownloads,
    Popularity,
    Name,
    LastUpdated,
    DateCreated,
}

impl FromStr for SortType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "downloads" => Ok(Self::TotalDownloads),
            "popularity" => Ok(Self::Popularity),
            "name" => Ok(Self::Name),
            "updated" => Ok(Self::LastUpdated),
            "created" => Ok(Self::DateCreated),

            s => Err(format!("'{}' not a valid sort type", s)),
        }
    }
}

impl Display for SortType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::TotalDownloads => "TotalDownloads",
                Self::Popularity => "Popularity",
                Self::Name => "Name",
                Self::LastUpdated => "LastUpdated",
                Self::DateCreated => "DateCreated",
            }
        )
    }
}

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
        self.game_files.iter().find(|&f| f.game_version == version)
    }

    pub fn author_names(&self) -> String {
        // TODO: implement join() for Vec<Author>
        format!(
            "{} {}",
            self.authors
                .iter()
                .take(3)
                .fold(String::new(), |mut a, c| {
                    a.push_str(&c.name);
                    a.push_str(", ");
                    a
                })
                .strip_suffix(", ")
                .expect("expected author string to end with a semicolon"),
            if self.authors.len() > 3 { "et al." } else { "" }
        )
    }

    pub fn is_fabric(&self) -> bool {
        self.categories
            .iter()
            .any(|c| c.category_id == Category::FABRIC_ID)
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
    pub const FABRIC_ID: u32 = 4780;
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

impl ModInfo {
    pub fn hard_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies
            .iter()
            .filter(|d| d.dep_type == Dependency::hard_id())
            .collect()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub addon_id: u32,
    #[serde(rename = "type")]
    pub dep_type: u32,
}

impl Dependency {
    pub fn hard_id() -> u32 {
        3
    }
}
