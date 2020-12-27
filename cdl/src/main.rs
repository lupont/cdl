use cdl_lib::git;
use std::{error::Error, fs};
use structopt::StructOpt;

mod cdl;
mod config;
mod ui;

use cdl::Cdl;
use config::Config;

fn choose_branch(repo: &git::Repository) -> git::Result<String> {
    let branches = repo
        .branches(None)?
        .filter_map(Result::ok)
        .filter(|(b, _)| b.name().is_ok())
        .map(|(b, _)| b)
        .collect::<Vec<_>>();

    ui::print_indexed_list(&["BRANCH"], &branches[..], |b| {
        b.name().unwrap().unwrap().to_string()
    });

    let input = ui::read_input()?;

    match input.parse::<usize>() {
        Ok(n) if n > 0 && n <= branches.len() => {
            return Ok(branches[n - 1].name().unwrap().unwrap().into());
        }

        _ => {
            println!("There's nothing to do.");
            return Err(git::GitError::InvalidBranchError);
        }
    }
}

fn handle_git(cdl: Cdl) -> git::Result<()> {
    let mut repo = git::clone(&cdl.query)?;

    println!("The following branches were found, please select one:");
    let branch = choose_branch(&repo)?;

    git::checkout(&mut repo, &branch)?;

    println!("Switched to branch {}, beginning build process...", &branch);
    git::execute_gradlew(&repo)?;

    println!("\nThe following jars were created, please select one or more:");
    let jars = git::get_compiled_jars(&repo)?;

    ui::print_indexed_list(&["FILE"], &jars, |j| {
        format!("{}", j.file_name().to_string_lossy())
    });

    let input = ui::read_input()?;
    if let Some(input) = ui::parse_input(&input) {
        for n in input {
            if n > 0 && n <= jars.len() {
                fs::copy(&mut jars[n - 1].path(), &mut jars[n - 1].file_name())?;
            } else {
                return Err(git::GitError::InvalidBranchError);
            }
        }
    }

    Ok(())
}

async fn handle_search(cdl: Cdl, config: Config) -> Result<(), cdl_lib::DownloadError> {
    let version = cdl.game_version.as_ref().unwrap_or(&config.game_version);
    let loader = cdl.mod_loader.as_ref().unwrap_or(&config.mod_loader);
    let amount = cdl.amount.unwrap_or(config.amount);
    let sort_type = cdl.sort.as_ref().unwrap_or(&config.sort_type);

    let search_results =
        cdl_lib::get_search_results(&cdl.query, version, amount, sort_type, loader).await?;

    if search_results.len() == 0 {
        println!(
            "No {} mods for {} including '{}' found.",
            loader.to_string(),
            version,
            cdl.query,
        );
        return Ok(());
    }

    ui::print_indexed_list2(
        &["NAME", "AUTHOR"],
        &search_results,
        |r| r.name.clone(),
        |r| r.author_names(),
    );

    println!(
        "Searched {} mods for {} including '{}'.",
        loader.to_string(),
        version,
        cdl.query,
    );

    let input = ui::read_input()?;

    let input = match ui::parse_input(&input) {
        Some(input) if input.iter().all(|i| *i <= search_results.len()) => input,
        _ => {
            println!("There's nothing to do.");
            return Ok(());
        }
    };

    let mods = search_results
        .iter()
        .enumerate()
        .filter(|(i, _)| input.contains(&(i + 1)))
        .map(|(_, r)| r)
        .collect::<Vec<_>>();

    cdl_lib::download_all(version, &mods[..], |event| {
        use cdl_lib::EventType::*;
        match event {
            MainDownloading(info) => print!("<== Downloading {}... ", info.file_name),
            MainDownloaded(_) => println!("done!"),
            MainAlreadyDownloaded(info) => {
                println!("<== {} is already downloaded.", info.file_name)
            }
            DepDownloading(info) => print!("    Downloading {}... ", info.file_name),
            DepDownloaded(_) => println!("done!"),
            DepAlreadyDownloaded(info) => println!("    {} is already downloaded.", info.file_name),
        }
    })
    .await?;
    Ok(())
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cdl = Cdl::from_args();
    let config = config::Config::load()?;

    if cdl.github {
        handle_git(cdl)?;
    } else {
        handle_search(cdl, config).await?;
    }

    Ok(())
}
