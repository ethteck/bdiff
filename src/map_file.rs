use std::{
    fs,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

use anyhow::Error;
use iset::IntervalMap;
use serde::Deserialize;

use crate::watcher::create_watcher;

#[derive(Deserialize, Debug)]
struct MapFileJsonSymbol {
    name: String,
    vram: usize,
    vrom: Option<usize>,
    size: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct MapFileJsonFile {
    filepath: Option<String>,
    #[serde(rename = "camelCase")]
    section_type: Option<String>,
    vram: usize,
    vrom: Option<usize>,
    size: usize,
    symbols: Vec<MapFileJsonSymbol>,
}

#[derive(Deserialize, Debug)]
struct MapFileJsonSegment {
    name: String,
    vram: usize,
    vrom: usize,
    size: usize,
    files: Vec<MapFileJsonFile>,
}

#[derive(Deserialize, Debug)]
struct MapFileJson {
    segments: Vec<MapFileJsonSegment>,
}

#[derive(Clone, Debug)]
pub struct MapFileEntry {
    pub seg_name: String,
    pub seg_vram: usize,
    pub seg_vrom: usize,
    pub seg_size: usize,
    pub file_path: Option<String>,
    pub file_section_type: Option<String>,
    pub file_vram: usize,
    pub file_vrom: Option<usize>,
    pub file_size: usize,
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
        let s = fs::read_to_string(path.clone())?;

        let json: MapFileJson = serde_json::from_str(&s)?;

        let data = collect_data(&json);

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

    pub fn reload(&mut self) -> Result<(), Error> {
        let s = fs::read_to_string(self.path.clone())?;

        let json: MapFileJson = serde_json::from_str(&s)?;

        self.data = collect_data(&json);

        Ok(())
    }

    pub fn get_entry(&self, start: usize, end: usize) -> Option<&MapFileEntry> {
        let entries: Vec<_> = self.data.values(start..end).collect();

        match entries.get(0) {
            Some(entry) => Some(*entry),
            None => None,
        }
    }
}

fn collect_data(json: &MapFileJson) -> IntervalMap<usize, MapFileEntry> {
    let mut ret: IntervalMap<usize, MapFileEntry> = IntervalMap::new();

    for segment in &json.segments {
        for file in &segment.files {
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
                    symbol_vram: symbol.vram,
                    symbol_vrom: symbol.vrom.unwrap(),
                    symbol_size: symbol.size.unwrap(),
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
