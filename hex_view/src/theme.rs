use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Clone)]
pub struct HexViewStyle {
    // Offset colors
    pub offset_text_color: Color,
    pub offset_leading_zero_color: Color,

    // Hex View colors
    pub selection_color: Color,
    pub diff_color: Color,
    pub hex_null_color: Color,
    pub other_hex_color: Color,

    // ASCII View colors
    pub ascii_null_color: Color,
    pub ascii_color: Color,
    pub other_ascii_color: Color,

    pub font_size: f32,
}

impl Default for HexViewStyle {
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

            font_size: 14.0,
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
