use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

use anyhow::Error;

use crate::watcher::create_watcher;

#[derive(Clone, Copy, Debug, Default)]
pub enum Endianness {
    Little,
    #[default]
    Big,
}

#[derive(Debug, Default)]
pub struct BinFile {
    pub path: PathBuf,
    pub data: Vec<u8>,
    pub endianness: Endianness,
    watcher: Option<notify::RecommendedWatcher>,
    pub modified: Arc<AtomicBool>,
}

pub fn read_file_bytes<P: Into<PathBuf>>(path: P) -> Result<Vec<u8>, Error> {
    let file = match File::open(path.into()) {
        Ok(file) => file,
        Err(_error) => {
            return Err(Error::msg("Failed to open file"));
        }
    };

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    let _ = buf_reader
        .read_to_end(&mut buffer)
        .or(Err(Error::msg("Failed to read file")));

    Ok(buffer)
}

impl BinFile {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self, Error> {
        let path: PathBuf = path.into();
        let data = match read_file_bytes(&path) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        let mut ret = Self {
            path: path.clone(),
            data,
            watcher: None,
            ..Default::default()
        };

        match create_watcher(path, ret.modified.clone()).map_err(anyhow::Error::new) {
            Ok(watcher) => {
                ret.watcher = Some(watcher);
            }
            Err(e) => log::error!("Failed to create watcher: {e}"),
        }

        Ok(ret)
    }
}
