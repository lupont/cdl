use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(Debug)]
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

#[derive(Debug)]
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
    about = "A command-line utility for downloading Minecraft mods."
)]
pub struct Cdl {
    #[structopt(short = "l", long, default_value = "forge")]
    pub mod_loader: ModLoader,

    #[structopt(short = "v", long, default_value = "1.16.4")]
    pub game_version: String,

    #[structopt(short, long, default_value = "popularity")]
    pub sort: SortType,

    #[structopt(short, long, default_value = "9")]
    pub amount: u8,

    #[structopt(parse(from_str = parse_query))]
    pub query: String,
}
