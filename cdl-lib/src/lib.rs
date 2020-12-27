pub mod git;
pub mod models;
pub mod url;

use models::{ModInfo, ModLoader, SearchResult, SortType};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;

pub async fn get_search_results(
    query: &str,
    version: &str,
    amount: u8,
    sort_type: &SortType,
    mod_loader: &ModLoader,
) -> surf::Result<Vec<SearchResult>> {
    let url = url::search_url(query, version, amount, sort_type);
    let mut results = surf::get(&url)
        .await?
        .body_json::<Vec<SearchResult>>()
        .await?;

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

#[derive(Debug)]
pub enum DownloadError {
    IoError(io::Error),
    SurfError(surf::Error),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "{}", e),
            Self::SurfError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for DownloadError {}

impl From<io::Error> for DownloadError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<surf::Error> for DownloadError {
    fn from(e: surf::Error) -> Self {
        Self::SurfError(e)
    }
}

pub async fn download(url: &str, file_name: &str) -> Result<(), DownloadError> {
    // The url from the API references an endpoint that redirects
    // to another url for downloading. reqwest handled
    // this fine, but surf seems to not follow the redirect.
    // Thus, one request is made in order to find the "real" url,
    // before sending a GET to it.
    let tmp = surf::get(&url).await?;
    let loc = tmp.header("location");
    let url = loc.map(|h| h.as_str()).unwrap_or(&url);

    let source = surf::get(&url).recv_bytes().await?;
    let mut dest = File::create(file_name)?;
    io::copy(&mut source.as_slice(), &mut dest)?;
    Ok(())
}

pub enum EventType<'a> {
    MainAlreadyDownloaded(&'a ModInfo),
    MainDownloading(&'a ModInfo),
    MainDownloaded(&'a ModInfo),
    MainError(&'a ModInfo),
    DepAlreadyDownloaded(&'a ModInfo),
    DepDownloading(&'a ModInfo),
    DepDownloaded(&'a ModInfo),
    DepError(&'a ModInfo),
}

pub async fn download_all<F: Fn(EventType)>(
    game_version: &str,
    results: &[&SearchResult],
    on_event: F,
) -> surf::Result<()> {
    use EventType::*;
    let mut already_downloaded = Vec::<u32>::new();
    for result in results {
        let m = get_with_dependencies(game_version, result.id).await?;
        if let Some((first, rest)) = m.split_first() {
            if Path::new(&first.file_name).exists() || already_downloaded.contains(&first.id) {
                on_event(MainAlreadyDownloaded(&first));
                continue;
            }

            on_event(MainDownloading(&first));
            if let Ok(()) = download(&first.download_url, &first.file_name).await {
                already_downloaded.push(first.id);
                on_event(MainDownloaded(&first));
            } else {
                on_event(MainError(&first));
            }

            for r in rest {
                if Path::new(&r.file_name).exists() || already_downloaded.contains(&r.id) {
                    on_event(DepAlreadyDownloaded(&r));
                    continue;
                }

                on_event(DepDownloading(&r));
                if let Ok(()) = download(&r.download_url, &r.file_name).await {
                    already_downloaded.push(r.id);
                    on_event(DepDownloaded(&r));
                } else {
                    on_event(DepError(&r));
                }
            }
        }
    }
    Ok(())
}

#[async_recursion::async_recursion]
async fn get_with_dependencies(game_version: &str, mod_id: u32) -> surf::Result<Vec<ModInfo>> {
    let url = url::mod_url(mod_id);
    let result = surf::get(&url).recv_json::<SearchResult>().await?;

    let file_id = result
        .get_file_by_version(game_version)
        .map(|file| file.project_file_id);

    if let None = file_id {
        return Ok(vec![]);
    }

    let file = surf::get(&url::info_url(result.id, file_id.unwrap()))
        .recv_json::<ModInfo>()
        .await?;

    let mut mods: Vec<ModInfo> = vec![];

    for dep in file.hard_dependencies() {
        let foo = get_with_dependencies(game_version, dep.addon_id).await?;
        mods.extend(foo);
    }

    mods.insert(0, file);

    Ok(mods)
}
