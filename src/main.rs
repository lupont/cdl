use std::{
    str::FromStr,
    fmt::{Display, Formatter},
};
use structopt::StructOpt;

mod models;

#[derive(Debug)]
enum ModLoader {
    Forge,
    Fabric,
    Both,
}

impl FromStr for ModLoader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "forge"  => Ok(Self::Forge),
            "fabric" => Ok(Self::Fabric),
            "both"   => Ok(Self::Both),

            s        => Err(format!("'{}' not a valid mod loader", s)),
        }
    }
}

#[derive(Debug)]
enum SortType {
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
            "downloads"  => Ok(Self::TotalDownloads),
            "popularity" => Ok(Self::Popularity),
            "name"       => Ok(Self::Name),
            "updated"    => Ok(Self::LastUpdated),
            "created"    => Ok(Self::DateCreated),

            s            => Err(format!("'{}' not a valid sort type", s)),
        }
    }
}

impl Display for SortType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match &self {
            &SortType::TotalDownloads => "TotalDownloads",
            &SortType::Popularity     => "Popularity",
            &SortType::Name           => "Name",
            &SortType::LastUpdated    => "LastUpdated",
            &SortType::DateCreated    => "DateCreated",
        })
    }
}

const BASE_URL: &str = "https://addons-ecs.forgesvc.net/api/v2/addon";

fn url(cdl: &Cdl) -> String {
    format!(
        "{base}/search?categoryId={category_id}&gameId={game_id}&gameVersion={game_version}&index={index}&pageSize={page_size}&searchFilter={search_filter}&sectionId={section_id}&sort={sort}", 
        base          = BASE_URL,
        category_id   = 0,
        game_id       = 432,
        game_version  = cdl.game_version,
        index         = 0,
        page_size     = cdl.amount,
        search_filter = cdl.query,
        section_id    = 6,
        sort          = cdl.sort,
    )
}

fn parse_query(src: &str) -> String {
    src.replace(" ", "%20").into()
}

#[derive(StructOpt, Debug)]
#[structopt(name = "cdl", about = "A command-line utility for downloading Minecraft mods.")]
struct Cdl {
    #[structopt(short = "l", long, default_value = "forge")]
    mod_loader: ModLoader,

    #[structopt(short = "v", long, default_value = "1.16.4")]
    game_version: String,

    #[structopt(short, long, default_value = "popularity")]
    sort: SortType,

    #[structopt(short, long, default_value = "9")]
    amount: u8,

    #[structopt(parse(from_str = parse_query))]
    query: String,
}

fn print_mod((index, result): (usize, models::SearchResult)) {
    println!("{}: {}", index + 1, result.name);
    println!("\t{}", result.website_url);
    println!("\t{:?}", result.game_files.iter().map(|g| &g.game_version).collect::<Vec<_>>());
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let cdl = Cdl::from_args();
    let url = url(&cdl);
    let result = reqwest::get(&url)
        .await?
        .json::<Vec<models::SearchResult>>()
        .await?;

    println!("{}", &url);

    result.into_iter()
        .enumerate()
        .for_each(print_mod);

    Ok(())
} 

