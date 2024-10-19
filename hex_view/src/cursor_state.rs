use egui::Context;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorState {
    Hovering,
    Pressed,
    StillDown,
    Released,
}

impl CursorState {
    pub fn get(ctx: &Context) -> Self {
        ctx.input(|i| {
            if i.pointer.primary_pressed() {
                Self::Pressed
            } else if i.pointer.primary_down() {
                Self::StillDown
            } else if i.pointer.primary_released() {
                Self::Released
            } else {
                Self::Hovering
            }
        })
    }
}