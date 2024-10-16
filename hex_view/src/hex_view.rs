use crate::{spacer::Spacer, theme::HexViewStyle};

use egui::{self, Color32, FontId, Sense, Separator};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorState {
    Hovering,
    Pressed,
    StillDown,
    Released,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HexViewSelectionRange {
    pub first: usize,
    pub second: usize,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum HexViewSelectionState {
    #[default]
    None,
    Selecting,
    Selected,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum HexViewSelectionSide {
    #[default]
    Hex,
    Ascii,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HexViewSelection {
    pub range: HexViewSelectionRange,
    pub state: HexViewSelectionState,
    pub side: HexViewSelectionSide,
}

impl HexViewSelection {
    pub fn start(&self) -> usize {
        self.range.first.min(self.range.second)
    }

    pub fn end(&self) -> usize {
        self.range.second.max(self.range.first)
    }

    fn contains(&self, grid_pos: usize) -> bool {
        self.state != HexViewSelectionState::None
            && grid_pos >= self.start()
            && grid_pos <= self.end()
    }

    pub fn begin(&mut self, grid_pos: usize, side: HexViewSelectionSide) {
        self.range.first = grid_pos;
        self.range.second = grid_pos;
        self.state = HexViewSelectionState::Selecting;
        self.side = side;
    }

    pub fn update(&mut self, grid_pos: usize) {
        self.range.second = grid_pos;
    }

    pub fn finalize(&mut self, grid_pos: usize) {
        self.range.second = grid_pos;
        self.state = HexViewSelectionState::Selected;
    }

    pub fn clear(&mut self) {
        self.range.first = 0;
        self.range.second = 0;
        self.state = HexViewSelectionState::None;
        self.side = HexViewSelectionSide::default();
    }

    pub fn adjust_cur_pos(&mut self, delta: isize) {
        self.range.first = (self.range.first as isize + delta).max(0) as usize;
        self.range.second = (self.range.second as isize + delta).max(0) as usize;
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct HexView {
    pub id: usize,
    pub style: HexViewStyle,
    pub bytes_per_row: usize,
    pub num_rows: usize,
    pub selection: HexViewSelection,
    pub cursor_pos: Option<usize>,
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

    pub fn bytes_per_screen(&self) -> usize {
        self.bytes_per_row * self.num_rows
    }

    pub fn get_cur_bytes(&self, data: &[u8], pos: isize) -> Vec<Option<u8>> {
        let bytes_per_screen = self.bytes_per_row * self.num_rows;

        if pos > 0 && (pos as usize) > data.len() {
            vec![None; bytes_per_screen]
        } else {
            let mut bytes = Vec::with_capacity(bytes_per_screen);
            for i in 0..bytes_per_screen {
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

    pub fn get_selected_bytes<'data>(&self, data: &'data [u8]) -> &'data [u8] {
        match self.selection.state {
            HexViewSelectionState::None => &[],
            HexViewSelectionState::Selecting | HexViewSelectionState::Selected => {
                let start = self.selection.start();
                let end = self.selection.end();
                if start < data.len() {
                    &data[start..(end + 1).min(data.len())]
                } else {
                    &[]
                }
            }
        }
    }

    fn show_offset(&mut self, data: &[u8], current_pos: isize, ui: &mut egui::Ui) {
        let num_digits = match data.len() {
            //0..=0xFFFF => 4,
            0x10000..=0xFFFFFFFF => 8,
            0x100000000..=0xFFFFFFFFFFFF => 12,
            _ => 8,
        };
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

    fn show_hex(
        &mut self,
        byte_grouping: usize,
        ui: &mut egui::Ui,
        start_pos: isize,
        row: &[Option<u8>],
        diffs: Option<&[bool]>,
        can_selection_change: bool,
        cursor_state: CursorState,
    ) {
        let mut i = 0;
        while i < self.bytes_per_row {
            if i > 0 && (i % byte_grouping) == 0 {
                ui.add(Spacer::default().spacing_x(4.0));
            }
            let pos = start_pos + i as isize;

            let byte: Option<u8> = row[i];

            let byte_text = match byte {
                Some(byte) => format!("{:02X}", byte),
                None => "  ".to_string(),
            };

            let hex_label = egui::Label::new(
                egui::RichText::new(byte_text)
                    .font(FontId::monospace(self.style.font_size))
                    .color(
                        if pos >= 0
                            && diffs.is_some_and(|diffs| {
                            (pos as usize) < diffs.len() && diffs[pos as usize]
                        })
                        {
                            self.style.diff_color.clone()
                        } else {
                            match byte {
                                Some(0) => self.style.hex_null_color.clone(),
                                _ => self.style.other_hex_color.clone(),
                            }
                        },
                    )
                    .background_color({
                        if pos >= 0 && self.selection.contains(pos as usize) {
                            self.style.selection_color.clone()
                        } else {
                            Color32::TRANSPARENT.into()
                        }
                    }),
            )
                .sense(Sense::click_and_drag());

            let res = ui.add(hex_label);

            if byte.is_some() {
                if res.contains_pointer() {
                    self.cursor_pos = Some(pos as usize);
                }
                if can_selection_change {
                    self.handle_selection(
                        ui,
                        res,
                        cursor_state,
                        pos as usize,
                        HexViewSelectionSide::Hex,
                    );
                }
            }
            i += 1;

            if i < self.bytes_per_row {
                ui.add(Spacer::default().spacing_x(4.0));
            }
        }
    }

    fn show_ascii(
        &mut self,
        row: &[Option<u8>],
        current_pos: isize,
        ui: &mut egui::Ui,
        can_selection_change: bool,
        cursor_state: CursorState,
    ) {
        let mut i = 0;
        while i < self.bytes_per_row {
            let byte: Option<u8> = row[i];

            let row_current_pos = current_pos + i as isize;

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
                    .background_color({
                        if row_current_pos >= 0 && self.selection.contains(row_current_pos as usize) {
                            self.style.selection_color.clone()
                        } else {
                            Color32::TRANSPARENT.into()
                        }
                    }),
            )
                .sense(Sense::click_and_drag());

            let res = ui.add(hex_label);
            ui.add(Spacer::default().spacing_x(1.0));

            if byte.is_some() {
                if res.contains_pointer() {
                    self.cursor_pos = Some(row_current_pos as usize);
                }
                if can_selection_change {
                    self.handle_selection(
                        ui,
                        res,
                        cursor_state,
                        row_current_pos as usize,
                        HexViewSelectionSide::Ascii,
                    );
                }
            }
            i += 1;
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        data: &[u8],
        offset: isize,
        cursor_state: CursorState,
        can_selection_change: bool,
        byte_grouping: usize,
        diffs: Option<&[bool]>,
    ) {
        let grid_rect = egui::Grid::new(format!("hex_grid{}", self.id))
            .striped(true)
            .spacing([0.0, 0.0])
            .min_col_width(0.0)
            .num_columns(40)
            .show(ui, |ui| {
                let screen_bytes = self.get_cur_bytes(data, offset);
                let mut current_pos = offset;

                let mut row_chunks = screen_bytes.chunks(self.bytes_per_row);

                let mut r = 0;
                while r < self.num_rows {
                    let row = row_chunks.next().unwrap_or_default();

                    self.show_offset(data, current_pos, ui);

                    ui.add(Spacer::default().spacing_x(8.0));
                    ui.add(Separator::default().vertical().spacing(0.0));
                    ui.add(Spacer::default().spacing_x(8.0));

                    self.show_hex(
                        byte_grouping,
                        ui,
                        current_pos,
                        row,
                        diffs,
                        can_selection_change,
                        cursor_state,
                    );

                    ui.add(Spacer::default().spacing_x(8.0));
                    ui.add(Separator::default().vertical().spacing(0.0));
                    ui.add(Spacer::default().spacing_x(8.0));

                    self.show_ascii(row, current_pos, ui, can_selection_change, cursor_state);

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
        row_current_pos: usize,
        side: HexViewSelectionSide,
    ) {
        if res.hovered() {
            if cursor_state == CursorState::Pressed && row_current_pos > 0 {
                self.selection.begin(row_current_pos, side);
            }

            self.cursor_pos = Some(row_current_pos);
        }

        if let Some(cursor_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if res.rect.contains(cursor_pos) && row_current_pos > 0 {
                match cursor_state {
                    CursorState::StillDown => {
                        if self.selection.state == HexViewSelectionState::Selecting {
                            self.selection.update(row_current_pos);
                        }
                    }
                    CursorState::Released => {
                        if self.selection.state == HexViewSelectionState::Selecting {
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
}
