
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::Error;
use log::error;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PathsConfig {
    pub dirs: HashMap<String, PathBuf>,
    pub files: HashMap<String, PathBuf>,
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|e| {
        error!("Failed to get current working directory: {}", e);
        PathBuf::from(".")
    })
}

fn project() -> PathBuf {
    cwd().parent()
        .map(Path::to_path_buf) 
        .ok_or(Error::new(
            std::io::ErrorKind::Other,
            "No parent directory"
        ))
        .unwrap_or_else(|e| {
            error!("Failed to get parent directory: {}", e);
            PathBuf::from("..")
        })
}

fn data_dir() -> PathBuf {
    match cfg!(debug_assertions) {
        true => project().join("oll-data"),
        false => cwd().join("oll-data"),
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            dirs: HashMap::from([
                ("data".into(), data_dir()),
                ("logs".into(), data_dir().join("logs")),
            ]),
            files: HashMap::from([
                ("config".into(), data_dir().join("config.toml")),
            ]),
        }
    }
}

impl PathsConfig {
    pub fn ensure(&self) -> std::io::Result<()> {
        for dir in self.dirs.values() {
            std::fs::create_dir_all(dir)?;
        }
        for file in self.files.values() {
            if !file.exists() {
                std::fs::File::create(file)?;
            }
        }
        Ok(())
    }

    pub fn file(&self, key: &str) -> &PathBuf {
        self.files.get(key)
            .expect(&format!("配置中缺少必要的文件路径: {}", key))
    }

    pub fn dir(&self, key: &str) -> &PathBuf {
        self.dirs.get(key)
            .expect(&format!("配置中缺少必要的目录路径: {}", key))
    }
}

