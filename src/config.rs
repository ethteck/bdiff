use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

use crate::settings::Settings;

#[derive(Clone, Deserialize, Serialize)]
pub struct FileConfig {
    pub path: PathBuf,
    pub map: Option<PathBuf>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub files: Vec<FileConfig>,
}

pub fn read_json_config(config_path: &Path) -> Result<Config, Error> {
    let mut reader = File::open(config_path)
        .with_context(|| format!("Failed to open config file at {}", config_path.display()))?;
    Ok(serde_json::from_reader(&mut reader)?)
}

#[allow(dead_code)]
pub fn write_json_config(config_path: &Path, config: &Config) -> Result<(), Error> {
    let mut writer = File::open(config_path)
        .with_context(|| format!("Failed to open config file at {}", config_path.display()))?;
    Ok(writer.write_all(serde_json::to_string_pretty(config)?.as_bytes())?)
}
