use std::{path::PathBuf, sync::atomic::Ordering};

use eframe::{
    egui::{self},
    epaint::{Rounding, Shadow},
};

use crate::{hex_view::HexView, read_file, BinFile};

#[derive(Default)]
pub struct BdiffApp {
    hex_views: Vec<HexView>,
}

impl BdiffApp {
    pub fn new(cc: &eframe::CreationContext<'_>, files: Vec<BinFile>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let mut hex_views = Vec::new();
        for file in files {
            let num_rows = (file.data.len() / 0x10).clamp(10, 25) as u32;

            hex_views.push(HexView {
                file,
                num_rows,
                bytes_per_row: 0x10,
                ..Default::default()
            });
        }
        Self { hex_views }
    }

    fn open_file(&mut self, path: PathBuf) {
        let file: BinFile = read_file(path.clone()).unwrap();

        self.hex_views.push(HexView::new(file));
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    let monospace_key = "jetbrains-mono";
    let string_key = "noto-sans-jp";

    fonts.font_data.insert(
        monospace_key.to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/jetbrains/JetBrainsMonoNL-Regular.ttf"
        )),
    );

    fonts.font_data.insert(
        string_key.to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/noto/NotoSansJP-Regular.ttf"
        )),
    );

    // Put custom fonts first (highest priority):
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, monospace_key.to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, string_key.to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

impl eframe::App for BdiffApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut style: egui::Style = (*ctx.style()).clone();
        style.visuals.popup_shadow = Shadow {
            extrusion: 0.0,
            color: egui::Color32::TRANSPARENT,
        };
        style.visuals.menu_rounding = Rounding::default();
        ctx.set_style(style);

        let cursor_state: CursorState = ctx.input(|i| {
            if i.pointer.primary_pressed() {
                CursorState::Pressed
            } else if i.pointer.primary_down() {
                CursorState::StillDown
            } else if i.pointer.primary_released() {
                CursorState::Released
            } else {
                CursorState::Hovering
            }
        });

        // TODO don't hard-code the 0th hex_view
        let target_hv = &mut self.hex_views[0];

        ctx.input(|i| {
            // Keys
            if i.key_pressed(egui::Key::Home) {
                target_hv.set_cur_pos(0);
            }
            if i.key_pressed(egui::Key::End)
                && target_hv.file.data.len() >= target_hv.bytes_per_screen()
            {
                target_hv.set_cur_pos(target_hv.file.data.len() - target_hv.bytes_per_screen())
            }
            if i.key_pressed(egui::Key::PageUp) {
                target_hv.adjust_cur_pos(-(target_hv.bytes_per_screen() as i32))
            }
            if i.key_pressed(egui::Key::PageDown) {
                target_hv.adjust_cur_pos(target_hv.bytes_per_screen() as i32)
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                target_hv.adjust_cur_pos(-1)
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                target_hv.adjust_cur_pos(1)
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                target_hv.adjust_cur_pos(-(target_hv.bytes_per_row as i32))
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                target_hv.adjust_cur_pos(target_hv.bytes_per_row as i32)
            }
            if i.key_pressed(egui::Key::Enter) {
                target_hv.adjust_cur_pos(target_hv.bytes_per_screen() as i32)
            }

            // Mouse
            if i.scroll_delta.y != 0.0 {
                let scroll_amt = -(i.scroll_delta.y as i32 / 50);
                let lines_per_scroll = 1;
                target_hv
                    .adjust_cur_pos(scroll_amt * lines_per_scroll * target_hv.bytes_per_row as i32)
            }

            if i.pointer.middle_down() {
                target_hv.selection.clear();
            }
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.open_file(path);
                        }
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.menu_button("Options", |ui| {
                    if ui.button("Options").clicked() {
                        ui.close_menu();
                    }
                });
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            for hv in self.hex_views.iter_mut() {
                hv.show(ctx, ui, cursor_state);
            }
        });

        for hv in self.hex_views.iter_mut() {
            if hv.file.modified.swap(false, Ordering::Relaxed) {
                let _ = hv.file.reload();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorState {
    Hovering,
    Pressed,
    StillDown,
    Released,
}
