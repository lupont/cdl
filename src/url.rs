use crate::Cdl;

const BASE_URL: &str = "https://addons-ecs.forgesvc.net/api/v2/addon";

pub fn search_url(cdl: &Cdl) -> String {
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

pub fn mod_url(mod_id: u32) -> String {
    format!("{base}/{mod_id}", base = BASE_URL, mod_id = mod_id,)
}

pub fn info_url(mod_id: u32, file_id: u32) -> String {
    format!(
        "{base}/{mod_id}/file/{file_id}",
        base = BASE_URL,
        mod_id = mod_id,
        file_id = file_id,
    )
}
