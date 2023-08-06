pub mod error;
pub mod theme;
pub mod theme_data;
pub mod widget;
pub mod window;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::result;

use argh::FromArgs;

use error::Error;
use iced::widget::{container, text, Column, Row, Space, Text};
use iced::{
    subscription, Application, Color, Command, Element, Event, Font, Renderer, Settings,
    Subscription,
};

pub use self::theme::Theme;

use crate::widget::selectable_text;

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

    println!(
        "Read {} bytes from {}",
        buffer.len(),
        path.file_name().unwrap().to_str().unwrap()
    );

    Ok(BinFile {
        path: path.to_str().unwrap().to_string(),
        data: buffer,
    })
}

#[derive(Debug, Default)]
struct HexView {
    file: BinFile,
    cur_pos: usize,
    num_rows: u32,
    bytes_per_row: usize,
    theme: Theme,
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
enum Message {
    FileLoaded(Result<BinFile, Error>),
    EventOccurred(Event),
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
                cur_pos: 0,
                num_rows: 30,
                bytes_per_row: 0x10,
                theme: Theme::default().clone().into(),
            },
            Command::perform(async { read_file_result }, Message::FileLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("BDiff")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileLoaded(Ok(bin_file)) => {
                *self = HexView {
                    file: bin_file,
                    cur_pos: 0,
                    num_rows: 30,
                    bytes_per_row: 0x10,
                    theme: Theme::default().clone().into(),
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
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        let content = {
            let file_name_text: Text = Text::new(self.file.path.clone()).size(20);

            let hex_rows: Vec<HexRow> = self.get_cur_hex_rows();

            let mut ui_rows: Vec<Element<Message, Renderer>> = hex_rows
                .iter()
                .map(|row| {
                    let mut row_children: Vec<Element<Message, Renderer>> = Vec::new();

                    let offset_text: Element<Message> = Element::from(
                        text(format!(
                            "{:04X?} {:04X?}",
                            row.offset >> 0x10,
                            row.offset % 0x10000
                        ))
                        .font(Font::with_name("Consolas"))
                        .style(Color::from_rgb8(0x98, 0x98, 0x98)),
                    );

                    let mut hex_text_elems: Vec<Element<Message, Renderer>> = Vec::new();
                    for (i, byte) in row.data.iter().enumerate() {
                        let hex_color: Color = match *byte {
                            0 => Color::from_rgb8(0x80, 0x80, 0x80),
                            _ => Color::WHITE,
                        };

                        let text_element = text(format!("{:02X?}", byte))
                            .font(Font::with_name("Consolas"))
                            .style(hex_color);

                        if i > 0 {
                            if (i % 8) == 0 {
                                hex_text_elems.push(Element::from(Space::with_width(10)));
                            } else {
                                hex_text_elems.push(Element::from(Space::with_width(5)));
                            }
                        }
                        hex_text_elems.push(Element::from(text_element));
                    }

                    let mut ascii_texts: Vec<Element<Message, Renderer>> = row
                        .data
                        .iter()
                        .map(|byte| {
                            let ascii_char: char = match *byte {
                                32..=126 => *byte as char,
                                _ => '·',
                            };
                            let ascii_color: Color = match *byte {
                                0 => Color::from_rgb8(0x40, 0x40, 0x40),
                                32..=126 => Color::WHITE,
                                _ => Color::from_rgb8(0x80, 0x80, 0x80),
                            };
                            text(ascii_char)
                                .font(Font::with_name("Consolas"))
                                .style(ascii_color)
                        })
                        .map(Element::from)
                        .collect();

                    let somethin = selectable_text("MEOW test 123   ahahaha");

                    row_children.push(Element::from(somethin));
                    row_children.push(offset_text);
                    row_children.push(Element::from(Space::with_width(10)));
                    row_children.append(&mut hex_text_elems);
                    row_children.push(Element::from(Space::with_width(10)));
                    row_children.append(&mut ascii_texts);

                    Row::with_children(row_children)
                })
                .map(Element::from)
                .collect();

            ui_rows.insert(0, Element::from(file_name_text));

            let hex_table = Column::with_children(ui_rows);

            hex_table.max_width(700)
        };

        container(content)
            //.style(theme::Container::Box)
            //     .width(Length::Fill)
            //     .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

#[derive(Default, Debug, Clone)]
struct BinFile {
    path: String,
    data: Vec<u8>,
}
