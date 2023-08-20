use iced::{
    clipboard,
    widget::{container, row, text},
    Command, Subscription,
};
use iced_core::{mouse::Button, Event, Font, Length};

use crate::{
    file_watcher, read_file, theme,
    widget::{byte_text, clip_viewport::ClipViewport, Column, Element, Renderer, Row, Space},
    BinFile, Message,
};

use encoding_rs::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(iced_core::widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(iced_core::widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(iced_core::widget::Id::unique())
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::unique()
    }
}

impl From<Id> for iced_core::widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

#[derive(Default, Debug, PartialEq)]
enum SelectionState {
    #[default]
    None,
    Selecting,
    Selected,
}

#[derive(Debug, Default)]
pub struct Selection {
    first: usize,
    second: usize,
    state: SelectionState,
}

impl Selection {
    fn start(&self) -> usize {
        self.first.min(self.second)
    }

    fn end(&self) -> usize {
        self.second.max(self.first)
    }

    fn contains(&self, grid_pos: usize) -> bool {
        self.state != SelectionState::None && grid_pos >= self.start() && grid_pos <= self.end()
    }

    fn clear(&mut self) {
        self.first = 0;
        self.second = 0;
        self.state = SelectionState::None;
    }
}

#[derive(Default)]
pub struct HexView {
    pub id: Id,
    pub file: BinFile,
    pub num_rows: u32,
    pub bytes_per_row: usize,
    pub cur_pos: usize,
    pub selection: Selection,
}

struct HexRow {
    offset: usize,
    data: Vec<u8>,
}

impl HexView {
    fn set_cur_pos(&mut self, val: usize) {
        self.cur_pos = val.min(self.file.data.len())
    }

    fn adjust_cur_pos(&mut self, delta: i32) {
        self.cur_pos =
            (self.cur_pos as i32 + delta).clamp(0, self.file.data.len() as i32 - 0x20) as usize;
    }

    fn bytes_per_screen(&self) -> i32 {
        (self.bytes_per_row * self.num_rows as usize) as i32
    }

    fn get_cur_hex_rows(&self) -> Vec<HexRow> {
        let mut row_start: usize = self.cur_pos;

        let mut hex_rows = Vec::new();
        let mut i = 0;
        while i < self.num_rows && row_start < self.file.data.len() {
            let row_end = (row_start + self.bytes_per_row).min(self.file.data.len());

            hex_rows.push(HexRow {
                offset: row_start,
                data: self.file.data[row_start..row_end].to_vec(),
            });
            row_start += self.bytes_per_row;
            i += 1;
        }
        hex_rows
    }

    fn get_selected_bytes(&self) -> Option<Vec<u8>> {
        match self.selection.state {
            SelectionState::None => None,
            SelectionState::Selecting | SelectionState::Selected => {
                Some(self.file.data[self.selection.start()..self.selection.end() + 1].to_vec())
            }
        }
    }
}

impl HexView {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileReloaded(Ok(bin_file)) => {
                self.file = bin_file;
                if self.selection.start() > self.file.data.len()
                    || self.selection.end() > self.file.data.len()
                {
                    self.selection.clear();
                }
                Command::none()
            }
            Message::FileReloaded(Err(_error)) => Command::none(),
            Message::Event(event) => {
                match event {
                    Event::Mouse(iced::mouse::Event::WheelScrolled {
                        delta: iced::mouse::ScrollDelta::Lines { y, .. },
                    }) => {
                        self.adjust_cur_pos(-(y as i32) * self.bytes_per_row as i32);
                    }
                    Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left)) => {
                        match self.selection.state {
                            SelectionState::None => (),
                            SelectionState::Selecting => {
                                self.selection.state = SelectionState::Selected;
                            }
                            SelectionState::Selected => (),
                        }
                    }
                    Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Middle)) => {
                        self.selection.clear();
                    }
                    Event::Keyboard(iced::keyboard::Event::KeyPressed { key_code, .. }) => {
                        match key_code {
                            iced::keyboard::KeyCode::Home => self.set_cur_pos(0),
                            iced::keyboard::KeyCode::End => self.set_cur_pos(
                                self.file.data.len() - self.bytes_per_screen() as usize,
                            ),
                            iced::keyboard::KeyCode::PageUp => {
                                self.adjust_cur_pos(-self.bytes_per_screen())
                            }
                            iced::keyboard::KeyCode::PageDown => {
                                self.adjust_cur_pos(self.bytes_per_screen())
                            }
                            iced::keyboard::KeyCode::Left => self.adjust_cur_pos(-1),
                            iced::keyboard::KeyCode::Right => self.adjust_cur_pos(1),
                            iced::keyboard::KeyCode::Up => {
                                self.adjust_cur_pos(-(self.bytes_per_row as i32))
                            }
                            iced::keyboard::KeyCode::Down => {
                                self.adjust_cur_pos(self.bytes_per_row as i32)
                            }
                            iced::keyboard::KeyCode::Enter => {
                                self.adjust_cur_pos(self.bytes_per_screen())
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }

                Command::none()
            }
            Message::CopySelection => {
                let selected_bytes = self.get_selected_bytes().unwrap_or_default();
                let contents = String::from_utf8(selected_bytes).unwrap_or_default();

                if !contents.is_empty() {
                    return clipboard::write(contents);
                }
                Command::none()
            }
            Message::WatchedFileChanged(event) => {
                let pbuf = event.paths[0].clone();

                Command::perform(async { read_file(pbuf) }, Message::FileReloaded)
            }
            _ => Command::none(),
        }
    }

    pub fn update_selection_state(&mut self, grid_pos: u32) {
        let selection_pos = self.cur_pos + grid_pos as usize;

        match self.selection.state {
            SelectionState::Selecting => {
                self.selection.second = selection_pos;
            }
            SelectionState::None | SelectionState::Selected => {
                self.selection.state = SelectionState::Selecting;
                self.selection.first = selection_pos;
                self.selection.second = selection_pos;
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        file_watcher::subscription(self.file.path.clone())
    }

    pub fn view(&self) -> Element<Message> {
        let content = {
            let file_name_text = text(self.file.path.clone());

            let hex_rows: Vec<HexRow> = self.get_cur_hex_rows();

            let mut offsets_col_vec: Vec<Element<Message>> = Vec::new();
            let mut hex_col_vec: Vec<Element<Message>> = Vec::new();
            let mut ascii_col_vec: Vec<Element<Message>> = Vec::new();

            for (r, row) in hex_rows.iter().enumerate() {
                let mut offset_text_elems: Vec<Element<Message>> = Vec::new();
                let num_digits = 8; // 8 of those boys
                let mut i = num_digits;
                let mut offset_leading_zeros = true;

                while i > 0 {
                    let digit = row.offset >> ((i - 1) * 4) & 0xF;

                    if offset_leading_zeros && digit > 0 {
                        offset_leading_zeros = false;
                    }
                    let style = match offset_leading_zeros {
                        true => theme::Text::Fainter,
                        false => theme::Text::Default,
                    };
                    let offset_digit_text: iced_core::widget::Text<'_, Renderer> =
                        text(format!("{:X?}", digit)).style(style);

                    if i < num_digits && (i % 4) == 0 {
                        offset_text_elems.push(Element::from(Space::with_width(5)));
                    }
                    offset_text_elems.push(Element::from(offset_digit_text));
                    i -= 1;
                }
                let offset_text = Row::with_children(offset_text_elems);

                let mut hex_text_elems: Vec<Element<Message>> = Vec::new();
                for (i, byte) in row.data.iter().enumerate() {
                    let style = match *byte {
                        0 => theme::Text::Fainter,
                        _ => theme::Text::Default,
                    };

                    let grid_pos: usize = r * self.bytes_per_row + i;

                    let text_element = byte_text(
                        format!("{:02X?}", byte),
                        self.id.clone(),
                        grid_pos as u32,
                        self.selection.contains(self.cur_pos + grid_pos),
                        Message::SelectionAdded,
                    )
                    .style(style);

                    if i > 0 {
                        if (i % 8) == 0 {
                            hex_text_elems.push(Element::from(Space::with_width(10)));
                        } else {
                            hex_text_elems.push(Element::from(Space::with_width(5)));
                        }
                    }
                    hex_text_elems.push(Element::from(text_element));
                }
                let hex_text = Row::with_children(hex_text_elems);

                let mut ascii_text_elems: Vec<Element<Message>> = Vec::new();
                for (i, byte) in row.data.iter().enumerate() {
                    let ascii_char: char = match *byte {
                        32..=126 => *byte as char,
                        _ => '·',
                    };

                    let grid_pos: usize = r * self.bytes_per_row + i;

                    let style = match *byte {
                        0 => theme::Text::Faintest,
                        32..=126 => theme::Text::Default,
                        _ => theme::Text::Fainter,
                    };

                    let text_element = byte_text(
                        ascii_char,
                        self.id.clone(),
                        grid_pos as u32,
                        self.selection.contains(self.cur_pos + grid_pos),
                        Message::SelectionAdded,
                    )
                    .style(style);
                    ascii_text_elems.push(Element::from(text_element));
                }
                let ascii_text = Row::with_children(ascii_text_elems);

                offsets_col_vec.push(Element::from(offset_text));
                hex_col_vec.push(Element::from(hex_text));
                ascii_col_vec.push(Element::from(ascii_text));
            }

            let offsets_col = Column::with_children(offsets_col_vec);
            let hex_col = Column::with_children(hex_col_vec);
            let ascii_col = Column::with_children(ascii_col_vec);

            let data_row = row![]
                .push(offsets_col)
                .push(Space::with_width(10))
                .push(ClipViewport::new(hex_col))
                .push(Space::with_width(10))
                .push(ClipViewport::new(ascii_col));

            let ui_rows: Vec<Element<Message>> = vec![
                Element::from(file_name_text),
                Element::from(data_row),
                //Element::from(Rule::horizontal(1)),
            ];

            let mut selection_rows: Vec<Element<Message>> = Vec::new();

            match self.selection.state {
                SelectionState::None => (),
                SelectionState::Selecting | SelectionState::Selected => {
                    let selection_display = text(format!(
                        "Selection: 0x{:X} : 0x{:X}",
                        self.selection.start(),
                        self.selection.end() + 1
                    ));
                    selection_rows.push(Element::from(selection_display));
                }
            }

            if let Some(selected_bytes) = self.get_selected_bytes() {
                let selection_len = selected_bytes.len();

                match selection_len {
                    0 => (),
                    1 => {
                        let bytes: [u8; 1] = selected_bytes[0..1].try_into().unwrap_or_default();

                        let signed = i8::from_be_bytes(bytes);
                        if signed >= 0 {
                            selection_rows.push(Element::from(text(format!("s8/u8: {:}", signed))));
                        } else {
                            selection_rows.push(Element::from(text(format!("s8: {:}", signed))));

                            let unsigned = u8::from_be_bytes(bytes);
                            selection_rows.push(Element::from(text(format!("u8: {:}", unsigned))));
                        }
                    }
                    2 => {
                        let bytes: [u8; 2] = selected_bytes[0..2].try_into().unwrap_or_default();

                        let signed = i16::from_be_bytes(bytes);
                        if signed >= 0 {
                            selection_rows
                                .push(Element::from(text(format!("s16/u16: {:}", signed))));
                        } else {
                            selection_rows.push(Element::from(text(format!("s16: {:}", signed))));

                            let unsigned = u16::from_be_bytes(bytes);
                            selection_rows.push(Element::from(text(format!("u16: {:}", unsigned))));
                        }
                    }
                    4 => {
                        let bytes: [u8; 4] = selected_bytes[0..4].try_into().unwrap_or_default();

                        let signed = i32::from_be_bytes(bytes);
                        if signed >= 0 {
                            selection_rows
                                .push(Element::from(text(format!("s32/u32: {:}", signed))));
                        } else {
                            selection_rows.push(Element::from(text(format!("s32: {:}", signed))));

                            let unsigned = u32::from_be_bytes(bytes);
                            selection_rows.push(Element::from(text(format!("u32: {:}", unsigned))));
                        }

                        selection_rows.push(Element::from(text(format!(
                            "f32: {:.}",
                            f32::from_be_bytes(bytes)
                        ))));
                    }
                    8 => {
                        let bytes: [u8; 8] = selected_bytes[0..8].try_into().unwrap_or_default();

                        let signed = i64::from_be_bytes(bytes);
                        if signed >= 0 {
                            selection_rows
                                .push(Element::from(text(format!("s64/u64: {:}", signed))));
                        } else {
                            selection_rows.push(Element::from(text(format!("s64: {:}", signed))));

                            let unsigned = u64::from_be_bytes(bytes);
                            selection_rows.push(Element::from(text(format!("u64: {:}", unsigned))));
                        }

                        selection_rows.push(Element::from(text(format!(
                            "f64: {:.}",
                            f64::from_be_bytes(bytes)
                        ))));
                    }
                    _ => (),
                }

                // Strings
                if selection_len > 0 {
                    if let Ok(string) = String::from_utf8(selected_bytes.clone()) {
                        selection_rows.push(Element::from(
                            text(format!("UTF-8: {:}", string)).font(Font::MONOSPACE),
                        ));
                    }

                    if let Some(string) = UTF_16BE
                        .decode_without_bom_handling_and_without_replacement(&selected_bytes)
                    {
                        selection_rows.push(Element::from(
                            text(format!("UTF-16: {:}", string)).font(Font::MONOSPACE),
                        ));
                    }

                    if let Some(string) =
                        EUC_JP.decode_without_bom_handling_and_without_replacement(&selected_bytes)
                    {
                        selection_rows.push(Element::from(
                            text(format!("EUC-JP: {:}", string)).font(Font::MONOSPACE),
                        ));
                    }

                    if let Some(string) = SHIFT_JIS
                        .decode_without_bom_handling_and_without_replacement(&selected_bytes)
                    {
                        selection_rows.push(Element::from(
                            text(format!("Shift_JIS: {:}", string)).font(Font::MONOSPACE),
                        ));
                    }
                }
            }

            let hex_table = row![
                container(Column::with_children(ui_rows).padding(10))
                    .style(theme::Container::PaneBody { selected: false }),
                Element::from(Space::with_width(1)),
                container(Column::with_children(selection_rows).width(Length::Fill))
                    .style(theme::Container::PaneBody { selected: false })
                    .width(Length::Fill),
            ];

            hex_table.height(Length::Shrink)
        };

        container(content).into()
    }
}
