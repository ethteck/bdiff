use crate::settings::panels::{color_selection, window_bottom_commands};
use crate::settings::Settings;
use eframe::egui;
use eframe::egui::{Align, Color32, Layout, RichText};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum VisualTheme {
    Decompme,
    Dark,
    Light,
}

impl Display for VisualTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Decompme => "Decomp.me",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ThemeSettings {
    pub active_theme: VisualTheme,

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
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            active_theme: VisualTheme::Decompme,

            // Offset colors
            offset_text_color: Color32::GRAY.into(),
            offset_leading_zero_color: Color32::DARK_GRAY.into(),

            // Hex View colors
            selection_color: Color32::DARK_GREEN.into(),
            diff_color: Color32::RED.into(),
            hex_null_color: Color32::DARK_GRAY.into(),
            other_hex_color: Color32::GRAY.into(),

            // ASCII View colors
            ascii_null_color: Color32::DARK_GRAY.into(),
            ascii_color: Color32::LIGHT_GRAY.into(),
            other_ascii_color: Color32::GRAY.into(),
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

pub fn show_theme_settings(ctx: &egui::Context, settings: &mut Settings) {
    egui::Window::new("Theme Settings")
        .default_open(true)
        .fixed_size((365.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Font Colors
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        ui.add(egui::Label::new(RichText::new("Font Colors").heading()));

                        egui::CollapsingHeader::new("Offset Colors")
                            .default_open(true)
                            .show(ui, |ui| {
                                color_selection(
                                    ui,
                                    "Offset text color",
                                    &mut settings.theme_settings.offset_text_color,
                                );

                                color_selection(
                                    ui,
                                    "Leading zero color",
                                    &mut settings.theme_settings.offset_leading_zero_color,
                                );
                            });

                        egui::CollapsingHeader::new("Hex Area Colors")
                            .default_open(true)
                            .show(ui, |ui| {
                                color_selection(
                                    ui,
                                    "Selection color",
                                    &mut settings.theme_settings.selection_color,
                                );
                                color_selection(
                                    ui,
                                    "Diff color",
                                    &mut settings.theme_settings.diff_color,
                                );
                                color_selection(
                                    ui,
                                    "Null color",
                                    &mut settings.theme_settings.hex_null_color,
                                );
                                color_selection(
                                    ui,
                                    "Other color",
                                    &mut settings.theme_settings.other_hex_color,
                                );
                            });

                        egui::CollapsingHeader::new("Ascii Area Colors")
                            .default_open(true)
                            .show(ui, |ui| {
                                color_selection(
                                    ui,
                                    "Null color",
                                    &mut settings.theme_settings.ascii_null_color,
                                );

                                color_selection(
                                    ui,
                                    "Ascii color",
                                    &mut settings.theme_settings.ascii_color,
                                );

                                color_selection(
                                    ui,
                                    "Other color",
                                    &mut settings.theme_settings.other_ascii_color,
                                );
                            });
                    });
                });

                // /// Visual Theme
                // egui::Frame::group(ui.style()).show(ui, |ui| {
                //     ui.vertical(|ui| {
                //         ui.add(egui::Label::new(RichText::new("Visual Theme").heading()));
                //
                //         for theme in &[VisualTheme::Decompme, VisualTheme::UltraDark, VisualTheme::Light] {
                //             ui.radio_value(
                //                 &mut settings.theme_settings.active_theme,
                //                 theme.clone(),
                //                 theme.to_string(),
                //             );
                //         }
                //     });
                // });
            });

            window_bottom_commands(ui, settings);
        });
}
