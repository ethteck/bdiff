use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Error;
use notify::Watcher;

#[derive(Default, Debug)]
pub struct BinFile {
    pub path: PathBuf,
    pub data: Vec<u8>,
    watcher: Option<notify::RecommendedWatcher>,
    pub modified: Arc<AtomicBool>,
}

pub fn read_file_bytes(path: PathBuf) -> Result<Vec<u8>, Error> {
    let file = match File::open(path.clone()) {
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

fn create_watcher(
    path: PathBuf,
    modified: Arc<AtomicBool>,
) -> notify::Result<notify::RecommendedWatcher> {
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if let notify::EventKind::Modify(_) = event.kind {
                    modified.store(true, Ordering::Relaxed);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })?;

    watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

    Ok(watcher)
}

pub fn read_file(path: PathBuf) -> Result<BinFile, Error> {
    let data = match read_file_bytes(path.clone()) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let mut ret = BinFile {
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
