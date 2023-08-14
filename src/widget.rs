pub use self::byte_text::byte_text;
use crate::Theme;

pub mod byte_text;
pub mod clip_viewport;

pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
pub type Content<'a, Message> = iced::widget::pane_grid::Content<'a, Message, Renderer>;
pub type TitleBar<'a, Message> = iced::widget::pane_grid::TitleBar<'a, Message, Renderer>;
pub type Column<'a, Message> = iced::widget::Column<'a, Message, Renderer>;
pub type Row<'a, Message> = iced::widget::Row<'a, Message, Renderer>;
pub type Text<'a> = iced::widget::Text<'a, Renderer>;
pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
pub type Button<'a, Message> = iced::widget::Button<'a, Message>;
pub type Space = iced::widget::Space;
pub type Rule<Renderer> = iced::widget::Rule<Renderer>;
