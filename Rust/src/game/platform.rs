use super::{
    game_object::{GameObject, UpdateContext},
    level_loader::parse_value,
};
use crate::drawing::{Renderer, SpriteId};
use crate::math::*;

pub struct Platform {
    position: Vec2,
    width_segments: i32,
    height_segments: i32,
}

impl Platform {
    pub fn parse(node: roxmltree::Node) -> Self {
        Self {
            position: Vec2 {
                x: parse_value(node, "x"),
                y: parse_value(node, "y"),
            },
            width_segments: parse_value(node, "widthSegments"),
            height_segments: parse_value(node, "heightSegments"),
        }
    }
}

impl GameObject for Platform {
    fn is_transparent(&self) -> bool {
        false
    }
    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width_segments as f32 * 32.0,
            self.height_segments as f32 * 32.0,
        )
    }

    fn is_platform(&self) -> bool {
        true
    }

    fn update(&mut self, _context: &UpdateContext<'_>) {}

    fn draw(&self, renderer: &mut dyn Renderer) {
        for iy in 0..self.height_segments {
            for ix in 0..self.width_segments {
                renderer.sprite(
                    SpriteId::Platform,
                    vec2(
                        self.position.x + (ix as f32) * 32.0,
                        self.position.y + (iy as f32) * 32.0,
                    ),
                );
            }
        }
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_bounds_match_its_tile_dimensions() {
        let platform = Platform {
            position: vec2(10.0, 20.0),
            width_segments: 3,
            height_segments: 2,
        };
        assert_eq!(platform.rect(), Rect::new(10.0, 20.0, 96.0, 64.0));
        assert!(platform.is_platform());
        assert!(!platform.is_movable());
    }
}
