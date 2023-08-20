pub mod error;
mod file_watcher;
mod hex_view;
pub mod theme;
pub mod theme_data;
pub mod widget;
mod window;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::result::Result;

use argh::FromArgs;

use error::BDiffError;
use hex_view::{HexView, Id};
use iced::widget::container;
use iced::{subscription, Application, Command, Event, Font, Settings, Subscription};
use widget::Column;

pub use self::theme::Theme;

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

#[derive(Default)]
struct BDiff {
    hex_views: HashMap<Id, HexView>,
    theme: Theme,
    focused_hex_view: Option<Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    WatchedFileChanged(notify::Event),
    FileReloaded(Result<BinFile, BDiffError>),
    Event(Event),
    CopySelection,
    SelectionAdded(Id, u32),
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
            Message::Event(_) => {
                let mut msgs: Vec<Command<Message>> = Vec::new();
                for hex_view in self.hex_views.values_mut() {
                    msgs.push(hex_view.update(message.clone()));
                }
                Command::batch(msgs)
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
        let rows = Column::with_children(
            self.hex_views
                .values()
                .map(|hex_view| hex_view.view())
                .collect(),
        );
        container(rows).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let events_sub = subscription::events().map(Message::Event);

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
