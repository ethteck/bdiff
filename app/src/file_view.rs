use anyhow::Error;
use bdiff_hex_view::{CursorState, HexView, HexViewSelectionState};
use eframe::{
    egui::{self, Id},
    epaint::Color32,
};

use crate::tools::data_viewer::DataViewer;
use crate::tools::string_viewer::StringViewer;
use crate::{
    bin_file::{read_file_bytes, BinFile, Endianness},
    diff_state::DiffState,
    settings::Settings,
    tools::symbol_tool::SymbolTool,
};

pub struct FileView {
    pub id: usize,
    pub file: BinFile,
    pub cur_pos: usize,
    pub pos_locked: bool,
    pub show_selection_info: bool,
    pub show_cursor_info: bool,
    pub hv: HexView,
    sv: StringViewer,
    dv: DataViewer,
    pub mt: SymbolTool,
    pub closed: bool,
}

impl FileView {
    pub fn new(file: BinFile, id: usize, bytes_per_row: usize, num_rows: usize) -> Self {
        Self {
            id,
            file,
            cur_pos: 0,
            pos_locked: false,
            show_selection_info: true,
            show_cursor_info: true,
            hv: HexView::new(id, bytes_per_row, num_rows),
            sv: StringViewer::default(),
            dv: DataViewer::default(),
            mt: SymbolTool::default(),
            closed: false,
        }
    }

    pub fn reload_file(&mut self) -> Result<(), Error> {
        self.file.data = read_file_bytes(self.file.path.clone())?;

        if self.hv.selection.range.first >= self.file.data.len()
            && self.hv.selection.range.second >= self.file.data.len()
        {
            self.hv.selection.clear();
        } else {
            self.hv.selection.range.first =
                self.hv.selection.range.first.min(self.file.data.len() - 1);
            self.hv.selection.range.second =
                self.hv.selection.range.second.min(self.file.data.len() - 1);
        }
        Ok(())
    }

    pub fn show(
        &mut self,
        settings: &Settings,
        diff_state: &DiffState,
        ctx: &egui::Context,
        cursor_state: CursorState,
        can_selection_change: bool,
        global_view_pos: usize,
    ) {
        egui::Window::new(self.file.path.to_str().unwrap())
            .id(Id::new(format!("hex_view_window_{}", self.id)))
            .title_bar(false)
            .show(ctx, |ui| {
                ui.with_layout(
                    egui::Layout::left_to_right(eframe::emath::Align::Min),
                    |ui| {
                        // Truncate file_name with leading ellipsis
                        let name_limit = 50;
                        let file_name = self.file.path.as_path().to_str().unwrap();
                        let file_name_brief = if file_name.len() > name_limit {
                            format!("...{}", &file_name[file_name.len() - name_limit - 3..])
                        } else {
                            file_name.to_owned()
                        };
                        ui.label(
                            egui::RichText::new(file_name_brief)
                                .monospace()
                                .size(14.0)
                                .color(Color32::LIGHT_GRAY),
                        )
                            .on_hover_text(egui::RichText::new(file_name));

                        let (lock_text, hover_text) = match self.pos_locked {
                            true => (
                                egui::RichText::new(egui_phosphor::regular::LOCK_SIMPLE)
                                    .color(Color32::RED),
                                "Unlock scroll position",
                            ),
                            false => (
                                egui::RichText::new(egui_phosphor::regular::LOCK_SIMPLE_OPEN)
                                    .color(Color32::GREEN),
                                "Lock scroll position",
                            ),
                        };
                        if ui.button(lock_text).on_hover_text(hover_text).clicked() {
                            self.pos_locked = !self.pos_locked;
                        }

                        match self.file.endianness {
                            Endianness::Little => {
                                if ui
                                    .button("LE")
                                    .on_hover_text("Switch to big-endian")
                                    .clicked()
                                {
                                    self.file.endianness = Endianness::Big;
                                }
                            }
                            Endianness::Big => {
                                if ui
                                    .button("BE")
                                    .on_hover_text("Switch to little-endian")
                                    .clicked()
                                {
                                    self.file.endianness = Endianness::Little;
                                }
                            }
                        }

                        ui.menu_button("...", |ui| {
                            ui.checkbox(&mut self.show_selection_info, "Selection info");
                            ui.checkbox(&mut self.show_cursor_info, "Cursor info");
                            ui.checkbox(&mut self.dv.show, "Data viewer");
                            ui.checkbox(&mut self.sv.show, "String viewer");
                            ui.checkbox(&mut self.mt.show, "Symbols");
                        });

                        if ui.button("X").on_hover_text("Close").clicked() {
                            self.closed = true;
                        }
                    },
                );

                ui.with_layout(
                    egui::Layout::left_to_right(eframe::emath::Align::Min),
                    |ui: &mut egui::Ui| {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                let diffs = match settings.diff_state_enabled {
                                    true => Some(&diff_state.diffs[..]),
                                    false => None
                                };
                                self.hv.show(
                                    ui,
                                    &self.file.data,
                                    global_view_pos as isize - self.cur_pos as isize, // TODO pass just the slice of data we care about, don't keep track of global pos inside hv
                                    cursor_state,
                                    can_selection_change,
                                    settings.byte_grouping.into(),
                                    diffs,
                                );
                            });

                            if self.show_selection_info {
                                let selection_text = match self.hv.selection.state {
                                    HexViewSelectionState::None => "No selection".to_owned(),
                                    _ => {
                                        let start = self.hv.selection.start();
                                        let end = self.hv.selection.end();
                                        let length = end - start + 1;

                                        let map_entry = match self.mt.map_file {
                                            Some(ref map_file) => {
                                                map_file.get_entry(start, end + 1)
                                            }
                                            None => None,
                                        };

                                        let beginning = match length {
                                            1 => {
                                                format!("Selection: 0x{:X}", start)
                                            }
                                            _ => {
                                                format!(
                                                    "Selection: 0x{:X} - 0x{:X} (len 0x{:X})",
                                                    start, end, length
                                                )
                                            }
                                        };

                                        match map_entry {
                                            Some(entry) => {
                                                format!(
                                                    "{} ({} + 0x{})",
                                                    beginning,
                                                    entry.symbol_name,
                                                    start - entry.symbol_vrom
                                                )
                                            }
                                            None => beginning,
                                        }
                                    }
                                };
                                ui.label(egui::RichText::new(selection_text).monospace());
                            }

                            if self.show_cursor_info {
                                let hover_text = match self.hv.cursor_pos {
                                    Some(pos) => {
                                        let map_entry = match self.mt.map_file {
                                            Some(ref map_file) => map_file.get_entry(pos, pos + 1),
                                            None => None,
                                        };

                                        match map_entry {
                                            Some(entry) => {
                                                format!(
                                                    "Cursor: 0x{:X} ({} + 0x{})",
                                                    pos,
                                                    entry.symbol_name,
                                                    pos - entry.symbol_vrom
                                                )
                                            }
                                            None => format!("Cursor: 0x{:X}", pos),
                                        }
                                    }
                                    None => "Not hovering".to_owned(),
                                };
                                ui.label(egui::RichText::new(hover_text).monospace());
                            }
                        });

                        ui.with_layout(egui::Layout::top_down(eframe::emath::Align::Min), |ui| {
                            self.dv.display(
                                ui,
                                self.id,
                                self.hv.get_selected_bytes(&self.file.data),
                                self.file.endianness,
                            );
                            self.sv.display(
                                ui,
                                self.id,
                                self.hv.get_selected_bytes(&self.file.data),
                                self.file.endianness,
                            );
                            self.mt.display(ui);
                        });
                    },
                );
            });
    }
}
