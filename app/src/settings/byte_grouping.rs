use eframe::egui;
use eframe::emath::Numeric;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
        let str = match self {
            Self::One => "One",
            Self::Two => "Two",
            Self::Four => "Four",
            Self::Eight => "Eight",
            Self::Sixteen => "Sixteen",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

impl Numeric for ByteGrouping {
    const INTEGRAL: bool = false;
    const MIN: Self = Self::One;
    const MAX: Self = Self::Sixteen;

    fn to_f64(self) -> f64 {
        match self {
            ByteGrouping::One => 1.0,
            ByteGrouping::Two => 2.0,
            ByteGrouping::Four => 4.0,
            ByteGrouping::Eight => 8.0,
            ByteGrouping::Sixteen => 16.0,
        }
    }

    fn from_f64(value: f64) -> Self {
        match value.round() as usize {
            1 => ByteGrouping::One,
            2 => ByteGrouping::Two,
            4 => ByteGrouping::Four,
            8 => ByteGrouping::Eight,
            16 => ByteGrouping::Sixteen,
            _ => ByteGrouping::Eight,
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

pub fn byte_grouping_slider(ui: &mut egui::Ui, byte_grouping: &mut ByteGrouping) {
    ui.add(
        egui::Slider::new(byte_grouping, ByteGrouping::One..=ByteGrouping::Sixteen)
            .text("Byte Grouping")
            .logarithmic(true)
            .integer(),
    );
}
