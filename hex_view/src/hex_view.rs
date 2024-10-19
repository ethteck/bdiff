use crate::{spacer::Spacer, theme::HexViewStyle, Color};

use crate::byte_grouping::ByteGrouping;
use crate::cursor_state::CursorState;
use crate::selection::{HexViewSelection, HexViewSelectionSide, HexViewSelectionState};
use egui::{self, Color32, FontId, Sense, Separator};

#[derive(Clone, Default, PartialEq)]
pub struct HexView {
    pub id: usize,
    pub style: HexViewStyle,
    pub bytes_per_row: usize,
    pub num_rows: usize,
    pub selection: HexViewSelection,
    pub cursor_pos: Option<usize>,
}

pub struct HexViewOptions {
    pub can_selection_change: bool,
    pub byte_grouping: ByteGrouping,
    pub num_offset_digits: usize,
}

pub struct HexViewState<'state> {
    pub file_data: &'state [u8],
    pub file_pos: usize,
    pub global_pos: usize,
    pub diffs: Option<&'state [bool]>,
}

impl HexView {
    pub fn new(id: usize, bytes_per_row: usize, num_rows: usize) -> Self {
        Self {
            id,
            style: HexViewStyle::default(),
            bytes_per_row,
            num_rows,
            selection: HexViewSelection::default(),
            cursor_pos: None,
        }
    }

    pub fn set_style(&mut self, style: HexViewStyle) {
        self.style = style;
    }

    pub fn get_selected_bytes<'data>(&self, data: &'data [u8], file_pos: usize) -> &'data [u8] {
        match self.selection.state {
            HexViewSelectionState::None => &[],
            HexViewSelectionState::Selecting | HexViewSelectionState::Selected => {
                let start = (self.selection.start() as isize - file_pos as isize).max(0) as usize;
                let end = (self.selection.end() as isize - file_pos as isize).max(0) as usize;
                if start < data.len() {
                    &data[start..(end + 1).min(data.len())]
                } else {
                    &[]
                }
            }
        }
    }

    fn show_offset(&mut self, num_digits: usize, current_pos: isize, ui: &mut egui::Ui) {
        let num_digits: i32 = num_digits as i32;

        let mut i: i32 = num_digits;
        let mut offset_leading_zeros = true;

        while i > 0 {
            let digit = current_pos >> ((i - 1) * 4) & 0xF;

            if offset_leading_zeros && digit > 0 {
                offset_leading_zeros = false;
            }

            let digit_text = if current_pos < 0 {
                " ".to_string()
            } else {
                format!("{:X}", digit)
            };

            let offset_digit = egui::Label::new(
                egui::RichText::new(digit_text)
                    .font(FontId::monospace(self.style.font_size))
                    .color({
                        if offset_leading_zeros {
                            self.style.offset_leading_zero_color.clone()
                        } else {
                            self.style.offset_text_color.clone()
                        }
                    }),
            );

            if i < num_digits && (i % 4) == 0 {
                ui.add(Spacer::default().spacing_x(4.0));
            }
            ui.add(offset_digit);
            i -= 1;
        }
    }

    fn get_selection_color(&self, pos: usize) -> Color {
        if self.selection.contains(pos) {
            self.style.selection_color.clone()
        } else {
            Color32::TRANSPARENT.into()
        }
    }

    fn show_hex(
        &mut self,
        ui: &mut egui::Ui,
        row: usize,
        row_data: &[Option<u8>],
        state: &HexViewState,
        cursor_state: CursorState,
        options: &HexViewOptions,
    ) {
        let mut i = 0;
        let mut cur_pos = state.file_pos + row * self.bytes_per_row;
        let mut global_pos = state.global_pos + row * self.bytes_per_row;

        while i < self.bytes_per_row {
            let byte_grouping: usize = options.byte_grouping.into();

            if i > 0 && (i % byte_grouping) == 0 {
                ui.add(Spacer::default().spacing_x(4.0));
            }

            let byte: Option<u8> = row_data[i];

            let byte_text = match byte {
                Some(byte) => format!("{:02X}", byte),
                None => "  ".to_string(),
            };

            let hex_label = egui::Label::new(
                egui::RichText::new(byte_text)
                    .font(FontId::monospace(self.style.font_size))
                    .color(
                        if state
                            .diffs
                            .is_some_and(|diffs| global_pos < diffs.len() && diffs[global_pos])
                        {
                            self.style.diff_color.clone()
                        } else {
                            match byte {
                                Some(0) => self.style.hex_null_color.clone(),
                                _ => self.style.other_hex_color.clone(),
                            }
                        },
                    )
                    .background_color(self.get_selection_color(global_pos)),
            )
            .sense(Sense::click_and_drag());

            let res = ui.add(hex_label);

            if byte.is_some() {
                if res.contains_pointer() {
                    self.cursor_pos = Some(cur_pos);
                }
                if options.can_selection_change {
                    self.handle_selection(
                        ui,
                        res,
                        cursor_state,
                        global_pos,
                        HexViewSelectionSide::Hex,
                    );
                }
            }
            i += 1;
            cur_pos += 1;
            global_pos += 1;

            if i < self.bytes_per_row {
                ui.add(Spacer::default().spacing_x(4.0));
            }
        }
    }

    fn show_ascii(
        &mut self,
        ui: &mut egui::Ui,
        row: usize,
        row_data: &[Option<u8>],
        state: &HexViewState,
        cursor_state: CursorState,
        options: &HexViewOptions,
    ) {
        let mut i = 0;
        let mut cur_pos = state.file_pos + row * self.bytes_per_row;
        let mut global_pos = state.global_pos + row * self.bytes_per_row;

        while i < self.bytes_per_row {
            let byte: Option<u8> = row_data[i];

            let ascii_char = match byte {
                Some(32..=126) => byte.unwrap() as char,
                Some(_) => '·',
                None => ' ',
            };

            let hex_label = egui::Label::new(
                egui::RichText::new(ascii_char)
                    .font(FontId::monospace(self.style.font_size))
                    .color(match byte {
                        Some(0) => self.style.ascii_null_color.clone(),
                        Some(32..=126) => self.style.ascii_color.clone(),
                        _ => self.style.other_ascii_color.clone(),
                    })
                    .background_color(self.get_selection_color(global_pos)),
            )
            .sense(Sense::click_and_drag());

            let res = ui.add(hex_label);
            ui.add(Spacer::default().spacing_x(1.0));

            if byte.is_some() {
                if res.contains_pointer() {
                    self.cursor_pos = Some(cur_pos);
                }
                if options.can_selection_change {
                    self.handle_selection(
                        ui,
                        res,
                        cursor_state,
                        global_pos,
                        HexViewSelectionSide::Ascii,
                    );
                }
            }
            i += 1;
            cur_pos += 1;
            global_pos += 1;
        }
    }

    fn get_display_bytes(
        &self,
        data: &[u8],
        file_offset: usize,
        global_offset: usize,
    ) -> Vec<Option<u8>> {
        let num_bytes = self.bytes_per_row * self.num_rows;
        let pos: isize = global_offset as isize - file_offset as isize;

        if pos > 0 && (pos as usize) > data.len() {
            vec![None; num_bytes]
        } else {
            let mut bytes = Vec::with_capacity(num_bytes);
            for i in 0..num_bytes {
                let idx = pos + i as isize;
                if idx >= 0 && (idx as usize) < data.len() {
                    bytes.push(Some(data[idx as usize]));
                } else {
                    bytes.push(None);
                }
            }
            bytes
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &HexViewState,
        cursor_state: CursorState,
        options: HexViewOptions,
    ) {
        let data = self.get_display_bytes(state.file_data, state.file_pos, state.global_pos);

        let grid_rect = egui::Grid::new(format!("hex_grid{}", self.id))
            .striped(true)
            .spacing([0.0, 0.0])
            .min_col_width(0.0)
            .num_columns(40)
            .show(ui, |ui| {
                let mut current_pos = state.global_pos as isize - state.file_pos as isize;
                let mut row_chunks = data.chunks(self.bytes_per_row);

                let mut r = 0;
                while r < self.num_rows {
                    let row_data = row_chunks.next().unwrap_or_default();

                    self.show_offset(options.num_offset_digits, current_pos, ui);

                    ui.add(Spacer::default().spacing_x(8.0));
                    ui.add(Separator::default().vertical().spacing(0.0));
                    ui.add(Spacer::default().spacing_x(8.0));

                    self.show_hex(ui, r, row_data, state, cursor_state, &options);

                    ui.add(Spacer::default().spacing_x(8.0));
                    ui.add(Separator::default().vertical().spacing(0.0));
                    ui.add(Spacer::default().spacing_x(8.0));

                    self.show_ascii(ui, r, row_data, state, cursor_state, &options);

                    current_pos += self.bytes_per_row as isize;
                    r += 1;
                    ui.end_row();
                }
            })
            .response
            .rect;

        if let Some(cursor_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if !grid_rect.contains(cursor_pos) {
                self.cursor_pos = None;
            }
        }
    }

    pub fn handle_selection(
        &mut self,
        ui: &mut egui::Ui,
        res: egui::Response,
        cursor_state: CursorState,
        pos: usize,
        side: HexViewSelectionSide,
    ) {
        if res.hovered() {
            if cursor_state == CursorState::Pressed {
                self.selection.begin(pos, side);
            }

            self.cursor_pos = Some(pos);
        }

        if let Some(cursor_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if res.rect.contains(cursor_pos) {
                match cursor_state {
                    CursorState::StillDown => {
                        if self.selection.state == HexViewSelectionState::Selecting {
                            self.selection.update(pos);
                        }
                    }
                    CursorState::Released => {
                        if self.selection.state == HexViewSelectionState::Selecting {
                            self.selection.finalize(pos);
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
}
