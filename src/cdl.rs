use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};
use structopt::StructOpt;

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
            &ModLoader::Forge => "Forge".into(),
            &ModLoader::Fabric => "Fabric".into(),
            &ModLoader::Both => "Forge/Fabric".into(),
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
                &SortType::TotalDownloads => "TotalDownloads",
                &SortType::Popularity => "Popularity",
                &SortType::Name => "Name",
                &SortType::LastUpdated => "LastUpdated",
                &SortType::DateCreated => "DateCreated",
            }
        )
    }
}

fn parse_query(src: &str) -> String {
    src.replace(" ", "%20").into()
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cdl",
    version = "0.2.0",
    about = "A command-line utility for downloading Minecraft mods."
)]
pub struct Cdl {
    #[structopt(short = "l", long, default_value = "forge", possible_values = &["forge", "fabric", "both"], help = "The mod loader to use when searching.")]
    pub mod_loader: ModLoader,

    #[structopt(
        short = "v",
        long,
        default_value = "1.16.4",
        help = "The version of the game."
    )]
    pub game_version: String,

    #[structopt(short, long, default_value = "popularity", possible_values = &["downloads", "popularity", "name", "updated", "created"], help = "The ordering of search results.")]
    pub sort: SortType,

    #[structopt(
        short,
        long,
        default_value = "9",
        help = "The amount of search results to show."
    )]
    pub amount: u8,

    #[structopt(short, long)]
    pub github_repo: bool,

    #[structopt(parse(from_str = parse_query), help = "The query to search for.")]
    pub query: String,
}
