use crate::settings::{byte_grouping::ByteGrouping, theme::ThemeSettings};
use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

mod byte_grouping;
pub mod panels;
pub mod theme;

pub use byte_grouping::byte_grouping_slider;
pub use theme::show_theme_settings;

#[derive(Deserialize, Serialize, Default, PartialEq, PartialOrd, Clone)]
pub struct Settings {
    pub byte_grouping: ByteGrouping,
    pub theme_settings: ThemeSettings,

    #[serde(skip)]
    pub theme_menu_open: bool,
}

impl SettingsControl for Settings {
    fn restore_defaults(&mut self) {
        let prev_theme_menu_open = self.theme_menu_open;
        *self = Settings::default();
        // todo dumb hack because the state of the window being open is part of the struct
        self.theme_menu_open = prev_theme_menu_open;
    }

    fn reload(&mut self) {
        let prev_theme_menu_open = self.theme_menu_open;
        *self = read_json_settings().expect("Failed to read settings!");
        // todo dumb hack because the state of the window being open is part of the struct
        self.theme_menu_open = prev_theme_menu_open;
    }

    fn save(&self) {
        write_json_settings(self).expect("Failed to save settings!");
    }
}

pub trait SettingsControl {
    fn restore_defaults(&mut self);
    fn reload(&mut self);
    fn save(&self);
}

pub fn get_settings_path() -> PathBuf {
    let mut path =
        dirs::config_local_dir().expect("Failed to get local configuration dir, report a bug!");
    path.push("bdiff");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create a config folder!");
    }
    path.push("settings.json");
    path
}

pub fn read_json_settings() -> Result<Settings, Error> {
    let settings_path = get_settings_path();
    let mut reader = File::open(&settings_path)
        .with_context(|| format!("Failed to open config file at {}", settings_path.display()))?;
    Ok(serde_json::from_reader(&mut reader)?)
}

pub fn write_json_settings(settings: &Settings) -> Result<(), Error> {
    let settings_path = get_settings_path();
    let mut oo = OpenOptions::new();
    let mut writer = oo
        .create(true)
        .write(true)
        .truncate(true)
        .open(&settings_path)
        .with_context(|| format!("Failed to open config file at {}", settings_path.display()))?;
    Ok(writer.write_all(serde_json::to_string_pretty(settings)?.as_bytes())?)
}
