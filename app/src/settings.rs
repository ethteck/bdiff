use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Error};
use eframe::epaint::Color32;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Settings {
    pub byte_grouping: ByteGrouping,
    pub theme_settings: ThemeSettings,
}

#[derive(Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ByteGrouping {
    One,
    Two,
    Four,
    #[default]
    Eight,
    Sixteen,
}

impl ByteGrouping {
    pub fn get_all_options() -> Vec<ByteGrouping> {
        vec![
            ByteGrouping::One,
            ByteGrouping::Two,
            ByteGrouping::Four,
            ByteGrouping::Eight,
            ByteGrouping::Sixteen,
        ]
    }
}

impl Display for ByteGrouping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "One"),
            Self::Two => write!(f, "Two"),
            Self::Four => write!(f, "Four"),
            Self::Eight => write!(f, "Eight"),
            Self::Sixteen => write!(f, "Sixteen"),
        }
    }
}

impl From<ByteGrouping> for usize {
    fn from(value: ByteGrouping) -> Self {
        match value {
            ByteGrouping::One => 1,
            ByteGrouping::Two => 2,
            ByteGrouping::Four => 4,
            ByteGrouping::Eight => 8,
            ByteGrouping::Sixteen => 16,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Color(pub [u8; 4]);

impl Color {
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8; 4] {
        &mut self.0
    }
}

impl From<Color32> for Color {
    fn from(value: Color32) -> Self {
        Self(value.to_array())
    }
}

impl From<Color> for Color32 {
    fn from(value: Color) -> Self {
        let sc = value.0;
        Color32::from_rgba_premultiplied(sc[0], sc[1], sc[2], sc[3])
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ThemeSettings {
    pub selection_color: Color,

    // Offset colors
    pub offset_text_color: Color,
    pub offset_leading_zero_color: Color,

    // Hex View colors
    pub diff_color: Color,
    pub hex_null_color: Color,
    pub other_hex_color: Color,

    // ASCII View colors
    pub ascii_null_color: Color,
    pub ascii_color: Color,
    pub other_ascii_color: Color,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            offset_text_color: Color32::GRAY.into(),
            offset_leading_zero_color: Color32::DARK_GRAY.into(),

            selection_color: Color32::DARK_GREEN.into(),
            diff_color: Color32::RED.into(),
            hex_null_color: Color32::DARK_GRAY.into(),
            other_hex_color: Color32::GRAY.into(),

            ascii_null_color: Color32::DARK_GRAY.into(),
            ascii_color: Color32::LIGHT_GRAY.into(),
            other_ascii_color: Color32::GRAY.into(),
        }
    }
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
