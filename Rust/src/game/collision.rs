use macroquad::prelude::Rect;

// Matches rectTolerance in SharedClasses/FPGraphics.m, not the landing tolerance.
const RECT_TOLERANCE: f32 = 0.1;

pub(super) fn collision_intersection(a: Rect, b: Rect) -> Option<Rect> {
    a.intersect(b)
        .filter(|intersection| intersection.w >= RECT_TOLERANCE && intersection.h >= RECT_TOLERANCE)
}

pub(super) fn intersects(a: Rect, b: Rect) -> bool {
    a.normalized()
        .intersect(b.normalized())
        .is_some_and(|r| r.w > 0.0 && r.h > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_filters_touching_and_sub_tolerance_overlaps() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        for other in [
            Rect::new(11.0, 0.0, 10.0, 10.0),
            Rect::new(10.0, 0.0, 10.0, 10.0),
            Rect::new(9.95, 0.0, 10.0, 10.0),
            Rect::new(0.0, 9.95, 10.0, 10.0),
        ] {
            assert_eq!(collision_intersection(rect, other), None);
        }
        let minimum = Rect::new(0.0, 0.0, RECT_TOLERANCE, RECT_TOLERANCE);
        assert_eq!(collision_intersection(rect, minimum), Some(minimum));
        assert_eq!(
            collision_intersection(rect, Rect::new(8.0, 7.0, 10.0, 10.0)),
            Some(Rect::new(8.0, 7.0, 2.0, 3.0)),
        );
    }
}
