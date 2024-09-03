use crate::spacer::Spacer;

use egui::{self, Color32, FontId, Sense, Separator};
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct HexViewStyle {
    pub selection_color: Color32,

    // Offset colors
    pub offset_text_color: Color32,
    pub offset_leading_zero_color: Color32,

    // Hex View colors
    pub diff_color: Color32,
    pub hex_null_color: Color32,
    pub other_hex_color: Color32,

    // ASCII View colors
    pub ascii_null_color: Color32,
    pub ascii_color: Color32,
    pub other_ascii_color: Color32,
}

impl Default for HexViewStyle {
    fn default() -> Self {
        Self {
            offset_text_color: Color32::GRAY,
            offset_leading_zero_color: Color32::DARK_GRAY,

            selection_color: Color32::DARK_GREEN,
            diff_color: Color32::RED,
            hex_null_color: Color32::DARK_GRAY,
            other_hex_color: Color32::GRAY,

            ascii_null_color: Color32::DARK_GRAY,
            ascii_color: Color32::LIGHT_GRAY,
            other_ascii_color: Color32::GRAY,
        }
    }
}

pub struct HexView {
    pub id: usize,
    pub style: HexViewStyle,
    pub bytes_per_row: usize,
    pub cur_pos: usize,
    pub pos_locked: bool,
    pub selection: HexViewSelection,
    pub cursor_pos: Option<usize>,
}

impl HexView {
    pub fn new(id: usize) -> Self {
        let default_bytes_per_row = 0x10;

        Self {
            id,
            style: HexViewStyle::default(),
            bytes_per_row: default_bytes_per_row,
            cur_pos: 0,
            pos_locked: false,
            selection: HexViewSelection::default(),
            cursor_pos: None,
        }
    }

    /// Change the [`HexViewStyle`] of the HexView upon creation.
    pub fn with_style(mut self, style: &HexViewStyle) -> Self {
        self.style = style.clone();
        self
    }

    pub fn set_cur_pos(&mut self, data: &[u8], val: usize) {
        if self.pos_locked {
            return;
        }
        let last_line_start_address = (data.len() / self.bytes_per_row) * self.bytes_per_row;
        self.cur_pos = val.clamp(0, last_line_start_address);
    }

    pub fn adjust_cur_pos(&mut self, data: &[u8], delta: isize) {
        if self.pos_locked {
            return;
        }
        let last_line_start_address = (data.len() / self.bytes_per_row) * self.bytes_per_row;
        self.cur_pos =
            (self.cur_pos as isize + delta).clamp(0, last_line_start_address as isize) as usize;
    }

    pub fn num_rows(&self, data: &[u8]) -> usize {
        let min_rows = 10;
        let max_rows = 25;
        (data.len() / self.bytes_per_row).clamp(min_rows, max_rows)
    }

    pub fn bytes_per_screen(&self, data: &[u8]) -> usize {
        self.bytes_per_row * self.num_rows(data)
    }

    pub fn get_cur_bytes<'data>(&self, data: &'data [u8]) -> &'data [u8] {
        let max_end = self.cur_pos + self.bytes_per_screen(data);
        let end = max_end.min(data.len());
        &data[self.cur_pos..end]
    }

    pub fn get_selected_bytes<'data>(&self, data: &'data [u8]) -> &'data [u8] {
        match self.selection.state {
            HexViewSelectionState::None => &[],
            HexViewSelectionState::Selecting | HexViewSelectionState::Selected => {
                &data[self.selection.start()..self.selection.end() + 1]
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        data: &[u8],
        cursor_state: CursorState,
        can_selection_change: bool,
        font_id: FontId,
        byte_grouping: usize,
    ) {
        let grid_rect = egui::Grid::new(format!("hex_grid{}", self.id))
            .striped(true)
            .spacing([0.0, 0.0])
            .min_col_width(0.0)
            .num_columns(40)
            .show(ui, |ui| {
                let screen_bytes = self.get_cur_bytes(data);
                let mut current_pos = self.cur_pos;

                let mut row_chunks = screen_bytes.chunks(self.bytes_per_row);

                let mut r = 0;
                let num_rows = self.num_rows(data);
                while r < num_rows {
                    let row: &[u8] = row_chunks.next().unwrap_or_default();

                    let num_digits = match data.len() {
                        //0..=0xFFFF => 4,
                        0x10000..=0xFFFFFFFF => 8,
                        0x100000000..=0xFFFFFFFFFFFF => 12,
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
                                .font(font_id.clone())
                                .color({
                                    if offset_leading_zeros {
                                        self.style.offset_leading_zero_color
                                    } else {
                                        self.style.offset_text_color
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
                        if i > 0 && (i % byte_grouping) == 0 {
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
                                .font(font_id.clone())
                                .color(
                                    // if diff_state.enabled
                                    //     && diff_state.is_diff_at(row_current_pos)
                                    // {
                                    //     self.style.diff_color
                                    // } else {
                                    match byte {
                                        Some(0) => self.style.hex_null_color,
                                        _ => self.style.other_hex_color,
                                    }, // },
                                )
                                .background_color({
                                    if self.selection.contains(row_current_pos) {
                                        self.style.selection_color
                                    } else {
                                        Color32::TRANSPARENT
                                    }
                                }),
                        )
                        .sense(Sense::click_and_drag());

                        let res = ui.add(hex_label);

                        if byte.is_some() {
                            if res.contains_pointer() {
                                self.cursor_pos = Some(row_current_pos);
                            }
                            if can_selection_change {
                                self.handle_selection(
                                    ui,
                                    res,
                                    cursor_state,
                                    row_current_pos,
                                    HexViewSelectionSide::Hex,
                                );
                            }
                        }
                        i += 1;

                        if i < self.bytes_per_row {
                            ui.add(Spacer::default().spacing_x(4.0));
                        }
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
                                .font(font_id.clone())
                                .color(match byte {
                                    Some(0) => self.style.ascii_null_color,
                                    Some(32..=126) => self.style.ascii_color,
                                    _ => self.style.other_ascii_color,
                                })
                                .background_color({
                                    if self.selection.contains(row_current_pos) {
                                        self.style.selection_color
                                    } else {
                                        Color32::TRANSPARENT
                                    }
                                }),
                        )
                        .sense(Sense::click_and_drag());

                        let res = ui.add(hex_label);
                        ui.add(Spacer::default().spacing_x(1.0));

                        if byte.is_some() {
                            if res.contains_pointer() {
                                self.cursor_pos = Some(row_current_pos);
                            }
                            if can_selection_change {
                                self.handle_selection(
                                    ui,
                                    res,
                                    cursor_state,
                                    row_current_pos,
                                    HexViewSelectionSide::Ascii,
                                );
                            }
                        }
                        i += 1;
                    }

                    current_pos += self.bytes_per_row;
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
            if cursor_state == CursorState::Pressed {
                self.selection.begin(row_current_pos, side);
            }

            self.cursor_pos = Some(row_current_pos);
        }

        if let Some(cursor_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if res.rect.contains(cursor_pos) {
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
