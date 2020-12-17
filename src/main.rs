use std::str::FromStr;
use structopt::StructOpt;

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

            s        => Err(format!("{} not a valid mod loader", s)),
        }
    }
}

const BASE_URL: &str = "https://addons-ecs.forgesvc.net/api/v2/addon";

fn url(version: &str, query: &str) -> String {
    format!(
        "{base}/search?categoryId={categoryID}&gameId={gameId}&gameVersion={gameVersion}&index={index}&pageSize={pageSize}5&searchFilter={searchFilter}&sectionId={sectionId}&sort={sort}", 
        base         = BASE_URL,
        categoryID   = 0,
        gameId       = 432,
        gameVersion  = version,
        index        = 0,
        pageSize     = 9,
        searchFilter = query,
        sectionId    = 6,
        sort         = "TotalDownloads"
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

    #[structopt(parse(from_str = parse_query))]
    query: String,
}

fn main() {
    let cdl = Cdl::from_args();

    // simulate request
    let url = url(&cdl.game_version, &cdl.query);

    println!("{}", &url);
} 

