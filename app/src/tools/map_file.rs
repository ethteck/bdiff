use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

use anyhow::Error;
use iset::IntervalMap;

use crate::watcher::create_watcher;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MapFileEntry {
    pub seg_name: String,
    pub seg_vram: u64,
    pub seg_vrom: u64,
    pub seg_size: u64,
    pub file_path: PathBuf,
    pub file_section_type: String,
    pub file_vram: u64,
    pub file_vrom: Option<u64>,
    pub file_size: u64,
    pub symbol_name: String,
    pub symbol_vram: usize,
    pub symbol_vrom: usize,
    pub symbol_size: usize,
}

#[derive(Default)]
pub struct MapFile {
    pub path: PathBuf,
    pub data: IntervalMap<usize, MapFileEntry>,
    watcher: Option<notify::RecommendedWatcher>,
    pub modified: Arc<AtomicBool>,
}

impl MapFile {
    pub fn from_path(path: PathBuf) -> Result<Self, Error> {
        let data = collect_data(path.clone());

        let mut ret = Self {
            path: path.clone(),
            data,
            watcher: None,
            ..Default::default()
        };

        match create_watcher(path, ret.modified.clone()).map_err(Error::new) {
            Ok(watcher) => {
                ret.watcher = Some(watcher);
            }
            Err(e) => log::error!("Failed to create watcher: {e}"),
        }

        Ok(ret)
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        self.data = collect_data(self.path.clone());

        Ok(())
    }

    pub fn get_entry(&self, start: usize, end: usize) -> Option<&MapFileEntry> {
        let entries: Vec<_> = self.data.values(start..end).collect();

        match entries.first() {
            Some(entry) => {
                if entry.symbol_vrom > start {
                    return None;
                }
                Some(*entry)
            }
            None => None,
        }
    }
}

fn collect_data(path: PathBuf) -> IntervalMap<usize, MapFileEntry> {
    let mut ret: IntervalMap<usize, MapFileEntry> = IntervalMap::new();

    let mut mf: mapfile_parser::MapFile = mapfile_parser::MapFile::new();

    mf.read_map_file(&path);

    for segment in &mf.segments_list {
        for file in &segment.files_list {
            for symbol in &file.symbols {
                if symbol.vrom.is_none() || symbol.size.is_none() || symbol.size.unwrap() == 0 {
                    continue;
                }

                let entry = MapFileEntry {
                    seg_name: segment.name.clone(),
                    seg_vram: segment.vram,
                    seg_vrom: segment.vrom,
                    seg_size: segment.size,
                    file_path: file.filepath.clone(),
                    file_section_type: file.section_type.clone(),
                    file_vram: file.vram,
                    file_vrom: file.vrom,
                    file_size: file.size,
                    symbol_name: symbol.name.clone(),
                    symbol_vram: symbol.vram as usize,
                    symbol_vrom: symbol.vrom.unwrap() as usize,
                    symbol_size: symbol.size.unwrap() as usize,
                };

                ret.insert(
                    entry.symbol_vrom..entry.symbol_vrom + entry.symbol_size,
                    entry.clone(),
                );
            }
        }
    }

    ret
}
