use crate::{
    drawing::{GAME_HEIGHT, GAME_WIDTH},
    math::{Rect, Vec2, vec2},
};

/// Maps window coordinates to logical game units. Gameplay fills the window;
/// the original menu artwork uses a separate, fitted 480 × 320 layout.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub rect: Rect,
    pub logical_size: Vec2,
}
impl Viewport {
    /// Keep the shorter dimension at 320 game units. Wider/taller displays reveal
    /// more of the level without stretching sprites or changing the physics scale.
    pub fn new(window_size: Vec2) -> Self {
        let window_size = window_size.max(Vec2::ONE);
        let scale = window_size.min_element() / GAME_HEIGHT;
        Self {
            rect: Rect::new(0.0, 0.0, window_size.x, window_size.y),
            logical_size: window_size / scale,
        }
    }

    pub fn menu(window_size: Vec2) -> Self {
        let window_size = window_size.max(Vec2::ONE);
        let scale = (window_size.x / GAME_WIDTH).min(window_size.y / GAME_HEIGHT);
        let logical_size = vec2(GAME_WIDTH, GAME_HEIGHT);
        let size = logical_size * scale;
        let origin = (window_size - size) * 0.5;
        Self {
            rect: Rect::new(origin.x, origin.y, size.x, size.y),
            logical_size,
        }
    }

    pub fn scale(self) -> f32 {
        self.rect.h / self.logical_size.y
    }

    pub fn game_position(self, point: Vec2) -> Option<Vec2> {
        self.rect
            .contains(point)
            .then(|| (point - self.rect.point()) * (self.logical_size / self.rect.size()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_fills_different_aspect_ratios_with_uniform_sprite_scale() {
        for size in [
            vec2(960.0, 640.0),
            vec2(1920.0, 1080.0),
            vec2(2400.0, 1080.0),
            vec2(1080.0, 2400.0),
        ] {
            let view = Viewport::new(size);
            assert_eq!(view.rect.point(), Vec2::ZERO);
            assert_eq!(view.rect.size(), size);
            assert!((view.logical_size.min_element() - 320.0).abs() < 0.0001);
            assert!((view.rect.w / view.logical_size.x - view.scale()).abs() < 0.0001);
            assert!(
                (view.game_position(size * 0.5).unwrap() - view.logical_size * 0.5).length()
                    < 0.0001
            );
            assert!((view.game_position(size).unwrap() - view.logical_size).length() < 0.0001);
        }
        assert_eq!(
            Viewport::new(vec2(960.0, 640.0)).logical_size,
            vec2(480.0, 320.0)
        );
        assert!(Viewport::new(vec2(2400.0, 1080.0)).logical_size.x > 700.0);
    }

    #[test]
    fn menu_keeps_its_original_layout_and_excludes_letterbox_margins() {
        for size in [vec2(960.0, 640.0), vec2(1600.0, 900.0), vec2(600.0, 1000.0)] {
            let view = Viewport::menu(size);
            assert_eq!(view.game_position(view.rect.point()), Some(Vec2::ZERO));
            assert_eq!(
                view.game_position(view.rect.point() + view.rect.size()),
                Some(vec2(480.0, 320.0))
            );
            assert_eq!(view.game_position(size * 0.5), Some(vec2(240.0, 160.0)));
            assert_eq!(view.game_position(view.rect.point() - Vec2::ONE), None);
        }
    }

    #[test]
    fn minimized_window_has_a_finite_projection() {
        for size in [Vec2::ZERO, vec2(0.0, 100.0), vec2(100.0, 0.0)] {
            let view = Viewport::new(size);
            assert!(view.logical_size.is_finite());
            assert!(view.scale().is_finite() && view.scale() > 0.0);
        }
    }
}
