use std::borrow::Cow;

use iced::advanced::renderer::Quad;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout, mouse, renderer, text, Layout, Widget};
use iced::{alignment, event, touch, Color, Element, Length, Pixels, Rectangle, Size};

pub use self::text::{LineHeight, Shaping};

pub fn byte_text<'a, Message, Renderer>(
    content: impl ToString,
    grid_pos: u32,
    selected: bool,
    on_selected: impl Fn(u32) -> Message + 'static,
) -> Text<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    Text::new(content.to_string(), grid_pos, selected, on_selected)
}

pub struct Text<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Cow<'a, str>,
    size: Option<f32>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Renderer::Font>,
    shaping: Shaping,
    style: <Renderer::Theme as StyleSheet>::Style,
    grid_pos: u32,
    selected: bool,
    on_selected: Box<dyn Fn(u32) -> Message>,
}

impl<'a, Message, Renderer> Text<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub fn new(
        content: impl Into<Cow<'a, str>>,
        grid_pos: u32,
        selected: bool,
        on_selected: impl Fn(u32) -> Message + 'static,
    ) -> Self {
        Text {
            content: content.into(),
            size: None,
            line_height: LineHeight::default(),
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            #[cfg(debug_assertions)]
            shaping: Shaping::Basic,
            #[cfg(not(debug_assertions))]
            shaping: Shaping::Advanced,
            style: Default::default(),
            grid_pos,
            selected,
            on_selected: Box::new(on_selected),
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into().0);
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn horizontal_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn vertical_alignment(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Text<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let size = self.size.unwrap_or_else(|| renderer.default_size());

        let bounds = limits.max();

        let Size { width, height } = renderer.measure(
            &self.content,
            size,
            self.line_height,
            self.font.unwrap_or_else(|| renderer.default_font()),
            bounds,
            self.shaping,
        );

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | iced::Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(cursor) = cursor.position() {
                    *state = State::Selecting;

                    if layout.bounds().contains(cursor) {
                        shell.publish((self.on_selected)(self.grid_pos));
                    }
                } else {
                    *state = State::Idle;
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | iced::Event::Touch(touch::Event::FingerLifted { .. })
            | iced::Event::Touch(touch::Event::FingerLost { .. }) => {
                if let State::Selecting = *state {
                    *state = State::Selected;
                } else {
                    *state = State::Idle;
                }
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. })
            | iced::Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(cursor) = cursor.position() {
                    if let State::Selecting = state {
                        if layout.bounds().contains(cursor) {
                            shell.publish((self.on_selected)(self.grid_pos));
                        }
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if viewport.intersection(&bounds).is_none() {
            return;
        }

        let appearance = theme.appearance(&self.style);

        if self.selected {
            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds(),
                    border_radius: 0.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                appearance.selection_color,
            );
        }

        draw(
            renderer,
            layout,
            &self.content,
            self.size,
            self.line_height,
            self.font,
            appearance.color.unwrap_or(style.text_color),
            self.horizontal_alignment,
            self.vertical_alignment,
            self.shaping,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw<Renderer>(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    content: &str,
    size: Option<f32>,
    line_height: LineHeight,
    font: Option<Renderer::Font>,
    value_color: Color,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    shaping: Shaping,
) where
    Renderer: text::Renderer,
{
    let bounds = layout.bounds();

    let x = match horizontal_alignment {
        alignment::Horizontal::Left => bounds.x,
        alignment::Horizontal::Center => bounds.center_x(),
        alignment::Horizontal::Right => bounds.x + bounds.width,
    };

    let y = match vertical_alignment {
        alignment::Vertical::Top => bounds.y,
        alignment::Vertical::Center => bounds.center_y(),
        alignment::Vertical::Bottom => bounds.y + bounds.height,
    };

    let size = size.unwrap_or_else(|| renderer.default_size());

    renderer.fill_text(iced::advanced::Text {
        content,
        size,
        line_height,
        bounds: Rectangle { x, y, ..bounds },
        color: value_color,
        font: font.unwrap_or_else(|| renderer.default_font()),
        horizontal_alignment,
        vertical_alignment,
        shaping,
    });
}

impl<'a, Message, Renderer> From<Text<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Message: 'a + Clone,
{
    fn from(text: Text<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}

#[derive(Debug, Clone, Copy, Default)]
enum State {
    #[default]
    Idle,
    Selecting,
    Selected,
}

pub trait StyleSheet {
    type Style: Default;

    fn appearance(&self, style: &Self::Style) -> Appearance;
}

pub struct Appearance {
    pub color: Option<Color>,
    pub selection_color: Color,
}
