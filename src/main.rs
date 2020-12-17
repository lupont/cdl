use std::{
    fmt::{Display, Formatter},
    io::{stdin, stdout, Write},
    str::FromStr,
};
use async_recursion::async_recursion;
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

fn search_url(cdl: &Cdl) -> String {
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

fn mod_url(mod_id: u32) -> String {
    format!(
        "{base}/{mod_id}",
        base   = BASE_URL,
        mod_id = mod_id,
    )
}

fn info_url(mod_id: u32, file_id: u32) -> String {
    format!(
        "{base}/{mod_id}/file/{file_id}",
        base    = BASE_URL,
        mod_id  = mod_id,
        file_id = file_id,
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

fn print_mod((index, result): (usize, &models::SearchResult)) {
    println!("{}: {} by {}", index + 1, result.name, result.author_names());
    println!("\t{}", result.website_url);
    println!("\t{}", result.description);
    println!("\t{}", result.id);
    println!("\t{:?}", result.get_file_by_version("1.16.4"));
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let cdl = Cdl::from_args();
    let url = search_url(&cdl);

    let result = reqwest::get(&url)
        .await?
        .json::<Vec<models::SearchResult>>()
        .await?;

    result.iter()
        .enumerate()
        .for_each(print_mod);

    print!("==> ");
    if let Err(_) = stdout().flush() {}

    let input = {
        let mut tmp = String::new();
        let stdin = stdin();

        match stdin.read_line(&mut tmp) {
            Ok(_)  => tmp.trim().into(),
            Err(_) => String::new(),
        }
    };

    let input = parse_input(&input);

    if let Some(input) = input {
        let mods = result.into_iter()
            .enumerate()
            .filter(|(i, _)| input.contains(&(i + 1)))
            .map(|(_, r)| r)
            .collect::<Vec<_>>();

        for moddy in mods {
            let m = get_with_dependencies(&cdl, moddy.id).await?;
            let (first, rest) = m.split_first().unwrap();
            println!("<== Downloading {}...", first.display_name);
            for r in rest {
                println!("\t<== Downloading dependency {}...", r.display_name);
            }
        }
    }

    Ok(())
} 

#[async_recursion]
async fn get_with_dependencies(cdl: &Cdl, mod_id: u32) -> Result<Vec<models::ModInfo>, reqwest::Error> {
    let url = mod_url(mod_id);
    let result = reqwest::get(&url)
        .await?
        .json::<models::SearchResult>()
        .await?;

    let file_id = result.get_file_by_version(&cdl.game_version).map(|o| o.project_file_id);

    if let None = file_id {
        return Ok(vec![]);
    }

    let file = reqwest::get(&info_url(result.id, file_id.unwrap()))
        .await?
        .json::<models::ModInfo>()
        .await?;

    let mut mods: Vec<models::ModInfo> = vec![];

    for dep in file.dependencies.iter().filter(|&d| d.dep_type == 3) {
        let foo = get_with_dependencies(cdl, dep.addon_id).await?;
        mods.extend(foo);
    } 

    mods.insert(0, file);

    Ok(mods)
}

fn parse_input(input: &str) -> Option<Vec<usize>> {
    let foo = input.split(' ')
        .filter_map(|s| s.parse().ok())
        .collect::<Vec<_>>();

    match foo.len() {
        0 => None,
        _ => Some(foo),
    }
}

mod tests {
    use super::parse_input;

    #[test]
    fn parse_input_one_argument() {
        let input = "1";
        assert_eq!(parse_input(input), Some(vec![1]));
    }

    #[test]
    fn parse_input_two_arguments() {
        let input = "1 2";
        assert_eq!(parse_input(input), Some(vec![1, 2]));
    }

    #[test]
    fn parse_input_two_arguments_long() {
        let input = "11 12";
        assert_eq!(parse_input(input), Some(vec![11, 12]));
    }

    #[test]
    fn parse_input_long_argument() {
        let input = "13";
        assert_eq!(parse_input(input), Some(vec![13]));
    }

    #[test]
    fn parse_input_invalid() {
        let input = "hej";
        assert_eq!(parse_input(input), None);
    }

    #[test]
    fn parse_input_invalid_somewhere_ignores_error() {
        let input = "1 2 f 4";
        assert_eq!(parse_input(input), Some(vec![1, 2, 4]));
    }
}

