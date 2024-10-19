use crate::settings::ui::color_selection;
use bdiff_hex_view::HexViewStyle;
use eframe::egui::{self};
use eframe::egui::{Align, Layout};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[serde(rename_all = "lowercase")]
pub enum VisualTheme {
    Decompme,
    Dark,
    Light,
}

impl Display for VisualTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Decompme => "decomp.me",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Clone)]
pub struct ThemeSettings {
    pub active_theme: VisualTheme,
    pub hex_view_style: HexViewStyle,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            active_theme: VisualTheme::Decompme,
            hex_view_style: HexViewStyle::default(),
        }
    }
}
pub fn show_theme_settings(ctx: &egui::Context, settings: &mut ThemeSettings) -> bool {
    let mut closed = false;

    egui::Window::new("theme").title_bar(false).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Theme");
            if ui.button("X").on_hover_text("Close").clicked() {
                closed = true;
            }
        });

        // Font Colors
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                egui::CollapsingHeader::new("Offset Colors")
                    .default_open(true)
                    .show(ui, |ui| {
                        color_selection(
                            ui,
                            "Offset text color",
                            &mut settings.hex_view_style.offset_text_color,
                        );

                        color_selection(
                            ui,
                            "Leading zero color",
                            &mut settings.hex_view_style.offset_leading_zero_color,
                        );
                    });

                egui::CollapsingHeader::new("Hex Area Colors")
                    .default_open(true)
                    .show(ui, |ui| {
                        color_selection(
                            ui,
                            "Selection color",
                            &mut settings.hex_view_style.selection_color,
                        );
                        color_selection(ui, "Diff color", &mut settings.hex_view_style.diff_color);
                        color_selection(
                            ui,
                            "Null color",
                            &mut settings.hex_view_style.hex_null_color,
                        );
                        color_selection(
                            ui,
                            "Other color",
                            &mut settings.hex_view_style.other_hex_color,
                        );
                    });

                egui::CollapsingHeader::new("Ascii Area Colors")
                    .default_open(true)
                    .show(ui, |ui| {
                        color_selection(
                            ui,
                            "Null color",
                            &mut settings.hex_view_style.ascii_null_color,
                        );

                        color_selection(
                            ui,
                            "Ascii color",
                            &mut settings.hex_view_style.ascii_color,
                        );

                        color_selection(
                            ui,
                            "Other color",
                            &mut settings.hex_view_style.other_ascii_color,
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

    closed
}
