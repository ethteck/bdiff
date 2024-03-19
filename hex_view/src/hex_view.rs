use crate::spacer::Spacer;

use eframe::egui::{self, Color32, Sense, Separator};
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

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Color(pub [u8; 4]);

impl Color {
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8; 4] {
        &mut self.0
    }
}

impl From<Color32> for Color {
    fn from(value: Color32) -> Self {
        Self(value.to_array())
    }
}

impl From<Color> for Color32 {
    fn from(value: Color) -> Self {
        let sc = value.0;
        Color32::from_rgba_premultiplied(sc[0], sc[1], sc[2], sc[3])
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct HexViewDisplaySettings {
    pub selection_color: Color,

    // Offset colors
    pub offset_text_color: Color,
    pub offset_leading_zero_color: Color,

    // Hex View colors
    pub diff_color: Color,
    pub hex_null_color: Color,
    pub other_hex_color: Color,

    // ASCII View colors
    pub ascii_null_color: Color,
    pub ascii_color: Color,
    pub other_ascii_color: Color,
}

struct HexView {
    pub id: usize,
    pub file: BinFile,
    pub num_rows: u32,
    pub bytes_per_row: usize,
    pub cur_pos: usize,
    pub pos_locked: bool,
    pub selection: HexViewSelection,
    pub cursor_pos: Option<usize>,
    pub show_selection_info: bool,
    pub show_cursor_info: bool,
    pub closed: bool,
}

impl Default for HexView {
    fn default() -> Self {
        Self {
            id: 0,
            file: BinFile::default(),
            num_rows: 0,
            bytes_per_row: 0,
            cur_pos: 0,
            pos_locked: false,
            selection: HexViewSelection::default(),
            cursor_pos: None,
            show_selection_info: true,
            show_cursor_info: true,
            closed: false,
        }
    }
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
        if self.pos_locked {
            return;
        }
        let last_line_start_address =
            (self.file.data.len() / self.bytes_per_row) * self.bytes_per_row;
        self.cur_pos = val.clamp(0, last_line_start_address);
    }

    pub fn adjust_cur_pos(&mut self, delta: isize) {
        if self.pos_locked {
            return;
        }
        let last_line_start_address =
            (self.file.data.len() / self.bytes_per_row) * self.bytes_per_row;
        self.cur_pos =
            (self.cur_pos as isize + delta).clamp(0, last_line_start_address as isize) as usize;
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

    fn show(
        &mut self,
        diff_state: &DiffState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        cursor_state: CursorState,
        can_selection_change: bool,
        font_size: f32,
        byte_grouping: usize,
        display_settings: HexViewDisplaySettings,
    ) {
        let grid_rect = ui
            .group(|ui| {
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
                            let row: &[u8] = row_chunks.next().unwrap_or_default();

                            let num_digits = match self.file.data.len() {
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
                                        .monospace()
                                        .size(font_size)
                                        .color({
                                            if offset_leading_zeros {
                                                Color32::from(
                                                    display_settings
                                                        .offset_leading_zero_color
                                                        .clone(),
                                                )
                                            } else {
                                                Color32::from(
                                                    display_settings.offset_text_color.clone(),
                                                )
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
                                        .monospace()
                                        .size(font_size)
                                        .color(
                                            if diff_state.enabled
                                                && diff_state.is_diff_at(row_current_pos)
                                            {
                                                Color32::from(display_settings.diff_color.clone())
                                            } else {
                                                match byte {
                                                    Some(0) => Color32::from(
                                                        display_settings.hex_null_color.clone(),
                                                    ),
                                                    _ => Color32::from(
                                                        display_settings.other_hex_color.clone(),
                                                    ),
                                                }
                                            },
                                        )
                                        .background_color({
                                            if self.selection.contains(row_current_pos) {
                                                display_settings.selection_color.clone().into()
                                            } else {
                                                Color32::TRANSPARENT
                                            }
                                        }),
                                )
                                .sense(Sense::click_and_drag());

                                let res = ui.add(hex_label);

                                if byte.is_some() {
                                    if res.hovered() {
                                        self.cursor_pos = Some(row_current_pos);
                                    }
                                    if can_selection_change {
                                        self.handle_selection(
                                            res,
                                            cursor_state,
                                            row_current_pos,
                                            ctx,
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
                                        .monospace()
                                        .size(font_size)
                                        .color(match byte {
                                            Some(0) => Color32::from(
                                                display_settings.ascii_null_color.clone(),
                                            ),
                                            Some(32..=126) => {
                                                Color32::from(display_settings.ascii_color.clone())
                                            }
                                            _ => Color32::from(
                                                display_settings.other_ascii_color.clone(),
                                            ),
                                        })
                                        .background_color({
                                            if self.selection.contains(row_current_pos) {
                                                display_settings.selection_color.clone().into()
                                            } else {
                                                Color32::TRANSPARENT
                                            }
                                        }),
                                )
                                .sense(Sense::click_and_drag());

                                let res = ui.add(hex_label);
                                ui.add(Spacer::default().spacing_x(1.0));

                                if byte.is_some() {
                                    if res.hovered() {
                                        self.cursor_pos = Some(row_current_pos);
                                    }
                                    if can_selection_change {
                                        self.handle_selection(
                                            res,
                                            cursor_state,
                                            row_current_pos,
                                            ctx,
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
                    });
            })
            .response
            .rect;

        if let Some(cursor_pos) = ctx.input(|i| i.pointer.hover_pos()) {
            if !grid_rect.contains(cursor_pos) {
                self.cursor_pos = None;
            }
        }
    }

    fn handle_selection(
        &mut self,
        res: egui::Response,
        cursor_state: CursorState,
        row_current_pos: usize,
        ctx: &egui::Context,
        side: HexViewSelectionSide,
    ) {
        if res.hovered() {
            if cursor_state == CursorState::Pressed {
                self.selection.begin(row_current_pos, side);
            }

            self.cursor_pos = Some(row_current_pos);
        }

        if let Some(cursor_pos) = ctx.input(|i| i.pointer.hover_pos()) {
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
