use crate::cdl::{ModLoader, SortType};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    TomlSerializeError(toml::ser::Error),
    TomlDeserializeError(toml::de::Error),
    VarError(env::VarError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IoError(e) => e.to_string(),
                Self::TomlSerializeError(e) => e.to_string(),
                Self::TomlDeserializeError(e) => e.to_string(),
                Self::VarError(e) => e.to_string(),
            }
        )
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        Self::TomlSerializeError(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::TomlDeserializeError(e)
    }
}

impl From<env::VarError> for ConfigError {
    fn from(e: env::VarError) -> Self {
        Self::VarError(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub sort_type: SortType,
    pub amount: u8,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let home_dir = &env::var("HOME")?;
        let config_dir = Path::join(Path::new(&home_dir), ".config/cdl");
        let config_file = Path::new("cdl.toml");
        let config_path = Path::join(&config_dir, config_file);

        if !config_path.exists() {
            fs::create_dir_all(config_dir)?;
            let default = Self::default();
            let toml = toml::to_string(&default)?;
            fs::write(&config_path, toml)?;
        }

        let file = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&file)?;

        Ok(config)
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
