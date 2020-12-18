use crate::{
    cdl::{ModLoader, SortType},
    Cdl,
};

pub struct Config {
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub sort_type: SortType,
    pub amount: u8,
}

impl Config {
    pub fn from_cdl(cdl: &Cdl) -> Self {
        Self {
            game_version: cdl.game_version.clone().into(),
            mod_loader: cdl.mod_loader.clone(),
            sort_type: cdl.sort.clone(),
            amount: cdl.amount,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game_version: "1.16.4".into(),
            mod_loader: ModLoader::Forge,
            sort_type: SortType::Popularity,
            amount: 9,
        }
    }
}
