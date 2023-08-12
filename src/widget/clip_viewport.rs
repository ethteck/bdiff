use iced::advanced::Widget;
use iced::{mouse, Element, Length};
use iced_core::widget::tree::{self};
use iced_core::widget::{Operation, Tree};
use iced_core::{event, layout, renderer, Clipboard, Event, Layout, Rectangle, Shell, Size};

/// The identifier of a [`ClipViewport`].
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

impl From<Id> for iced_core::widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}
pub struct ClipViewport<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    width: Length,
    height: Length,
    style: <Renderer::Theme as StyleSheet>::Style,
    content: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> ClipViewport<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`ClipViewport`].
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        ClipViewport {
            id: None,
            width: Length::Shrink,
            height: Length::Shrink,
            style: Default::default(),
            content: content.into(),
        }
    }

    /// Sets the [`Id`] of the [`ClipViewport`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the width of the [`ClipViewport`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`ClipViewport`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the style of the [`ClipViewport`]
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> From<ClipViewport<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Message: 'a + Clone,
{
    fn from(text: ClipViewport<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    #[default]
    Default,
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for ClipViewport<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            |renderer, limits| self.content.as_widget().layout(renderer, limits),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();

        operation.container(self.id.as_ref().map(|id| &id.0), bounds, &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        update(
            tree.state.downcast_mut::<State>(),
            event,
            layout,
            cursor,
            clipboard,
            shell,
            |event, layout, cursor, clipboard, shell, viewport| {
                self.content.as_widget_mut().on_event(
                    &mut tree.children[0],
                    event,
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            },
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        draw(
            tree.state.downcast_ref::<State>(),
            renderer,
            theme,
            layout,
            cursor,
            |renderer, layout, cursor, viewport| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    viewport,
                )
            },
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(
            tree.state.downcast_ref::<State>(),
            layout,
            cursor,
            |layout, cursor, viewport| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor,
                    viewport,
                    renderer,
                )
            },
        )
    }

    // fn overlay<'b>(
    //     &'b mut self,
    //     tree: &'b mut Tree,
    //     layout: Layout<'_>,
    //     renderer: &Renderer,
    // ) -> Option<overlay::Element<'b, Message, Renderer>> {
    //     self.content
    //         .as_widget_mut()
    //         .overlay(
    //             &mut tree.children[0],
    //             layout.children().next().unwrap(),
    //             renderer,
    //         )
    //         .map(|overlay| {
    //             let bounds = layout.bounds();
    //             let content_layout = layout.children().next().unwrap();
    //             let content_bounds = content_layout.bounds();
    //         })
    // }
}

/// Computes the layout of a [`ClipViewport`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.width(width).height(height);

    let child_limits = layout::Limits::new(
        Size::new(limits.min().width, limits.min().height),
        Size::new(limits.max().width, limits.max().height),
    );

    let content = layout_content(renderer, &child_limits);
    let size = limits.resolve(content.size());

    layout::Node::with_children(size, vec![content])
}

/// Processes an [`Event`] and updates the [`State`] of a [`ClipViewport`]
/// accordingly.
pub fn update<Message>(
    state: &mut State,
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    update_content: impl FnOnce(
        Event,
        Layout<'_>,
        mouse::Cursor,
        &mut dyn Clipboard,
        &mut Shell<'_, Message>,
        &Rectangle,
    ) -> event::Status,
) -> event::Status {
    let bounds = layout.bounds();
    let cursor_over_scrollable = cursor.position_over(bounds);

    let content = layout.children().next().unwrap();
    let content_bounds = content.bounds();

    let event_status = {
        update_content(
            event.clone(),
            content,
            cursor,
            clipboard,
            shell,
            &Rectangle {
                y: bounds.y,
                x: bounds.x,
                ..bounds
            },
        )
    };
    event::Status::Ignored
}

/// Computes the current [`mouse::Interaction`] of a [`ClipViewport`].
pub fn mouse_interaction(
    state: &State,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    content_interaction: impl FnOnce(Layout<'_>, mouse::Cursor, &Rectangle) -> mouse::Interaction,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let cursor_over_scrollable = cursor.position_over(bounds);

    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    let cursor = match cursor_over_scrollable {
        Some(cursor_position) => mouse::Cursor::Available(cursor_position),
        _ => mouse::Cursor::Unavailable,
    };

    content_interaction(
        content_layout,
        cursor,
        &Rectangle {
            y: bounds.y,
            x: bounds.x,
            ..bounds
        },
    )
}

/// Draws a [`ClipViewport`].
pub fn draw<Renderer>(
    state: &State,
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    draw_content: impl FnOnce(&mut Renderer, Layout<'_>, mouse::Cursor, &Rectangle),
) where
    Renderer: iced_core::Renderer,
{
    let bounds = layout.bounds();
    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    draw_content(
        renderer,
        content_layout,
        cursor,
        &Rectangle {
            x: bounds.x,
            y: bounds.y,
            ..bounds
        },
    );
}

pub trait StyleSheet {
    type Style: Default;

    fn appearance(&self, style: &Self::Style) -> Appearance;
}

pub struct Appearance {}
