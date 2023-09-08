use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Error};

#[derive(Clone, serde::Deserialize)]
pub struct FileConfig {
    pub path: PathBuf,
    pub map: Option<PathBuf>,
}

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub files: Vec<FileConfig>,
}

pub fn read_json_config(config_path: &Path) -> Result<Config, Error> {
    let mut reader = File::open(config_path)
        .with_context(|| format!("Failed to open config file at {}", config_path.display()))?;
    Ok(serde_json::from_reader(&mut reader)?)
}
