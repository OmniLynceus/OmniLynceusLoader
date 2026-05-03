mod path;
mod metadata;

use metadata::MetadataConfig;
use path::PathsConfig;

use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use std::fs;
use toml;
use log::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub metadata: MetadataConfig,
    pub paths: PathsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            metadata: MetadataConfig::default(),
            paths: PathsConfig::default(),
        }
    }
}

impl Config {
    // In there the logget has not been initialized, so just ignore the error and use default config when failed to load the config file.
    // Hope there won't be a hidden danger in this.
    fn load() -> Self {
        let config = Self::default();
        match toml::from_str::<Config>(
            fs::read_to_string(&config.paths.file("config"))
                .unwrap_or_default().as_str()
        ) {
            Ok(loaded) => loaded,
            Err(_) => config
        } 
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(
            &self.paths.file("config"),
            toml::to_string_pretty(&self)?
        )?;
        info!("配置已保存到 {}", self.paths.file("config").display());
        Ok(())
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);    
