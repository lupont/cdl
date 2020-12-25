use cdl_lib::{git, models};
use std::{
    error::Error,
    fs,
    io::{self, Write},
};
use structopt::StructOpt;

mod cdl;
mod config;

use cdl::Cdl;
use config::Config;

fn choose_branch(repo: &git::Repository) -> git::Result<String> {
    let branches = repo
        .branches(None)?
        .filter_map(Result::ok)
        .filter(|(b, _)| b.name().is_ok())
        .map(|(b, _)| b)
        .enumerate()
        .collect::<Vec<(usize, git::Branch)>>();

    println!("  INDEX  BRANCH");
    for (i, branch) in &branches {
        let name = branch.name().unwrap();
        if let Some(name) = name {
            println!(
                "> {}     {}{}",
                i + 1,
                if i + 1 < 10 { " " } else { "" },
                name
            );
        }
    }

    print!("==> ");
    io::stdout().flush()?;

    let input = crate::read_input()?;

    match input.parse::<usize>() {
        Ok(n) if n > 0 && n <= branches.len() => {
            let branch = &branches[n - 1].1;
            return Ok(branch.name().unwrap().unwrap().into());
        }

        _ => {
            println!("here's nothing to do.");
            return Err(git::GitError::InvalidBranchError);
        }
    }
}

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

fn handle_git(cdl: Cdl) -> git::Result<()> {
    let mut repo = git::clone(&cdl.query)?;

    println!("The following branches were found, please select one:");
    let branch = choose_branch(&repo)?;

    git::checkout(&mut repo, &branch)?;

    println!("Switched to branch {}, beginning build process...", &branch);
    git::execute_gradlew(&repo)?;

    println!("\nThe following jars were created, please select one or more:");
    let jars = git::get_compiled_jars(&repo)?;

    println!("  INDEX  FILE");
    for (i, jar) in jars.iter().enumerate() {
        println!(
            "> {}     {}{}",
            i + 1,
            if i + 1 < 10 { " " } else { "" },
            jar.file_name().to_string_lossy()
        );
    }

    print!("==> ");
    io::stdout().flush()?;
    let input = read_input()?;
    if let Some(input) = parse_input(&input) {
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

    let url = cdl_lib::url::search_url(&cdl.query, version, amount, sort_type);

    let search_results = cdl_lib::get_search_results(&url, loader).await?;

    if search_results.len() == 0 {
        println!(
            "No {} mods for {} including '{}' found.",
            loader.to_string(),
            version,
            cdl.query,
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
        loader.to_string(),
        version,
        cdl.query,
    );

    print!("==> ");
    io::stdout().flush()?;

    let input = read_input()?;

    let input = match parse_input(&input) {
        Some(input) => input,
        None => {
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
            MainAlreadyDownloaded(info) => println!("<== {} already downloaded.", info.file_name),
            DepDownloading(info) => print!("    Downloading {}... ", info.file_name),
            DepDownloaded(_) => println!("done!"),
            DepAlreadyDownloaded(info) => println!("    {} already downloaded.", info.file_name),
        }
    })
    .await?;
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

fn parse_input(input: &str) -> Option<Vec<usize>> {
    let foo = input
        .trim()
        .split(' ')
        .filter_map(|s| {
            if s.contains("-") {
                let mut parts = s.split("-");
                let start = parts.next()?.parse::<usize>().ok()?;
                let end = parts.next()?.parse::<usize>().ok()?;

                Some((start..=end).collect::<Vec<_>>())
            } else {
                Some(vec![s.parse::<usize>().ok()?])
            }
        })
        .flatten()
        .fold(Vec::new(), |mut a, c| {
            if !a.contains(&c) {
                a.push(c);
            }
            a
        });

    match foo.len() {
        0 => None,
        _ => Some(foo),
    }
}

fn read_input() -> io::Result<String> {
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_parse() {
        assert_eq!(parse_input("1"), Some(vec![1]));
        assert_eq!(parse_input("1 2 3"), Some(vec![1, 2, 3]));
        assert_eq!(parse_input("1 1 2 1 3 4 10"), Some(vec![1, 2, 3, 4, 10]));
        assert_eq!(parse_input("1-9"), Some((1..=9).collect()));
        assert_eq!(parse_input("1-3 5 7"), Some(vec![1, 2, 3, 5, 7]));
        assert_eq!(parse_input("1 3 5-6 7"), Some(vec![1, 3, 5, 6, 7]));
        assert_eq!(parse_input("1-3 1 2 3"), Some((1..=3).collect()));
    }
}
