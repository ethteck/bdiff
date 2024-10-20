use crate::tools::data_viewer::DataViewer;
use crate::tools::string_viewer::StringViewer;
use crate::{
    bin_file::{read_file_bytes, BinFile, Endianness},
    diff_state::DiffState,
    settings::Settings,
    tools::symbol_tool::SymbolTool,
};
use anyhow::Error;
use bdiff_hex_view::cursor_state::CursorState;
use bdiff_hex_view::selection::HexViewSelectionState;
use bdiff_hex_view::{HexView, HexViewOptions, HexViewState};
use eframe::{
    egui::{self, Id},
    epaint::Color32,
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
    pub st: SymbolTool,
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
            st: SymbolTool::default(),
            closed: false,
        }
    }

    pub fn reload_file(&mut self) -> Result<(), Error> {
        self.file.data = read_file_bytes(self.file.path.clone())?;

        if self.hv.selection.start() >= self.file.data.len()
            && self.hv.selection.end() >= self.file.data.len()
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
        ctx: &egui::Context,
        settings: &Settings,
        diff_state: &DiffState,
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
                            format!("...{}", &file_name[file_name.len() - name_limit..])
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
                                "Unlock file position",
                            ),
                            false => (
                                egui::RichText::new(egui_phosphor::regular::LOCK_SIMPLE_OPEN)
                                    .color(Color32::GREEN),
                                "Lock file position",
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
                            ui.checkbox(&mut self.st.show, "Symbols");
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
                                let diffs = match settings.diff_enabled {
                                    true => Some(&diff_state.diffs[..]),
                                    false => None,
                                };

                                let num_offset_digits = match self.file.data.len() {
                                    //0..=0xFFFF => 4,
                                    0x10000..=0xFFFFFFFF => 8,
                                    0x100000000..=0xFFFFFFFFFFFF => 12,
                                    _ => 8,
                                };

                                self.hv.show(
                                    ui,
                                    &HexViewState {
                                        file_data: &self.file.data,
                                        file_pos: self.cur_pos,
                                        global_pos: global_view_pos,
                                        diffs,
                                    },
                                    CursorState::get(ctx),
                                    HexViewOptions {
                                        can_selection_change,
                                        byte_grouping: settings.byte_grouping,
                                        num_offset_digits,
                                    },
                                );
                            });

                            if self.show_selection_info {
                                let selection_text = match self.hv.selection.state {
                                    HexViewSelectionState::None => "No selection".to_owned(),
                                    _ => {
                                        // Convert to file coords
                                        let start = self.hv.selection.start() as isize
                                            - self.cur_pos as isize;
                                        let end = self.hv.selection.end() as isize
                                            - self.cur_pos as isize;
                                        let length = end - start + 1;

                                        let map_entry = match self.st.map_file {
                                            Some(ref map_file) => {
                                                if start >= 0 && end >= 0 {
                                                    map_file
                                                        .get_entry(start as usize, end as usize + 1)
                                                } else {
                                                    None
                                                }
                                            }
                                            None => None,
                                        };

                                        let beginning = match length {
                                            1 => {
                                                format!("Selection: 0x{:X}", start)
                                            }
                                            _ => {
                                                let start_str = match start < 0 {
                                                    true => format!("-0x{:X}", -start),
                                                    false => format!("0x{:X}", start),
                                                };
                                                let end_str = match end < 0 {
                                                    true => format!("-0x{:X}", -end),
                                                    false => format!("0x{:X}", end),
                                                };
                                                format!(
                                                    "Selection: {} - {} (len 0x{:X})",
                                                    start_str, end_str, length
                                                )
                                            }
                                        };

                                        match map_entry {
                                            Some(entry) => {
                                                format!(
                                                    "{} ({} + 0x{})",
                                                    beginning,
                                                    entry.symbol_name,
                                                    start as usize - entry.symbol_vrom
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
                                        if pos < self.cur_pos {
                                            "Not hovering".to_owned()
                                        } else {
                                            // Convert to file position from global position
                                            let pos =
                                                (pos as isize - self.cur_pos as isize) as usize;

                                            let map_entry = match self.st.map_file {
                                                Some(ref map_file) => {
                                                    map_file.get_entry(pos, pos + 1)
                                                }
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
                                self.hv.get_selected_bytes(&self.file.data, self.cur_pos),
                                self.file.endianness,
                            );
                            self.sv.display(
                                ui,
                                self.id,
                                self.hv.get_selected_bytes(&self.file.data, self.cur_pos),
                                self.file.endianness,
                            );
                            self.st.display(ui);
                        });
                    },
                );
            });
    }
}
