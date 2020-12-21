use async_recursion::async_recursion;
use reqwest::{Client, IntoUrl};
use std::{
    error::Error,
    fs::File,
    io::{copy, stdin, stdout, Write},
};
use structopt::StructOpt;

mod cdl;
mod config;
mod git;
mod models;
mod url;

use cdl::Cdl;
use config::Config;

fn print_mod(max_len: usize, (index, result): (usize, &models::SearchResult)) {
    println!(
        "> {index}{space1}{name} {space2}{authors} {fabric}",
        index = index + 1,
        space1 = " ".repeat(6 - (index + 1).to_string().len() + 1),
        name = result.name,
        space2 = " ".repeat(max_len - result.name.len() + 1),
        authors = result.author_names(),
        fabric = if result.is_fabric() { "[FABRIC]" } else { "" }
    );
}

fn handle_git(cdl: Cdl) -> Result<(), git::GitError> {
    let mut repo = git::clone(&cdl.query)?;

    println!("The following branches were found, please select one:");
    let branch = git::choose_branch(&repo)?;

    git::checkout(&mut repo, &branch)?;

    println!("Switched to branch {}, beginning build process...", &branch);
    git::execute_gradlew(&repo)?;

    println!("\nThe following jars were created, please select one or more:");
    git::copy_compiled_mod(&repo)?;

    Ok(())
}

async fn handle_search(cdl: Cdl, config: Config) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = url::search_url(&config, &cdl.query);

    let mut search_results = client
        .get(&url)
        .send()
        .await?
        .json::<Vec<models::SearchResult>>()
        .await?;

    {
        let mut i = 0;
        while i != search_results.len() {
            if &cdl.mod_loader == &cdl::ModLoader::Forge && search_results[i].is_fabric() {
                search_results.remove(i);
            } else {
                i += 1;
            }
        }
    }

    if search_results.len() == 0 {
        println!(
            "No {} mods for {} including '{}' found.",
            config.mod_loader.to_string(),
            config.game_version,
            cdl.query
        );
        return Ok(());
    }

    let max_len = search_results
        .iter()
        .fold(0, |a, c| if c.name.len() > a { c.name.len() } else { a });

    println!(
        "  INDEX  NAME{} AUTHOR",
        " ".repeat(if max_len < 3 { 1 } else { max_len - 3 }),
    );

    search_results
        .iter()
        .enumerate()
        .for_each(|m| print_mod(max_len, m));

    println!(
        "Searched {} mods for {} including '{}'.",
        config.mod_loader.to_string(),
        config.game_version,
        cdl.query,
    );

    print!("==> ");
    stdout().flush()?;

    let input = {
        let mut tmp = String::new();
        stdin().read_line(&mut tmp)?;
        tmp.trim().to_string()
    };

    let input = match parse_input(&input) {
        Some(input) => input,
        None => {
            println!("There's nothing to do.");
            return Ok(());
        }
    };

    let mods = search_results
        .into_iter()
        .enumerate()
        .filter(|(i, _)| input.contains(&(i + 1)))
        .map(|(_, r)| r)
        .collect::<Vec<_>>();

    let mut already_downloaded: Vec<u32> = vec![];

    for moddy in mods {
        let m = get_with_dependencies(&cdl, &client, moddy.id).await?;
        let (first, rest) = m.split_first().ok_or("mod list was empty")?;

        print!("<== Downloading {}...", first.file_name);
        match download(&client, &first.download_url, &first.file_name).await {
            Ok(_) => {
                println!(" done!");
                already_downloaded.push(first.id);
            }
            Err(_) => println!(" An error occured."),
        }

        for r in rest {
            if already_downloaded.contains(&r.id) {
                println!("    {} already downloaded.", r.file_name);
                continue;
            }

            print!("    Downloading {}...", r.file_name);
            match download(&client, &r.download_url, &r.file_name).await {
                Ok(_) => {
                    println!(" done!");
                    already_downloaded.push(r.id);
                }
                Err(_) => println!(" An error occured."),
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cdl = Cdl::from_args();
    let config = config::Config::load()?;

    if cdl.github_repo {
        handle_git(cdl)?;
    } else {
        handle_search(cdl, config).await?;
    }

    Ok(())
}

async fn download<T: IntoUrl>(
    client: &Client,
    url: T,
    file_name: &str,
) -> Result<(), Box<dyn Error>> {
    let mut dest = File::create(file_name)?;
    let source = client.get(url).send().await?.bytes().await?;
    copy(&mut source.as_ref(), &mut dest)?;
    Ok(())
}

#[async_recursion]
async fn get_with_dependencies(
    cdl: &Cdl,
    client: &Client,
    mod_id: u32,
) -> Result<Vec<models::ModInfo>, reqwest::Error> {
    let url = url::mod_url(mod_id);
    let result = client
        .get(&url)
        .send()
        .await?
        .json::<models::SearchResult>()
        .await?;

    let file_id = result
        .get_file_by_version(&cdl.game_version)
        .map(|o| o.project_file_id);

    if let None = file_id {
        return Ok(vec![]);
    }

    let file = client
        .get(&url::info_url(result.id, file_id.unwrap()))
        .send()
        .await?
        .json::<models::ModInfo>()
        .await?;

    let mut mods: Vec<models::ModInfo> = vec![];

    for dep in file.dependencies.iter().filter(|&d| d.dep_type == 3) {
        let foo = get_with_dependencies(cdl, client, dep.addon_id).await?;
        mods.extend(foo);
    }

    mods.insert(0, file);

    Ok(mods)
}

fn parse_input(input: &str) -> Option<Vec<usize>> {
    let foo = input
        .trim()
        .split(' ')
        .filter_map(|s| s.parse().ok())
        .collect::<Vec<_>>();

    match foo.len() {
        0 => None,
        _ => Some(foo),
    }
}
