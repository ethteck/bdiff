pub mod error;
pub mod theme;
pub mod theme_data;
mod watcher;
pub mod widget;
mod window;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::result::Result;

use argh::FromArgs;

use error::BDiffError;
use iced::widget::text;
use iced::widget::{container, row};
use iced::{clipboard, subscription, Application, Command, Event, Font, Settings, Subscription};
use iced_core::mouse::Button;
use widget::clip_viewport::ClipViewport;
use widget::{Column, Renderer, Row, Space};

pub use self::theme::Theme;
use self::widget::Element;

use crate::widget::byte_text;

use encoding_rs::*;

#[derive(FromArgs)]
/// binary differ
struct Args {
    /// input files
    #[argh(positional)]
    files: Vec<PathBuf>,
}

struct Flags {
    file_paths: Vec<PathBuf>,
}

fn main() -> iced::Result {
    let args: Args = argh::from_env();

    BDiff::run(Settings {
        id: None,
        window: window::settings(),
        flags: Flags {
            file_paths: args.files,
        },
        default_font: Font::with_name("Consolas"),
        default_text_size: 16.0,
        antialiasing: false,
        exit_on_close_request: true,
    })
}

fn read_file(path: PathBuf) -> Result<BinFile, BDiffError> {
    let file = match File::open(path.clone()) {
        Ok(file) => file,
        Err(_error) => return Result::Err(BDiffError::IOError),
    };

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    let _ = buf_reader
        .read_to_end(&mut buffer)
        .or(Err(BDiffError::IOError));

    Ok(BinFile {
        path: path.to_str().unwrap().to_string(),
        data: buffer,
    })
}

#[derive(Default, Debug, Clone)]
pub struct BinFile {
    path: String,
    data: Vec<u8>,
}

#[derive(Default, Debug, PartialEq)]
enum SelectionState {
    #[default]
    None,
    Selecting,
    Selected,
}

#[derive(Debug, Default)]
struct HVSelection {
    first: usize,
    second: usize,
    state: SelectionState,
}

impl HVSelection {
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
struct BDiff {
    hex_views: HashMap<Id, HexView>,
    theme: Theme,
    focused_hex_view: Option<Id>,
}

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

#[derive(Default)]
struct HexView {
    id: Id,
    file: BinFile,
    num_rows: u32,
    bytes_per_row: usize,
    cur_pos: usize,
    selection: HVSelection,
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

#[derive(Debug, Clone)]
pub enum Message {
    WatchedFileChanged(notify::Event),
    FileReloaded(Result<BinFile, BDiffError>),
    EventOccurred(Event),
    CopySelection,
    SelectionAdded(Id, u32),
}

struct HexRow {
    offset: usize,
    data: Vec<u8>,
}

impl Application for BDiff {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let paths = flags.file_paths;
        let mut hex_views: HashMap<Id, HexView> = HashMap::new();
        let min_rows = 10;
        let max_rows = 20;
        let bytes_per_row = 0x10;

        for path in paths {
            let file = read_file(path.clone()).unwrap();

            let num_rows = (file.data.len() / bytes_per_row).clamp(min_rows, max_rows) as u32;

            let id = Id::unique();

            hex_views.insert(
                id.clone(),
                HexView {
                    id,
                    file,
                    num_rows,
                    bytes_per_row,
                    ..Default::default()
                },
            );
        }

        (
            Self {
                hex_views,
                theme: Theme::default(),
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("bdiff")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::WatchedFileChanged(_) => todo!(),
            Message::FileReloaded(_) => todo!(),
            Message::EventOccurred(_) => {
                for hex_view in self.hex_views.values_mut() {
                    let _ = hex_view.update(message.clone()); // TODO does this ever have to return a command?
                }
                Command::none()
            }
            Message::CopySelection => {
                if let Some(focused) = &self.focused_hex_view {
                    if let Some(hex_view) = self.hex_views.get_mut(focused) {
                        return hex_view.update(message.clone());
                    }
                }
                Command::none()
            }
            Message::SelectionAdded(hex_view_id, grid_pos) => {
                if let Some(hex_view) = self.hex_views.get_mut(&hex_view_id) {
                    hex_view.update_selection_state(grid_pos);
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let cols = Column::with_children(
            self.hex_views
                .values()
                .map(|hex_view| hex_view.view())
                .collect(),
        );
        container(cols).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let events_sub = subscription::events().map(Message::EventOccurred);

        let hex_view_subs = self
            .hex_views
            .values()
            .map(|hex_view| hex_view.subscription());

        let mut subs = vec![events_sub];
        subs.extend(hex_view_subs);

        Subscription::batch(subs)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

impl HexView {
    fn update(&mut self, message: Message) -> Command<Message> {
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
            Message::EventOccurred(event) => {
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
            Message::SelectionAdded(_, _) => todo!(),
        }
    }

    fn update_selection_state(&mut self, grid_pos: u32) {
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

    fn subscription(&self) -> Subscription<Message> {
        watcher::subscription(self.file.path.clone())
    }

    fn view(&self) -> Element<Message> {
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

            let mut ui_rows: Vec<Element<Message>> = vec![
                Element::from(file_name_text),
                Element::from(data_row),
                //Element::from(Rule::horizontal(1)),
            ];

            match self.selection.state {
                SelectionState::None => (),
                SelectionState::Selecting | SelectionState::Selected => {
                    let selection_display = text(format!(
                        "Selection: 0x{:X} : 0x{:X}",
                        self.selection.start(),
                        self.selection.end() + 1
                    ));
                    ui_rows.push(Element::from(selection_display));
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
                            ui_rows.push(Element::from(text(format!("s8/u8: {:}", signed))));
                        } else {
                            ui_rows.push(Element::from(text(format!("s8: {:}", signed))));

                            let unsigned = u8::from_be_bytes(bytes);
                            ui_rows.push(Element::from(text(format!("u8: {:}", unsigned))));
                        }
                    }
                    2 => {
                        let bytes: [u8; 2] = selected_bytes[0..2].try_into().unwrap_or_default();

                        let signed = i16::from_be_bytes(bytes);
                        if signed >= 0 {
                            ui_rows.push(Element::from(text(format!("s16/u16: {:}", signed))));
                        } else {
                            ui_rows.push(Element::from(text(format!("s16: {:}", signed))));

                            let unsigned = u16::from_be_bytes(bytes);
                            ui_rows.push(Element::from(text(format!("u16: {:}", unsigned))));
                        }
                    }
                    4 => {
                        let bytes: [u8; 4] = selected_bytes[0..4].try_into().unwrap_or_default();

                        let signed = i32::from_be_bytes(bytes);
                        if signed >= 0 {
                            ui_rows.push(Element::from(text(format!("s32/u32: {:}", signed))));
                        } else {
                            ui_rows.push(Element::from(text(format!("s32: {:}", signed))));

                            let unsigned = u32::from_be_bytes(bytes);
                            ui_rows.push(Element::from(text(format!("u32: {:}", unsigned))));
                        }

                        ui_rows.push(Element::from(text(format!(
                            "f32: {:.}",
                            f32::from_be_bytes(bytes)
                        ))));
                    }
                    8 => {
                        let bytes: [u8; 8] = selected_bytes[0..8].try_into().unwrap_or_default();

                        let signed = i64::from_be_bytes(bytes);
                        if signed >= 0 {
                            ui_rows.push(Element::from(text(format!("s64/u64: {:}", signed))));
                        } else {
                            ui_rows.push(Element::from(text(format!("s64: {:}", signed))));

                            let unsigned = u64::from_be_bytes(bytes);
                            ui_rows.push(Element::from(text(format!("u64: {:}", unsigned))));
                        }

                        ui_rows.push(Element::from(text(format!(
                            "f64: {:.}",
                            f64::from_be_bytes(bytes)
                        ))));
                    }
                    _ => (),
                }

                // Strings
                if selection_len > 0 {
                    if let Ok(string) = String::from_utf8(selected_bytes.clone()) {
                        ui_rows.push(Element::from(
                            text(format!("UTF-8: {:}", string)).font(Font::with_name("Meiryo")),
                        ));
                    }

                    if let Some(string) = UTF_16BE
                        .decode_without_bom_handling_and_without_replacement(&selected_bytes)
                    {
                        ui_rows.push(Element::from(
                            text(format!("UTF-16: {:}", string)).font(Font::with_name("Meiryo")),
                        ));
                    }

                    if let Some(string) =
                        EUC_JP.decode_without_bom_handling_and_without_replacement(&selected_bytes)
                    {
                        ui_rows.push(Element::from(
                            text(format!("EUC-JP: {:}", string)).font(Font::with_name("Meiryo")),
                        ));
                    }

                    if let Some(string) = SHIFT_JIS
                        .decode_without_bom_handling_and_without_replacement(&selected_bytes)
                    {
                        ui_rows.push(Element::from(
                            text(format!("Shift_JIS: {:}", string)).font(Font::with_name("Meiryo")),
                        ));
                    }
                }
            }

            let hex_table = Column::with_children(ui_rows).padding(10);

            hex_table
        };

        container(content)
            .style(theme::Container::PaneBody { selected: false })
            //.width(Length::Shrink)
            //.height(Length::Shrink)
            .into()
    }
}
