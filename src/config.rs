use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct FileConfig {
    pub path: PathBuf,
    pub map: Option<PathBuf>,
}

impl From<PathBuf> for FileConfig {
    fn from(path: PathBuf) -> Self {
        Self { path, map: None }
    }
}

impl From<&Path> for FileConfig {
    fn from(path: &Path) -> Self {
        let path: PathBuf = path.into();
        Self { path, map: None }
    }
}

#[derive(Clone, Deserialize, Serialize, Default)]
pub struct Config {
    pub files: Vec<FileConfig>,
    #[serde(skip)]
    pub changed: bool,
}

pub fn read_json_config(config_path: &Path) -> Result<Config, Error> {
    let mut reader = File::open(config_path)
        .with_context(|| format!("Failed to open config file at {}", config_path.display()))?;
    Ok(serde_json::from_reader(&mut reader)?)
}

#[allow(dead_code)]
pub fn write_json_config<P: Into<PathBuf>>(config_path: P, config: &Config) -> Result<(), Error> {
    let path: PathBuf = config_path.into();
    let mut oo = OpenOptions::new();
    let mut writer = oo
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .with_context(|| format!("Failed to open config file at {}", path.display()))?;
    Ok(writer.write_all(serde_json::to_string_pretty(config)?.as_bytes())?)
}
