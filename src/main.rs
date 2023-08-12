pub mod error;
pub mod theme;
pub mod theme_data;
pub mod widget;
mod window;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::result;

use argh::FromArgs;

use error::Error;
use iced::widget::text;
use iced::widget::{container, row};
use iced::{clipboard, subscription, Application, Command, Event, Font, Settings, Subscription};
use widget::clip_viewport::ClipViewport;
use widget::{Column, Renderer, Row, Space};

pub use self::theme::Theme;
use self::widget::Element;

use crate::widget::byte_text;

#[derive(FromArgs)]
/// binary differ
struct Args {
    /// input file
    #[argh(positional)]
    file: String,
}

struct Flags {
    file_path: PathBuf,
}

fn main() -> iced::Result {
    let args: Args = argh::from_env();

    let path: &Path = Path::new(&args.file);

    HexView::run(Settings {
        id: None,
        window: window::settings(),
        flags: Flags {
            file_path: path.to_path_buf(),
        },
        default_font: Font::DEFAULT,
        default_text_size: 16.0,
        antialiasing: false,
        exit_on_close_request: true,
    })
}

fn read_file(path: &Path) -> std::result::Result<BinFile, Error> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_error) => return result::Result::Err(Error::IOError),
    };

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    let _ = buf_reader
        .read_to_end(&mut buffer)
        .or(result::Result::Err(Error::IOError));

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

#[derive(Debug, Default)]
struct HVSelection {
    start: u32,
    len: u32,
}

#[derive(Debug, Default)]
struct HexView {
    file: BinFile,
    num_rows: u32,
    bytes_per_row: usize,
    theme: Theme,
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
        while i < self.num_rows && row_start + self.bytes_per_row < self.file.data.len() {
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
}

#[derive(Debug, Clone)]
pub enum Message {
    FileLoaded(Result<BinFile, Error>),
    EventOccurred(Event),
    SelectedText(Vec<(u32, String)>),
    SelectionStarted(u32),
}

struct HexRow {
    offset: usize,
    data: Vec<u8>,
}

impl Application for HexView {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(_flags: Flags) -> (HexView, Command<Message>) {
        let path = _flags.file_path;
        let read_file_result = read_file(&path);

        (
            HexView {
                file: BinFile {
                    path: String::from("Loading"),
                    data: vec![],
                },
                num_rows: 30,
                bytes_per_row: 0x10,
                theme: Theme::default(),
                cur_pos: 0,
                selection: HVSelection { start: 0, len: 0 },
            },
            Command::perform(async { read_file_result }, Message::FileLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("bdiff")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileLoaded(Ok(bin_file)) => {
                *self = HexView {
                    file: bin_file,
                    num_rows: 30,
                    bytes_per_row: 0x10,
                    theme: Theme::default(),
                    selection: HVSelection { start: 0, len: 0 },
                    cur_pos: 0,
                };
                Command::none()
            }
            Message::FileLoaded(Err(_error)) => Command::none(),
            Message::EventOccurred(event) => {
                match event {
                    Event::Mouse(iced::mouse::Event::WheelScrolled {
                        delta: iced::mouse::ScrollDelta::Lines { y, .. },
                    }) => {
                        self.adjust_cur_pos(-(y as i32) * self.bytes_per_row as i32);
                    }
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key_code: iced::keyboard::KeyCode::C,
                        modifiers,
                    }) if modifiers.command() => return byte_text::selected(Message::SelectedText),
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
            Message::SelectedText(contents) => {
                let contents = contents
                    .into_iter()
                    .fold(String::new(), |acc, (_, content)| {
                        format!("{}{}\n", acc, content)
                    });

                if !contents.is_empty() {
                    return clipboard::write(contents);
                }
                Command::none()
            }
            Message::SelectionStarted(grid_pos) => {
                println!("{:} selected", grid_pos);
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        let content = {
            let file_name_text = text(self.file.path.clone()).font(Font::with_name("Consolas"));

            let hex_rows: Vec<HexRow> = self.get_cur_hex_rows();

            let mut offsets_col_vec: Vec<Element<Message>> = Vec::new();
            let mut hex_col_vec: Vec<Element<Message>> = Vec::new();
            let mut ascii_col_vec: Vec<Element<Message>> = Vec::new();

            for (r, row) in hex_rows.iter().enumerate() {
                let mut offset_text_elems: Vec<Element<Message>> = Vec::new();
                let num_digits = 8; // 8 of those boys
                let mut i = num_digits;
                let mut leading = true;

                while i > 0 {
                    let digit = row.offset >> ((i - 1) * 4) & 0xF;

                    if leading && digit > 0 {
                        leading = false;
                    }
                    let style = match leading {
                        true => theme::Text::Fainter,
                        false => theme::Text::Default,
                    };
                    let offset_digit_text: iced_core::widget::Text<'_, Renderer> =
                        text(format!("{:X?}", digit))
                            .font(Font::with_name("Consolas"))
                            .style(style);

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
                        grid_pos as u32,
                        Message::SelectionStarted,
                    )
                    .font(Font::with_name("Consolas"))
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
                for byte in &row.data {
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

                    let text_element =
                        byte_text(ascii_char, grid_pos as u32, Message::SelectionStarted)
                            .font(Font::with_name("Consolas"))
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
                .push(ascii_col);

            let f32_display = text(format!("{:}", 5.0)).font(Font::with_name("Consolas"));

            let ui_rows: Vec<Element<Message>> = vec![
                Element::from(file_name_text),
                Element::from(data_row),
                Element::from(f32_display),
            ];

            let hex_table = Column::with_children(ui_rows).padding(10);

            hex_table.max_width(700)
        };

        container(content)
            .style(theme::Container::PaneBody { selected: false })
            // .width(Length::Fill)
            // .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
