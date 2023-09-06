use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Error};

use crate::app::BdiffApp;

#[derive(Clone, serde::Deserialize)]
pub struct FileConfig {
    pub path: PathBuf,
    pub map: Option<PathBuf>,
}

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub files: Vec<FileConfig>,
}

pub fn load_project_config(state: &mut BdiffApp) -> Result<(), Error> {
    let config_path = Path::new("bdiff.json");

    if config_path.exists() {
        let config = read_json_config(config_path).unwrap();

        for file in config.files {
            let hv = state.open_file(file.path)?;

            if let Some(map) = file.map {
                hv.mt.load_file(&map);
            }
        }
    }

    Ok(())
}

fn read_json_config(config_path: &Path) -> Result<Config, Error> {
    let mut reader = File::open(config_path)
        .with_context(|| format!("Failed to open config file at {}", config_path.display()))?;
    Ok(serde_json::from_reader(&mut reader)?)
}
