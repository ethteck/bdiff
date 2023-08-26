use std::thread::current;

use eframe::{
    egui::{self, Sense},
    epaint::Color32,
};

use crate::app::CursorState;
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

    pub fn clear(&mut self) {
        self.first = 0;
        self.second = 0;
        self.state = HexViewSelectionState::None;
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

    pub fn get_selected_bytes(&self) -> Option<Vec<u8>> {
        match self.selection.state {
            HexViewSelectionState::None => None,
            HexViewSelectionState::Selecting | HexViewSelectionState::Selected => {
                Some(self.file.data[self.selection.start()..self.selection.end() + 1].to_vec())
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, cursor_state: CursorState) {
        ui.label(
            egui::RichText::new(self.file.path.as_path().to_str().unwrap())
                .monospace()
                .size(18.0)
                .color(Color32::LIGHT_GRAY),
        );
        ui.separator();

        // Render each byte as a label in a grid of 16 columns
        let num_bytes_per_row = 0x10;

        egui::Grid::new("hex_grid")
            .striped(true)
            .spacing([0.0, 0.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                let screen_bytes = self.get_cur_bytes();
                let mut current_pos = self.cur_pos;

                let mut row_chunks = screen_bytes.chunks(num_bytes_per_row);

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
                                .size(18.0)
                                .color({
                                    if offset_leading_zeros {
                                        Color32::DARK_GRAY
                                    } else {
                                        Color32::LIGHT_GRAY
                                    }
                                }),
                        );

                        if i < num_digits && (i % 4) == 0 {
                            ui.add_space(4.0)
                        }
                        ui.add(offset_digit);
                        i -= 1;
                    }

                    ui.add_space(10.0);

                    // hex view
                    let mut i = 0;
                    while i < self.bytes_per_row {
                        if i > 0 && (i % 8) == 0 {
                            ui.add_space(4.0)
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
                                .size(18.0)
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
                            ui.add_space(4.0);
                        }
                        i += 1;
                    }

                    ui.add_space(10.0);

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
                                .size(18.0)
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
                        ui.add_space(1.0);

                        if byte.is_some() {
                            if let Some(cursor_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                                if res.rect.contains(cursor_pos) {
                                    match cursor_state {
                                        CursorState::Pressed => {
                                            self.selection.first = row_current_pos;
                                            self.selection.second = row_current_pos;
                                            self.selection.state = HexViewSelectionState::Selecting;
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

                    current_pos += num_bytes_per_row;
                    r += 1;
                    ui.end_row();
                }
            });
        ui.separator();

        // Render the selection as u8, s8, u16, s16, u32, s32, u64, s64, f32, f64
        if let Some(bytes) = self.get_selected_bytes() {
            let mut i = 0;
            while i < bytes.len() {
                let byte = bytes[i];

                let byte_text = format!("{:02X}", byte);

                let byte_label = egui::Label::new(
                    egui::RichText::new(byte_text)
                        .monospace()
                        .size(18.0)
                        .color(Color32::LIGHT_GRAY),
                );

                ui.add(byte_label);

                if i < bytes.len() - 1 {
                    ui.add_space(4.0);
                }
                i += 1;
            }
        }
    }
}
