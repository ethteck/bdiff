use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use anyhow::Error;
use eframe::{
    egui::{self, Checkbox},
    epaint::{Color32, Rounding, Shadow},
};

use egui_modal::Modal;

use crate::{
    bin_file::BinFile, config::read_json_config, diff_state::DiffState, hex_view::HexView,
};

#[derive(Default)]
struct GotoModal {
    value: String,
    status: String,
}

#[derive(Default)]
pub struct BdiffApp {
    next_hv_id: usize,
    hex_views: Vec<HexView>,
    diff_state: DiffState,
    goto_modal: GotoModal,
}

impl BdiffApp {
    pub fn new(cc: &eframe::CreationContext<'_>, paths: Vec<PathBuf>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let hex_views = Vec::new();

        let mut ret = Self {
            next_hv_id: 0,
            hex_views,
            ..Default::default()
        };

        if !paths.is_empty() {
            for path in paths {
                let _ = ret.open_file(path);
            }
        } else {
            log::info!("Loading project config from file");
            let config_path = Path::new("bdiff.json");

            if config_path.exists() {
                let config = read_json_config(config_path).unwrap();

                for file in config.files {
                    let hv = ret.open_file(file.path).unwrap();

                    if let Some(map) = file.map {
                        hv.mt.load_file(&map);
                    }
                }
            }
        }

        ret.diff_state.recalculate(&ret.hex_views);

        ret
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<&mut HexView, Error> {
        let file = BinFile::from_path(path.clone())?;

        let hv = HexView::new(file, self.next_hv_id);
        self.hex_views.push(hv);
        self.next_hv_id += 1;

        Ok(self.hex_views.last_mut().unwrap())
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorState {
    Hovering,
    Pressed,
    StillDown,
    Released,
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

        let goto_modal: Modal = Modal::new(ctx, "goto_modal");

        // Standard HexView input
        if !goto_modal.is_open() {
            for hv in self.hex_views.iter_mut() {
                if !hv.pos_locked {
                    ctx.input(|i| {
                        // Keys
                        if i.key_pressed(egui::Key::Home) {
                            hv.set_cur_pos(0);
                        }
                        if i.key_pressed(egui::Key::End)
                            && hv.file.data.len() >= hv.bytes_per_screen()
                        {
                            hv.set_cur_pos(hv.file.data.len() - hv.bytes_per_screen())
                        }
                        if i.key_pressed(egui::Key::PageUp) {
                            hv.adjust_cur_pos(-(hv.bytes_per_screen() as isize))
                        }
                        if i.key_pressed(egui::Key::PageDown) {
                            hv.adjust_cur_pos(hv.bytes_per_screen() as isize)
                        }
                        if i.key_pressed(egui::Key::ArrowLeft) {
                            hv.adjust_cur_pos(-1)
                        }
                        if i.key_pressed(egui::Key::ArrowRight) {
                            hv.adjust_cur_pos(1)
                        }
                        if i.key_pressed(egui::Key::ArrowUp) {
                            hv.adjust_cur_pos(-(hv.bytes_per_row as isize))
                        }
                        if i.key_pressed(egui::Key::ArrowDown) {
                            hv.adjust_cur_pos(hv.bytes_per_row as isize)
                        }
                        if i.key_pressed(egui::Key::Enter) {
                            let last_byte = hv.cur_pos + hv.bytes_per_screen() - 1;

                            if self.diff_state.enabled {
                                if last_byte < hv.file.data.len() {
                                    match self.diff_state.get_next_diff(last_byte) {
                                        Some(next_diff) => {
                                            // Move to the next diff
                                            let new_pos =
                                                next_diff - (next_diff % hv.bytes_per_row);
                                            hv.set_cur_pos(new_pos);
                                        }
                                        None => {
                                            // Move to the end of the file
                                            if hv.file.data.len() >= hv.bytes_per_screen() {
                                                hv.set_cur_pos(
                                                    hv.file.data.len() - hv.bytes_per_screen(),
                                                );
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Move one screen down
                                hv.adjust_cur_pos(hv.bytes_per_screen() as isize)
                            }
                        }

                        // Mouse
                        if i.scroll_delta.y != 0.0 {
                            let scroll_amt = -(i.scroll_delta.y as isize / 50);
                            let lines_per_scroll = 1;
                            hv.adjust_cur_pos(
                                scroll_amt * lines_per_scroll * hv.bytes_per_row as isize,
                            )
                        }
                    });
                }
            }
        }

        // Goto modal
        goto_modal.show(|ui| {
            self.show_modal(&goto_modal, ui, ctx);
        });

        if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            if goto_modal.is_open() {
                goto_modal.close();
            } else {
                self.goto_modal.value = "0x".to_owned();
                goto_modal.open();
            }
        }

        // Menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let _ = self.open_file(path);
                            self.diff_state.recalculate(&self.hex_views);
                        }

                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.menu_button("Options", |ui| {
                    let diff_checkbox = Checkbox::new(&mut self.diff_state.enabled, "Display diff");

                    if ui
                        .add_enabled(self.hex_views.len() >= 2, diff_checkbox)
                        .clicked()
                        && self.diff_state.enabled
                    {
                        self.diff_state.recalculate(&self.hex_views);
                    }
                });
            })
        });

        // Reload changed files
        let mut calc_diff = false;

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            for hv in self.hex_views.iter_mut() {
                hv.show(&self.diff_state, ctx, ui, cursor_state);
            }

            self.hex_views.retain(|hv| {
                calc_diff = calc_diff || hv.closed;
                let delete: bool = { hv.closed };
                !delete
            })
        });

        for hv in self.hex_views.iter_mut() {
            if hv.file.modified.swap(false, Ordering::Relaxed) {
                let _ = hv.reload_file();
                calc_diff = true;
            }

            if hv.mt.map_file.is_some() {
                let map_file = hv.mt.map_file.as_mut().unwrap();
                if map_file.modified.swap(false, Ordering::Relaxed) {
                    let _ = map_file.reload();
                }
            }
        }

        if calc_diff {
            self.diff_state.recalculate(&self.hex_views);
        }
    }
}

impl BdiffApp {
    fn show_modal(&mut self, goto_modal: &Modal, ui: &mut egui::Ui, ctx: &egui::Context) {
        goto_modal.title(ui, "Go to address");
        ui.label("Enter a hex address to go to");

        ui.text_edit_singleline(&mut self.goto_modal.value)
            .request_focus();

        ui.label(egui::RichText::new(self.goto_modal.status.clone()).color(Color32::RED));

        goto_modal.buttons(ui, |ui| {
            if ui.button("Go").clicked() || ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                let pos: Option<usize> = parse_int::parse(&self.goto_modal.value).ok();

                match pos {
                    Some(pos) => {
                        for hv in self.hex_views.iter_mut() {
                            hv.set_cur_pos(pos);
                        }
                        goto_modal.close();
                    }
                    None => {
                        self.goto_modal.status = "Invalid address".to_owned();
                        self.goto_modal.value = "0x".to_owned();
                    }
                }
            }

            if goto_modal.button(ui, "Cancel").clicked() {
                goto_modal.close();
            };

            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                goto_modal.close();
            }
        });
    }
}
