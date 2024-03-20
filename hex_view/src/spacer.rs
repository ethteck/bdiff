use egui::{
    epaint::{vec2, Vec2},
    {Response, Sense, Ui, Widget},
};

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct Spacer {
    spacing: Vec2,
}

impl Default for Spacer {
    fn default() -> Self {
        Self {
            spacing: vec2(4.0, 0.0),
        }
    }
}

impl Spacer {
    /// Set the x spacing.
    ///
    pub fn spacing_x(mut self, space: f32) -> Self {
        self.spacing.x = space;
        self
    }
}

impl Widget for Spacer {
    fn ui(self, ui: &mut Ui) -> Response {
        let Spacer { spacing } = self;

        let size = spacing;

        let (_rect, response) = ui.allocate_at_least(size, Sense::hover());

        response
    }
}
