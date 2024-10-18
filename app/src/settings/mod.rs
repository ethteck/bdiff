use crate::settings::theme::ThemeSettings;
use anyhow::{Context, Error};
use bdiff_hex_view::byte_grouping::ByteGrouping;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

pub mod ui;
pub mod theme;

pub use theme::show_theme_settings;

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Clone)]
pub struct Settings {
    pub mirror_selection: bool,
    pub diff_enabled: bool,
    pub byte_grouping: ByteGrouping,
    pub show_quick_access_bar: bool,
    pub theme: ThemeSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mirror_selection: true,
            diff_enabled: true,
            byte_grouping: ByteGrouping::default(),
            show_quick_access_bar: false,
            theme: ThemeSettings::default(),
        }
    }
}

impl SettingsControl for Settings {
    fn restore_defaults(&mut self) {
        *self = Settings::default();
    }

    fn reload(&mut self) {
        *self = read_json_settings().expect("Failed to read settings!");
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
