use enigo::{Enigo, Mouse, Settings};
use serde::{Serialize, Deserialize};
use log::LevelFilter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Screen {
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetadataConfig {
    pub author: String,
    pub title: String,
    pub version: String,
    pub screen: Screen,
    pub log_level: LevelFilter,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        let size = Enigo::new(&Settings::default()).unwrap()
            .main_display().unwrap();

        Self {
            author: "omnilynceus".into(),
            title: "omnilynceus-loader".into(),
            version: "0.1.0".into(),
            screen: Screen { width: size.0, height: size.1 },
            log_level: match cfg!(debug_assertions) {
                true => LevelFilter::Debug,
                false => LevelFilter::Info,
            },
        }
    }
}