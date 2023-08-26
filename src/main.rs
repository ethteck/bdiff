mod app;
mod error;
mod hex_view;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use app::BdiffApp;
use argh::FromArgs;
use eframe::IconData;
use error::BDiffError;
use notify::Watcher;

#[derive(FromArgs)]
/// binary differ
struct Args {
    /// input files
    #[argh(positional)]
    files: Vec<PathBuf>,
}

#[derive(Default, Debug)]
pub struct BinFile {
    path: PathBuf,
    data: Vec<u8>,
    watcher: Option<notify::RecommendedWatcher>,
    modified: Arc<AtomicBool>,
}

impl BinFile {
    pub fn reload(&mut self) -> Result<(), BDiffError> {
        self.data = read_file_bytes(self.path.clone())?;
        Ok(())
    }
}

pub fn read_file_bytes(path: PathBuf) -> Result<Vec<u8>, BDiffError> {
    let file = match File::open(path.clone()) {
        Ok(file) => file,
        Err(_error) => return Result::Err(BDiffError::IOError),
    };

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    let _ = buf_reader
        .read_to_end(&mut buffer)
        .or(Err(BDiffError::IOError));

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

fn read_file(path: PathBuf) -> Result<BinFile, BDiffError> {
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

fn main() {
    let args: Args = argh::from_env();

    let native_options = eframe::NativeOptions {
        icon_data: Some(
            IconData::try_from_png_bytes(include_bytes!("../assets/icon.png")).unwrap(),
        ),
        ..Default::default()
    };

    let mut files = Vec::new();
    for path in args.files {
        files.push(read_file(path).unwrap());
    }

    let _ = eframe::run_native(
        "bdiff",
        native_options,
        Box::new(|cc| Box::new(BdiffApp::new(cc, files))),
    );
}
