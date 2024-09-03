use crate::settings;
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use crate::{
    bin_file::BinFile,
    config::{read_json_config, write_json_config, Config, FileConfig},
    diff_state::DiffState,
    file_view::FileView,
    settings::{read_json_settings, show_theme_settings, write_json_settings, Settings},
};
use anyhow::Error;
use bdiff_hex_view::{CursorState, HexViewSelection, HexViewSelectionSide, HexViewSelectionState};
use eframe::egui::{Align, Layout, Modifiers, RichText, Ui};
use eframe::{
    egui::{self, Checkbox, Context, Style, ViewportCommand},
    epaint::{Rounding, Shadow},
};
use egui_modal::Modal;

#[derive(Default)]
struct GotoModal {
    value: String,
    status: String,
}

#[derive(Default)]
struct OverwriteModal {
    open: bool,
}

struct Options {
    mirror_selection: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            mirror_selection: true,
        }
    }
}

#[derive(Default)]
pub struct BdiffApp {
    next_hv_id: usize,
    file_views: Vec<FileView>,
    goto_modal: GotoModal,
    overwrite_modal: OverwriteModal,
    scroll_overflow: f32,
    options: Options,
    global_selection: HexViewSelection, // the selection that all hex views will mirror
    selecting_hv: Option<usize>,
    last_selected_hv: Option<usize>,
    bottom_bar_open: bool,
    settings: Settings,
    config: Config,
    started_with_arguments: bool,
    diff_state: DiffState,
}

impl BdiffApp {
    pub fn new(cc: &eframe::CreationContext<'_>, paths: Vec<PathBuf>) -> Self {
        set_up_custom_fonts(&cc.egui_ctx);

        let hex_views = Vec::new();

        let settings = if let Ok(settings) = read_json_settings() {
            settings
        } else {
            let sett = Settings::default();
            write_json_settings(&sett)
                .expect("Failed to write empty settings to the settings file!");
            sett
        };

        let started_with_arguments = !paths.is_empty();

        let mut ret = Self {
            next_hv_id: 0,
            file_views: hex_views,
            settings,
            started_with_arguments,
            ..Default::default()
        };

        log::info!("Loading project config from file");
        let config_path = Path::new("bdiff.json");

        let config = if started_with_arguments {
            let file_configs = paths
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<FileConfig>>();

            Config {
                files: file_configs,
                changed: true,
            }
        } else if config_path.exists() {
            read_json_config(config_path).unwrap()
        } else {
            Config::default()
        };

        for file in config.files.iter() {
            match ret.open_file(&file.path) {
                Ok(fv) => {
                    if let Some(map) = file.map.as_ref() {
                        fv.mt.load_file(map);
                    }
                }
                Err(e) => {
                    log::error!("Failed to open file: {}", e);
                }
            }
        }

        ret.config = config;

        ret
    }

    pub fn open_file(&mut self, path: &Path) -> Result<&mut FileView, Error> {
        let file = BinFile::from_path(path)?;
        self.config.files.push(path.into());
        self.config.changed = true;

        let fv = FileView::new(file, self.next_hv_id);
        self.file_views.push(fv);
        self.next_hv_id += 1;

        self.recalculate_diffs();

        Ok(self.file_views.last_mut().unwrap())
    }

    fn get_hex_view_by_id(&mut self, id: usize) -> Option<&mut FileView> {
        self.file_views.iter_mut().find(|fv| fv.id == id)
    }

    fn handle_hex_view_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.modifiers.shift) {
            // Move selection
            if let Some(fv) = self.last_selected_hv {
                if let Some(fv) = self.get_hex_view_by_id(fv) {
                    let mut changed = false;
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft))
                        && fv.hv.selection.start() > 0
                        && fv.hv.selection.end() > 0
                    {
                        fv.hv.selection.adjust_cur_pos(-1);
                        changed = true;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight))
                        && fv.hv.selection.start() < fv.file.data.len() - 1
                        && fv.hv.selection.end() < fv.file.data.len() - 1
                    {
                        fv.hv.selection.adjust_cur_pos(1);
                        changed = true;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp))
                        && fv.hv.selection.start() >= fv.bytes_per_row
                        && fv.hv.selection.end() >= fv.bytes_per_row
                    {
                        fv.hv.selection.adjust_cur_pos(-(fv.bytes_per_row as isize));
                        changed = true;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown))
                        && fv.hv.selection.start() < fv.file.data.len() - fv.bytes_per_row
                        && fv.hv.selection.end() < fv.file.data.len() - fv.bytes_per_row
                    {
                        fv.hv.selection.adjust_cur_pos(fv.bytes_per_row as isize);
                        changed = true;
                    }

                    if changed {
                        self.global_selection = fv.hv.selection.clone();
                    }
                }
            }
        } else {
            // Move view
            let prev_positions: Vec<usize> = self
                .file_views
                .iter()
                .map(|fv| fv.cur_pos)
                .collect::<Vec<usize>>();

            let diffing = self.file_views.len() > 1 && self.diff_state.enabled;

            for fv in self.file_views.iter_mut() {
                // Keys
                if ctx.input(|i| i.key_pressed(egui::Key::Home)) {
                    fv.hv.set_cur_pos(&fv.file.data, 0);
                }
                if ctx.input(|i| i.key_pressed(egui::Key::End))
                    && fv.file.data.len() >= fv.hv.bytes_per_screen(&fv.file.data)
                {
                    fv.hv.set_cur_pos(
                        &fv.file.data,
                        fv.file.data.len() - fv.hv.bytes_per_screen(&fv.file.data),
                    )
                }
                if ctx.input(|i| i.key_pressed(egui::Key::PageUp)) {
                    fv.hv.adjust_cur_pos(
                        &fv.file.data,
                        -(fv.hv.bytes_per_screen(&fv.file.data) as isize),
                    )
                }
                if ctx.input(|i| i.key_pressed(egui::Key::PageDown)) {
                    fv.hv.adjust_cur_pos(
                        &fv.file.data,
                        fv.hv.bytes_per_screen(&fv.file.data) as isize,
                    )
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    fv.hv.adjust_cur_pos(&fv.file.data, -1)
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    fv.hv.adjust_cur_pos(&fv.file.data, 1)
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    fv.hv
                        .adjust_cur_pos(&fv.file.data, -(fv.bytes_per_row as isize))
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    fv.hv
                        .adjust_cur_pos(&fv.file.data, fv.bytes_per_row as isize)
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let last_byte = fv.cur_pos + fv.hv.bytes_per_screen(&fv.file.data);

                    if diffing {
                        if last_byte < fv.file.data.len() {
                            match self.diff_state.get_next_diff(last_byte) {
                                Some(next_diff) => {
                                    // Move to the next diff
                                    let new_pos = next_diff - (next_diff % fv.bytes_per_row);
                                    fv.hv.set_cur_pos(&fv.file.data, new_pos);
                                }
                                None => {
                                    // Move to the end of the file
                                    if fv.file.data.len() >= fv.hv.bytes_per_screen(&fv.file.data) {
                                        fv.hv.set_cur_pos(
                                            &fv.file.data,
                                            fv.file.data.len()
                                                - fv.hv.bytes_per_screen(&fv.file.data),
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        // Move one screen down
                        fv.hv.adjust_cur_pos(
                            &fv.file.data,
                            fv.hv.bytes_per_screen(&fv.file.data) as isize,
                        )
                    }
                }

                let scroll_y = ctx.input(|i| i.raw_scroll_delta.y);

                // Scrolling
                if scroll_y != 0.0 {
                    let lines_per_scroll = 1;
                    let scroll_threshold = 20; // One tick of the scroll wheel for me
                    let scroll_amt: isize;

                    if scroll_y.abs() >= scroll_threshold as f32 {
                        // Scroll wheels / very fast scrolling
                        scroll_amt = scroll_y as isize / scroll_threshold;
                        self.scroll_overflow = 0.0;
                    } else {
                        // Trackpads - Accumulate scroll amount until it reaches the threshold
                        self.scroll_overflow += scroll_y;
                        scroll_amt = self.scroll_overflow as isize / scroll_threshold;
                        if scroll_amt != 0 {
                            self.scroll_overflow -= (scroll_amt * scroll_threshold) as f32;
                        }
                    }
                    fv.hv.adjust_cur_pos(
                        &fv.file.data,
                        -scroll_amt * lines_per_scroll * fv.bytes_per_row as isize,
                    )
                }
            }

            // If any of the current positions are different from the previous ones
            let has_new_positions = self
                .file_views
                .iter()
                .zip(prev_positions.iter())
                .any(|(fv, &prev_pos)| fv.cur_pos != prev_pos);

            // and if any hex views are also locked, we need to recalculate the diff
            if has_new_positions && self.file_views.iter().any(|fv| fv.pos_locked) {
                self.recalculate_diffs()
            }
        }
    }
}

fn set_up_custom_fonts(ctx: &Context) {
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

    // for egui-phosphor
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    // Finally store all the changes we made
    ctx.set_fonts(fonts);
}

impl eframe::App for BdiffApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style: Style = (*ctx.style()).clone();
        style.visuals.popup_shadow = Shadow::NONE;
        style.visuals.window_shadow = Shadow::NONE;
        style.visuals.menu_rounding = Rounding::default();
        style.visuals.window_rounding = Rounding::default();
        style.interaction.selectable_labels = false;
        style.interaction.multi_widget_text_select = false;
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

        // Goto modal
        goto_modal.show(|ui| {
            self.show_goto_modal(&goto_modal, ui, ctx);
        });

        let overwrite_modal: Modal = Modal::new(ctx, "overwrite_modal");

        if self.overwrite_modal.open {
            self.show_overwrite_modal(&overwrite_modal);
            overwrite_modal.open();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            if goto_modal.is_open() {
                goto_modal.close();
            } else {
                self.goto_modal.value = "0x".to_owned();
                goto_modal.open();
            }
        }

        // Standard HexView input
        if !(overwrite_modal.is_open() || goto_modal.is_open()) {
            self.handle_hex_view_input(ctx);
        }

        // Open dropped files
        if ctx.input(|i| !i.raw.dropped_files.is_empty()) {
            for file in ctx.input(|i| i.raw.dropped_files.clone()) {
                let _ = self.open_file(&file.path.unwrap());
            }
        }

        // Copy selection
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
            let mut selection = String::new();

            for fv in self.file_views.iter() {
                if self.last_selected_hv.is_some() && fv.id == self.last_selected_hv.unwrap() {
                    let selected_bytes = fv.hv.get_selected_bytes(&fv.file.data);

                    let selected_bytes: String = match fv.hv.selection.side {
                        HexViewSelectionSide::Hex => selected_bytes
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<String>>()
                            .join(" "),
                        HexViewSelectionSide::Ascii => {
                            String::from_utf8_lossy(selected_bytes).to_string()
                        }
                    };
                    // convert selected_bytes to an ascii string

                    selection.push_str(&selected_bytes.to_string());
                }
            }

            if !selection.is_empty() {
                ctx.output_mut(|o| o.copied_text = selection);
            }
        }

        // Menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let _ = self.open_file(&path);
                        }

                        ui.close_menu();
                    }
                    if ui.button("Save Workspace").clicked() {
                        if self.config.changed {
                            if self.started_with_arguments {
                                self.overwrite_modal.open = true;
                            } else {
                                write_json_config("bdiff.json", &self.config)
                                    .expect("Failed to write config");
                                self.config.changed = false;
                            };
                        }
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close)
                    }
                });

                ui.menu_button("Action", |ui| {
                    if ui.button("Go to address (G)").clicked() {
                        self.goto_modal.value = "0x".to_owned();
                        goto_modal.open();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Preferences", |ui| {
                    if ui.button("Theme Settings").clicked() {
                        self.settings.theme_menu_open = !self.settings.theme_menu_open;
                    }
                });

                ui.menu_button("Options", |ui| {
                    let diff_checkbox = Checkbox::new(&mut self.diff_state.enabled, "Display diff");
                    let mirror_selection_checkbox = Checkbox::new(
                        &mut self.options.mirror_selection,
                        "Mirror selection across files",
                    );

                    ui.label("Diff View");

                    settings::byte_grouping_slider(ui, &mut self.settings.byte_grouping);

                    if ui
                        .add_enabled(self.file_views.len() > 1, diff_checkbox)
                        .clicked()
                        && self.diff_state.enabled
                    {
                        //self.recalculate_diffs()
                    }

                    ui.add_enabled(self.file_views.len() > 1, mirror_selection_checkbox);

                    ui.separator();
                    ui.label("General Interface");
                    ui.add(Checkbox::new(
                        &mut self.bottom_bar_open,
                        "Show Quick Access bar",
                    ));
                });
            })
        });

        // Quick Access bar
        if self.bottom_bar_open {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(Align::LEFT), |ui| {
                    settings::byte_grouping_slider(ui, &mut self.settings.byte_grouping);
                });
            });
        }

        // Reload changed files
        let mut calc_diff = false;

        // Main panel
        for fv in self.file_views.iter_mut() {
            let cur_sel = fv.hv.selection.clone();
            let can_selection_change = match self.selecting_hv {
                Some(id) => id == fv.id,
                None => true,
            };
            fv.show(
                &mut self.config,
                &self.settings,
                &self.diff_state,
                ctx,
                cursor_state,
                can_selection_change,
            );
            if fv.hv.selection != cur_sel {
                match fv.hv.selection.state {
                    HexViewSelectionState::Selecting => {
                        self.selecting_hv = Some(fv.id);
                        self.last_selected_hv = Some(fv.id);
                    }
                    _ => {
                        self.selecting_hv = None;
                    }
                }
                self.global_selection = fv.hv.selection.clone();
            }

            if cursor_state == CursorState::Released {
                // If we released the mouse button somewhere else, end the selection
                // The state wouldn't be Selecting if we had captured the release event inside the fv
                if fv.hv.selection.state == HexViewSelectionState::Selecting {
                    fv.hv.selection.state = HexViewSelectionState::Selected;
                }
            }
        }

        if cursor_state == CursorState::Released {
            self.selecting_hv = None;
            if self.global_selection.state == HexViewSelectionState::Selecting {
                self.global_selection.state = HexViewSelectionState::Selected;
            }
        }

        if self.options.mirror_selection {
            for fv in self.file_views.iter_mut() {
                if fv.hv.selection != self.global_selection {
                    fv.hv.selection = self.global_selection.clone();
                    if fv.hv.selection.start() >= fv.file.data.len()
                        || fv.hv.selection.end() >= fv.file.data.len()
                    {
                        fv.hv.selection.clear()
                    }
                }
            }
        }

        // Delete any closed hex views
        self.file_views.retain(|fv| {
            calc_diff = calc_diff || fv.closed;
            let delete: bool = { fv.closed };

            if let Some(id) = self.last_selected_hv {
                if delete && fv.id == id {
                    self.last_selected_hv = None;
                }
            }

            !delete
        });

        // If we have no hex views left, don't keep track of any selection
        if self.file_views.is_empty() {
            self.global_selection.clear();
        }

        // File reloading
        for fv in self.file_views.iter_mut() {
            if fv.file.modified.swap(false, Ordering::Relaxed) {
                match fv.reload_file() {
                    Ok(_) => {
                        log::info!("Reloaded file {}", fv.file.path.display());
                        calc_diff = true;
                    }
                    Err(e) => {
                        log::error!("Failed to reload file: {}", e);
                    }
                }
            }

            if fv.mt.map_file.is_some() {
                let map_file = fv.mt.map_file.as_mut().unwrap();
                if map_file.modified.swap(false, Ordering::Relaxed) {
                    match map_file.reload() {
                        Ok(_) => {
                            log::info!("Reloaded map file {}", map_file.path.display());
                        }
                        Err(e) => {
                            log::error!("Failed to reload map file: {}", e);
                        }
                    }
                }
            }
        }

        if calc_diff {
            self.recalculate_diffs();
        }

        if self.settings.theme_menu_open {
            show_theme_settings(ctx, &mut self.settings);
        }
    }
}

impl BdiffApp {
    fn recalculate_diffs(&mut self) {
        let files: &[&[u8]] = &self
            .file_views
            .iter()
            .map(|fv| fv.file.data.as_slice())
            .collect::<Vec<&[u8]>>();
        self.diff_state.recalculate(files);
    }

    fn show_overwrite_modal(&mut self, modal: &Modal) {
        modal.show(|ui| {
            modal.title(ui, "Overwrite previous config");
            ui.label(&format!(
                "By saving, you are going to overwrite existing configuration file at \"{}\".",
                "./bdiff.json"
            ));
            ui.label("Are you sure you want to proceed?");

            modal.buttons(ui, |ui| {
                if ui.button("Overwrite").clicked() {
                    write_json_config("bdiff.json", &self.config).unwrap();
                    self.config.changed = false;
                    self.overwrite_modal.open = false;
                }
                if ui.button("Cancel").clicked() {
                    modal.close();
                    self.overwrite_modal.open = false;
                }
            });
        });
    }

    fn show_goto_modal(&mut self, goto_modal: &Modal, ui: &mut Ui, ctx: &Context) {
        goto_modal.title(ui, "Go to address");
        ui.label("Enter a hex address to go to");

        ui.text_edit_singleline(&mut self.goto_modal.value)
            .request_focus();

        ui.label(RichText::new(self.goto_modal.status.clone()).color(egui::Color32::RED));

        goto_modal.buttons(ui, |ui| {
            if ui.button("Go").clicked()
                || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Enter))
            {
                let pos: Option<usize> = parse_int::parse(&self.goto_modal.value).ok();

                match pos {
                    Some(pos) => {
                        for fv in self.file_views.iter_mut() {
                            fv.hv.set_cur_pos(&fv.file.data, pos);
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
                self.goto_modal.status = "".to_owned();
                goto_modal.close();
            };

            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) {
                goto_modal.close();
            }
        });
    }
}
