use std::{borrow::BorrowMut, ops::Deref};

use eframe::{
    egui::{self, Sense, Separator},
    epaint::Color32,
};
use encoding_rs::*;

use crate::app::CursorState;
use crate::spacer::Spacer;
use crate::BinFile;

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
    pub file: BinFile,
    pub num_rows: u32,
    pub bytes_per_row: usize,
    pub cur_pos: usize,
    pub selection: HexViewSelection,
}

impl HexView {
    pub fn new(file: BinFile) -> Self {
        let min_rows = 10;
        let max_rows = 25;
        let default_bytes_per_row = 0x10;
        let num_rows = (file.data.len() / default_bytes_per_row).clamp(min_rows, max_rows) as u32;

        Self {
            file,
            num_rows,
            bytes_per_row: default_bytes_per_row,
            ..Default::default()
        }
    }

    pub fn set_cur_pos(&mut self, val: usize) {
        self.cur_pos = val.clamp(0, self.file.data.len() - 0x8);
    }

    pub fn adjust_cur_pos(&mut self, delta: i32) {
        self.cur_pos =
            (self.cur_pos as i32 + delta).clamp(0, self.file.data.len() as i32 - 0x8) as usize;
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

    fn render_hex_grid(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        cursor_state: CursorState,
        font_size: f32,
    ) {
        ui.group(|ui| {
            egui::Grid::new("hex_grid")
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

                        // offset
                        let num_digits = 8; // 8 of those boys
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
        let file_name = self.file.path.as_path().to_str().unwrap();
        ui.label(
            egui::RichText::new(file_name)
                .monospace()
                .size(font_size)
                .color(Color32::LIGHT_GRAY),
        );
        ui.with_layout(
            egui::Layout::left_to_right(eframe::emath::Align::Min),
            |ui: &mut egui::Ui| {
                ui.vertical(|ui| {
                    self.render_hex_grid(ctx, ui, cursor_state, font_size);
                    ui.group(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("String Viewer").monospace(),
                        ));

                        egui::Grid::new("string_grid")
                            .striped(true)
                            .num_columns(2)
                            .show(ui, |ui| {
                                let selected_bytes = self.get_selected_bytes();

                                ui.add(egui::Label::new(egui::RichText::new("UTF-8").monospace()));
                                ui.text_edit_singleline(
                                    &mut String::from_utf8(selected_bytes.clone())
                                        .unwrap_or_default(),
                                );
                                ui.end_row();

                                ui.add(egui::Label::new(egui::RichText::new("UTF-16").monospace()));
                                ui.text_edit_singleline(
                                    &mut UTF_16BE
                                        .decode_without_bom_handling_and_without_replacement(
                                            &selected_bytes,
                                        )
                                        .unwrap_or_default()
                                        .to_string(),
                                );
                                ui.end_row();

                                ui.add(egui::Label::new(egui::RichText::new("EUC-JP").monospace()));
                                ui.text_edit_singleline(
                                    &mut EUC_JP
                                        .decode_without_bom_handling_and_without_replacement(
                                            &selected_bytes,
                                        )
                                        .unwrap_or_default()
                                        .to_string(),
                                );
                                ui.end_row();

                                ui.add(egui::Label::new(
                                    egui::RichText::new("Shift JIS").monospace(),
                                ));
                                ui.text_edit_singleline(
                                    &mut SHIFT_JIS
                                        .decode_without_bom_handling_and_without_replacement(
                                            &selected_bytes,
                                        )
                                        .unwrap_or_default()
                                        .to_string(),
                                );
                                ui.end_row();
                            });
                    });
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("Selection Info").monospace(),
                        ));

                        let start: String = match self.selection.state {
                            HexViewSelectionState::None => "".to_owned(),
                            _ => format!("0x{:X}", self.selection.start()),
                        };
                        let end: String = match self.selection.state {
                            HexViewSelectionState::None => "".to_owned(),
                            _ => format!("0x{:X}", self.selection.end()),
                        };
                        let length: String = match self.selection.state {
                            HexViewSelectionState::None => "".to_owned(),
                            _ => {
                                format!("0x{:X}", self.selection.end() - self.selection.start() + 1)
                            }
                        };

                        egui::Grid::new("hex_grid_selection")
                            .striped(true)
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.add(egui::Label::new(egui::RichText::new("start").monospace()));
                                ui.add(egui::Label::new(egui::RichText::new(start).monospace()));
                                ui.end_row();

                                ui.add(egui::Label::new(egui::RichText::new("end").monospace()));
                                ui.add(egui::Label::new(egui::RichText::new(end).monospace()));
                                ui.end_row();

                                ui.add(egui::Label::new(egui::RichText::new("length").monospace()));
                                ui.add(egui::Label::new(egui::RichText::new(length).monospace()));
                                ui.end_row();

                                let mut buffer = dtoa::Buffer::new();

                                let selected_bytes = self.get_selected_bytes();
                                let selection_len = selected_bytes.len();

                                if selection_len >= 1 {
                                    let bytes = selected_bytes[0..1].try_into().unwrap_or_default();

                                    self.selection_data_row(
                                        ui,
                                        "s8",
                                        format!("{:}", i8::from_be_bytes(bytes)),
                                    );

                                    self.selection_data_row(
                                        ui,
                                        "u8",
                                        format!("{:}", u8::from_be_bytes(bytes)),
                                    );
                                }

                                if selection_len >= 2 {
                                    let bytes = selected_bytes[0..2].try_into().unwrap_or_default();

                                    self.selection_data_row(
                                        ui,
                                        "s16",
                                        format!("{:}", i16::from_be_bytes(bytes)),
                                    );

                                    self.selection_data_row(
                                        ui,
                                        "u16",
                                        format!("{:}", u16::from_be_bytes(bytes)),
                                    );
                                }

                                if selection_len >= 4 {
                                    let bytes = selected_bytes[0..4].try_into().unwrap_or_default();

                                    self.selection_data_row(
                                        ui,
                                        "s32",
                                        format!("{:}", i32::from_be_bytes(bytes)),
                                    );

                                    self.selection_data_row(
                                        ui,
                                        "u32",
                                        format!("{:}", u32::from_be_bytes(bytes)),
                                    );

                                    self.selection_data_row(
                                        ui,
                                        "f32",
                                        buffer.format(f32::from_be_bytes(bytes)),
                                    );
                                }

                                if selection_len >= 8 {
                                    let bytes = selected_bytes[0..8].try_into().unwrap_or_default();

                                    self.selection_data_row(
                                        ui,
                                        "s64",
                                        format!("{:}", i64::from_be_bytes(bytes)),
                                    );

                                    self.selection_data_row(
                                        ui,
                                        "u64",
                                        format!("{:}", u64::from_be_bytes(bytes)),
                                    );

                                    self.selection_data_row(
                                        ui,
                                        "f64",
                                        buffer.format(f64::from_be_bytes(bytes)),
                                    );
                                }
                            });
                    });
                });
            },
        );
    }

    fn selection_data_row(
        &self,
        ui: &mut egui::Ui,
        name: impl Into<String>,
        data: impl Into<String>,
    ) {
        ui.add(egui::Label::new(egui::RichText::new(name).monospace()));
        ui.add(egui::Label::new(egui::RichText::new(data).monospace()));
        ui.end_row();
    }
}
