use cdl_lib::models::{ModLoader, SortType};
use structopt::StructOpt;

fn parse_query(src: &str) -> String {
    src.replace(" ", "%20").into()
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cdl",
    about = "A command-line utility for downloading Minecraft mods."
)]
pub struct Cdl {
    #[structopt(short = "l", long, possible_values = &["forge", "fabric", "both"], help = "The mod loader to use when searching.")]
    pub mod_loader: Option<ModLoader>,

    #[structopt(short = "v", long, help = "The version of the game.")]
    pub game_version: Option<String>,

    #[structopt(short, long, possible_values = &["downloads", "popularity", "name", "updated", "created"], help = "The ordering of search results.")]
    pub sort: Option<SortType>,

    #[structopt(short, long, help = "The amount of search results to show.")]
    pub amount: Option<u8>,

    #[structopt(
        short,
        long,
        help = "Whether the query is the name of a Github repository and thus should be built from source."
    )]
    pub github: bool,

    #[structopt(parse(from_str = parse_query), help = "The query to search for.")]
    pub query: String,
}
