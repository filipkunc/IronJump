//! Geometry shared by simulation and drawing, independent of the GPU backend.
pub use glam::{Vec2, vec2};

/// Screen-space rectangle: x increases rightward and y downward.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    pub fn point(self) -> Vec2 {
        vec2(self.x, self.y)
    }
    pub fn size(self) -> Vec2 {
        vec2(self.w, self.h)
    }
    pub fn left(self) -> f32 {
        self.x
    }
    pub fn right(self) -> f32 {
        self.x + self.w
    }
    pub fn top(self) -> f32 {
        self.y
    }
    pub fn bottom(self) -> f32 {
        self.y + self.h
    }

    // Inclusive boundaries preserve the original menu hit testing.
    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    /// Includes zero-area edge contact; gameplay collision helpers apply their
    /// own overlap tolerance after this geometric intersection.
    pub fn intersect(self, other: Self) -> Option<Self> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right < left || bottom < top {
            None
        } else {
            Some(Self::new(left, top, right - left, bottom - top))
        }
    }

    pub fn combine_with(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self::new(
            x,
            y,
            self.right().max(other.right()) - x,
            self.bottom().max(other.bottom()) - y,
        )
    }

    pub fn offset(self, offset: Vec2) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
    }

    pub fn normalized(self) -> Self {
        Self::new(
            self.x.min(self.right()),
            self.y.min(self.bottom()),
            self.w.abs(),
            self.h.abs(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_contact_is_distinct_from_disjoint_rectangles() {
        let rect = Rect::new(0.0, 0.0, 32.0, 32.0);
        assert_eq!(
            rect.intersect(rect.offset(vec2(32.0, 0.0))),
            Some(Rect::new(32.0, 0.0, 0.0, 32.0))
        );
        assert_eq!(rect.intersect(rect.offset(vec2(33.0, 0.0))), None);
        assert!(rect.contains(vec2(32.0, 32.0)));
    }

    #[test]
    fn swept_bounds_cover_both_positions_and_normalization_preserves_area() {
        let rect = Rect::new(10.0, 20.0, 32.0, 32.0);
        assert_eq!(
            rect.combine_with(rect.offset(vec2(-5.0, 15.0))),
            Rect::new(5.0, 20.0, 37.0, 47.0)
        );
        assert_eq!(Rect::new(42.0, 52.0, -32.0, -32.0).normalized(), rect);
    }
}
