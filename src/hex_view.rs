use anyhow::Error;
use eframe::{
    egui::{self, Sense, Separator},
    epaint::Color32,
};

use crate::{
    app::CursorState, bin_file::read_file_bytes, data_viewer::DataViewer,
    string_viewer::StringViewer,
};
use crate::{bin_file::BinFile, spacer::Spacer};

#[derive(Default, Debug, PartialEq)]
enum HexViewSelectionState {
    #[default]
    None,
    Selecting,
    Selected,
}

#[derive(Debug, Default)]
pub struct HexViewSelection {
    first: usize,
    second: usize,
    state: HexViewSelectionState,
}

impl HexViewSelection {
    fn start(&self) -> usize {
        self.first.min(self.second)
    }

    fn end(&self) -> usize {
        self.second.max(self.first)
    }

    fn contains(&self, grid_pos: usize) -> bool {
        self.state != HexViewSelectionState::None
            && grid_pos >= self.start()
            && grid_pos <= self.end()
    }

    pub fn begin(&mut self, grid_pos: usize) {
        self.first = grid_pos;
        self.second = grid_pos;
        self.state = HexViewSelectionState::Selecting;
    }

    pub fn update(&mut self, grid_pos: usize) {
        self.second = grid_pos;
    }

    pub fn finalize(&mut self, grid_pos: usize) {
        self.second = grid_pos;
        self.state = HexViewSelectionState::Selected;
    }

    pub fn clear(&mut self) {
        self.first = 0;
        self.second = 0;
        self.state = HexViewSelectionState::None;
    }
}

#[derive(Default)]
pub struct HexView {
    pub id: usize,
    pub file: BinFile,
    pub num_rows: u32,
    pub bytes_per_row: usize,
    pub cur_pos: usize,
    pub locked: bool,
    pub selection: HexViewSelection,
    sv: StringViewer,
    dv: DataViewer,
}

impl HexView {
    pub fn new(file: BinFile, id: usize) -> Self {
        let min_rows = 10;
        let max_rows = 25;
        let default_bytes_per_row = 0x10;
        let num_rows = (file.data.len() / default_bytes_per_row).clamp(min_rows, max_rows) as u32;

        Self {
            id,
            file,
            num_rows,
            bytes_per_row: default_bytes_per_row,
            ..Default::default()
        }
    }

    pub fn set_cur_pos(&mut self, val: usize) {
        self.cur_pos = val.clamp(0, self.file.data.len() - 0x8);
    }

    pub fn adjust_cur_pos(&mut self, delta: isize) {
        self.cur_pos =
            (self.cur_pos as isize + delta).clamp(0, self.file.data.len() as isize - 0x8) as usize;
    }

    pub fn bytes_per_screen(&self) -> usize {
        self.bytes_per_row * self.num_rows as usize
    }

    pub fn get_cur_bytes(&self) -> Vec<u8> {
        let max_end = self.cur_pos + self.bytes_per_screen();
        let end = max_end.min(self.file.data.len());

        self.file.data[self.cur_pos..end].to_vec()
    }

    pub fn get_selected_bytes(&self) -> Vec<u8> {
        match self.selection.state {
            HexViewSelectionState::None => vec![],
            HexViewSelectionState::Selecting | HexViewSelectionState::Selected => {
                self.file.data[self.selection.start()..self.selection.end() + 1].to_vec()
            }
        }
    }

    pub fn reload_file(&mut self) -> Result<(), Error> {
        self.file.data = read_file_bytes(self.file.path.clone())?;

        if self.selection.first >= self.file.data.len()
            && self.selection.second >= self.file.data.len()
        {
            self.selection.clear();
        } else {
            self.selection.first = self.selection.first.min(self.file.data.len() - 1);
            self.selection.second = self.selection.second.min(self.file.data.len() - 1);
        }
        Ok(())
    }

    fn show_hex_grid(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        cursor_state: CursorState,
        font_size: f32,
    ) {
        ui.group(|ui| {
            egui::Grid::new(format!("hex_grid{}", self.id))
                .striped(true)
                .spacing([0.0, 0.0])
                .min_col_width(0.0)
                .num_columns(40)
                .show(ui, |ui| {
                    let screen_bytes = self.get_cur_bytes();
                    let mut current_pos = self.cur_pos;

                    let mut row_chunks = screen_bytes.chunks(self.bytes_per_row);

                    let mut r = 0;
                    while r < self.num_rows {
                        let row = row_chunks.next().unwrap_or_default();

                        let num_digits = match self.file.data.len() {
                            0..=0xFFFF => 4,
                            0x1000000..=0xFFFFFFFF => 8,
                            0x10000000000..=0xFFFFFFFFFFFF => 12,
                            _ => 8,
                        };
                        let mut i = num_digits;
                        let mut offset_leading_zeros = true;

                        while i > 0 {
                            let digit = current_pos >> ((i - 1) * 4) & 0xF;

                            if offset_leading_zeros && digit > 0 {
                                offset_leading_zeros = false;
                            }

                            let offset_digit = egui::Label::new(
                                egui::RichText::new(format!("{:X}", digit))
                                    .monospace()
                                    .size(font_size)
                                    .color({
                                        if offset_leading_zeros {
                                            Color32::DARK_GRAY
                                        } else {
                                            Color32::GRAY
                                        }
                                    }),
                            );

                            if i < num_digits && (i % 4) == 0 {
                                ui.add(Spacer::default().spacing_x(4.0));
                            }
                            ui.add(offset_digit);
                            i -= 1;
                        }

                        ui.add(Spacer::default().spacing_x(8.0));
                        ui.add(Separator::default().vertical().spacing(0.0));
                        ui.add(Spacer::default().spacing_x(8.0));

                        // hex view
                        let mut i = 0;
                        while i < self.bytes_per_row {
                            if i > 0 && (i % 8) == 0 {
                                ui.add(Spacer::default().spacing_x(4.0));
                            }
                            let row_current_pos = current_pos + i;

                            let byte: Option<u8> = row.get(i).copied();

                            let byte_text = match byte {
                                Some(byte) => format!("{:02X}", byte),
                                None => "  ".to_string(),
                            };

                            let hex_label = egui::Label::new(
                                egui::RichText::new(byte_text)
                                    .monospace()
                                    .size(font_size)
                                    .color(match byte {
                                        Some(0) => Color32::DARK_GRAY,
                                        _ => Color32::LIGHT_GRAY,
                                    })
                                    .background_color({
                                        if self.selection.contains(row_current_pos) {
                                            Color32::DARK_GREEN
                                        } else {
                                            Color32::TRANSPARENT
                                        }
                                    }),
                            )
                            .sense(Sense::click_and_drag());

                            let res = ui.add(hex_label);

                            if byte.is_some() {
                                if let Some(cursor_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                                    if res.rect.contains(cursor_pos) {
                                        match cursor_state {
                                            CursorState::Pressed => {
                                                self.selection.begin(row_current_pos);
                                            }
                                            CursorState::StillDown => match self.selection.state {
                                                HexViewSelectionState::None => {
                                                    self.selection.begin(row_current_pos);
                                                }
                                                HexViewSelectionState::Selecting => {
                                                    self.selection.update(row_current_pos);
                                                }
                                                _ => (),
                                            },
                                            CursorState::Released => {
                                                if self.selection.state
                                                    == HexViewSelectionState::Selecting
                                                {
                                                    self.selection.finalize(row_current_pos);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }

                                if res.middle_clicked() {
                                    self.selection.clear();
                                }
                            }

                            if i < self.bytes_per_row - 1 {
                                ui.add(Spacer::default().spacing_x(4.0));
                            }
                            i += 1;
                        }

                        ui.add(Spacer::default().spacing_x(8.0));
                        ui.add(Separator::default().vertical().spacing(0.0));
                        ui.add(Spacer::default().spacing_x(8.0));

                        // ascii view
                        let mut i = 0;
                        while i < self.bytes_per_row {
                            let byte: Option<u8> = row.get(i).copied();

                            let row_current_pos = current_pos + i;

                            let ascii_char = match byte {
                                Some(32..=126) => byte.unwrap() as char,
                                Some(_) => '·',
                                None => ' ',
                            };

                            let hex_label = egui::Label::new(
                                egui::RichText::new(ascii_char)
                                    .monospace()
                                    .size(font_size)
                                    .color(match byte {
                                        Some(0) => Color32::DARK_GRAY,
                                        Some(32..=126) => Color32::LIGHT_GRAY,
                                        _ => Color32::GRAY,
                                    })
                                    .background_color({
                                        if self.selection.contains(row_current_pos) {
                                            Color32::DARK_GREEN
                                        } else {
                                            Color32::TRANSPARENT
                                        }
                                    }),
                            )
                            .sense(Sense::click_and_drag());

                            let res = ui.add(hex_label);
                            ui.add(Spacer::default().spacing_x(1.0));

                            if byte.is_some() {
                                if let Some(cursor_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                                    if res.rect.contains(cursor_pos) {
                                        match cursor_state {
                                            CursorState::Pressed => {
                                                self.selection.first = row_current_pos;
                                                self.selection.second = row_current_pos;
                                                self.selection.state =
                                                    HexViewSelectionState::Selecting;
                                            }
                                            CursorState::StillDown => {
                                                if self.selection.state
                                                    == HexViewSelectionState::Selecting
                                                {
                                                    self.selection.second = row_current_pos;
                                                }
                                            }
                                            CursorState::Released => {
                                                if self.selection.state
                                                    == HexViewSelectionState::Selecting
                                                {
                                                    self.selection.second = row_current_pos;
                                                    self.selection.state =
                                                        HexViewSelectionState::Selected;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            i += 1;
                        }

                        current_pos += self.bytes_per_row;
                        r += 1;
                        ui.end_row();
                    }
                });
        });
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, cursor_state: CursorState) {
        let font_size = 14.0;

        ui.group(|ui| {
            let file_name = self.file.path.as_path().to_str().unwrap();

            ui.label(
                egui::RichText::new(file_name)
                    .monospace()
                    .size(font_size)
                    .color(Color32::LIGHT_GRAY),
            );

            ui.with_layout(
                egui::Layout::left_to_right(eframe::emath::Align::Min),
                |ui| {
                    if ui
                        .small_button("strings")
                        .on_hover_text(match self.sv.show {
                            true => "Hide String Viewer",
                            false => "Show String Viewer",
                        })
                        .clicked()
                    {
                        self.sv.show = !self.sv.show;
                    };

                    if ui
                        .small_button("data")
                        .on_hover_text(match self.dv.show {
                            true => "Hide Data Viewer",
                            false => "Show Data Viewer",
                        })
                        .clicked()
                    {
                        self.dv.show = !self.dv.show;
                    };
                },
            );

            ui.with_layout(
                egui::Layout::left_to_right(eframe::emath::Align::Min),
                |ui: &mut egui::Ui| {
                    ui.vertical(|ui| {
                        self.show_hex_grid(ctx, ui, cursor_state, font_size);

                        self.sv.display(ui, self.id, self.get_selected_bytes());

                        let selection_text = match self.selection.state {
                            HexViewSelectionState::None => "No selection".to_owned(),
                            _ => {
                                let start = self.selection.start();
                                let end = self.selection.end();
                                let length = end - start + 1;

                                format!(
                                    "Selection: 0x{:X} - 0x{:X} (len 0x{:X})",
                                    start, end, length
                                )
                            }
                        };
                        ui.label(egui::RichText::new(selection_text).monospace());
                    });

                    self.dv.display(ui, self.id, self.get_selected_bytes());
                },
            );
        });
    }
}
