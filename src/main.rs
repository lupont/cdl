use async_recursion::async_recursion;
use reqwest::{Client, IntoUrl};
use std::{
    error::Error,
    fs::File,
    io::{copy, stdin, stdout, Write},
};
use structopt::StructOpt;

mod cdl;
mod models;
mod url;

use cdl::Cdl;

fn print_mod((index, result): (usize, &models::SearchResult)) {
    println!(
        "{}: {} by {}",
        index + 1,
        result.name,
        result.author_names()
    );
    println!("\t{}", result.website_url);
    println!("\t{}", result.description);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cdl = Cdl::from_args();
    let client = Client::new();
    let url = url::search_url(&cdl);

    let search_results = client
        .get(&url)
        .send()
        .await?
        .json::<Vec<models::SearchResult>>()
        .await?;

    search_results.iter().enumerate().for_each(print_mod);

    print!("==> ");
    stdout().flush()?;

    let input = {
        let mut tmp = String::new();
        stdin().read_line(&mut tmp)?;
        tmp.trim().to_string()
    };

    let input = parse_input(&input).ok_or("")?;

    let mods = search_results
        .into_iter()
        .enumerate()
        .filter(|(i, _)| input.contains(&(i + 1)))
        .map(|(_, r)| r)
        .collect::<Vec<_>>();

    for moddy in mods {
        let m = get_with_dependencies(&cdl, &client, moddy.id).await?;
        let (first, rest) = m.split_first().ok_or("mod list was empty")?;

        println!("<== Downloading {}...", first.display_name);
        download(&client, &first.download_url, &first.file_name).await?;

        for r in rest {
            println!("\t<== Downloading dependency {}...", r.display_name);
            download(&client, &r.download_url, &r.file_name).await?;
        }
    }

    Ok(())
}

async fn download<T>(client: &Client, url: T, file_name: &str) -> Result<(), Box<dyn Error>>
where
    T: IntoUrl,
{
    let mut dest = File::create(file_name)?;
    let source = client.get(url).send().await?.text().await?;
    copy(&mut source.as_bytes(), &mut dest)?;

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
        .split(' ')
        .filter_map(|s| s.parse().ok())
        .collect::<Vec<_>>();

    match foo.len() {
        0 => None,
        _ => Some(foo),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
