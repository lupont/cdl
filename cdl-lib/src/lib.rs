pub mod git;
pub mod models;
pub mod url;

use models::{ModInfo, ModLoader, SearchResult};
use std::error::Error;
use std::fs::File;
use std::io::copy;

pub async fn get_search_results<T: reqwest::IntoUrl>(
    url: T,
    mod_loader: &ModLoader,
) -> reqwest::Result<Vec<SearchResult>> {
    let mut results = reqwest::get(url).await?.json::<Vec<SearchResult>>().await?;

    let mut i = 0;
    while i != results.len() {
        if mod_loader == &ModLoader::Forge && results[i].is_fabric() {
            results.remove(i);
        } else if mod_loader == &ModLoader::Fabric && !results[i].is_fabric() {
            results.remove(i);
        } else {
            i += 1;
        }
    }

    Ok(results)
}

pub async fn download<T: reqwest::IntoUrl>(url: T, file_name: &str) -> Result<(), Box<dyn Error>> {
    let mut dest = File::create(file_name)?;
    let source = reqwest::get(url).await?.bytes().await?;
    copy(&mut source.as_ref(), &mut dest)?;
    Ok(())
}

pub async fn download_all(game_version: &str, ids: &[u32]) -> Result<(), Box<dyn Error>> {
    let mut already_downloaded = Vec::<u32>::new();
    for id in ids {
        let m = get_with_dependencies(game_version, *id).await?;
        let (first, rest) = m.split_first().ok_or("mod list was empty")?;

        if let Ok(()) = download(&first.download_url, &first.file_name).await {
            already_downloaded.push(first.id);
        }

        for r in rest {
            if already_downloaded.contains(&r.id) {
                continue;
            }

            if let Ok(()) = download(&r.download_url, &r.file_name).await {
                already_downloaded.push(r.id);
            }
        }
    }
    Ok(())
}

#[async_recursion::async_recursion]
async fn get_with_dependencies(game_version: &str, mod_id: u32) -> reqwest::Result<Vec<ModInfo>> {
    let url = url::mod_url(mod_id);
    let result = reqwest::get(&url).await?.json::<SearchResult>().await?;

    let file_id = result
        .get_file_by_version(game_version)
        .map(|file| file.project_file_id);

    if let None = file_id {
        return Ok(vec![]);
    }

    let file = reqwest::get(&url::info_url(result.id, file_id.unwrap()))
        .await?
        .json::<ModInfo>()
        .await?;

    let mut mods: Vec<ModInfo> = vec![];

    for dep in file.dependencies.iter().filter(|&d| d.dep_type == 3) {
        let foo = get_with_dependencies(game_version, dep.addon_id).await?;
        mods.extend(foo);
    }

    mods.insert(0, file);

    Ok(mods)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
