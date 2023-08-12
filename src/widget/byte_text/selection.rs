use iced::{Point, Rectangle, Vector};

#[derive(Debug, Clone, Copy)]
pub struct Raw {
    pub start: Point,
    pub end: Point,
}

impl Raw {
    pub fn resolve(&self, bounds: Rectangle) -> Option<Resolved> {
        if f32::max(f32::min(self.start.y, self.end.y), bounds.y)
            <= f32::min(f32::max(self.start.y, self.end.y), bounds.y + bounds.height)
        {
            let (mut start, mut end) = if self.start.y < self.end.y
                || self.start.y == self.end.y && self.start.x < self.end.x
            {
                (self.start, self.end)
            } else {
                (self.end, self.start)
            };

            let clip = |p: Point| Point {
                x: p.x.max(bounds.x).min(bounds.x + bounds.width),
                y: p.y.max(bounds.y).min(bounds.y + bounds.height),
            };

            if start.y < bounds.y {
                start = bounds.position();
            } else {
                start = clip(start);
            }

            if end.y > bounds.y + bounds.height {
                end = bounds.position() + Vector::from(bounds.size());
            } else {
                end = clip(end);
            }

            ((start.x - end.x).abs() > 1.0).then_some(Resolved { start, end })
            //Some(Resolved { start, end })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Resolved {
    pub start: Point,
    pub end: Point,
}
