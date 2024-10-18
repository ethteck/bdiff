use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::bin_file::Endianness;
use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct WorkspaceFile {
    pub path: PathBuf,
    pub map: Option<PathBuf>,
    pub endianness: Endianness,
}

impl From<PathBuf> for WorkspaceFile {
    fn from(path: PathBuf) -> Self {
        Self { path, map: None, endianness: Endianness::Big }
    }
}

impl From<&Path> for WorkspaceFile {
    fn from(path: &Path) -> Self {
        let path: PathBuf = path.into();
        Self { path, map: None, endianness: Endianness::Big }
    }
}

#[derive(Clone, Deserialize, Serialize, Default)]
pub struct Workspace {
    pub files: Vec<WorkspaceFile>,
}

pub fn read_workspace_json(config_path: &Path) -> Result<Workspace, Error> {
    let mut reader = File::open(config_path)
        .with_context(|| format!("Failed to open config file at {}", config_path.display()))?;
    Ok(serde_json::from_reader(&mut reader)?)
}

pub fn write_workspace_json<P: Into<PathBuf>>(config_path: P, config: &Workspace) -> Result<(), Error> {
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
