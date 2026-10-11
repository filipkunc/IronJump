//! A small immediate-mode drawing contract. Implementations own GPU resources.
use crate::math::{Rect, Vec2};
pub(crate) mod translated;

/// Reference dimensions for the original menu and default game viewport.
pub const GAME_WIDTH: f32 = 480.0;
pub const GAME_HEIGHT: f32 = 320.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}
pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
pub const RED: Color = Color::new(0.9, 0.16, 0.22, 1.0);

/// Asset identities, never texture handles. Animation indices match the original atlas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteId {
    Ball,
    Diamond,
    Platform,
    Background(usize),
    Movable,
    Elevator(usize),
    Trampoline(usize),
    Magnet,
    SpeedPowerUp,
    SpeedEffect,
    Exit,
    Menu,
    LevelIcon(usize),
    Arrow,
    Play,
    Win,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteStyle {
    pub color: Color,
    /// Clockwise rotation in radians about the sprite's center.
    pub rotation: f32,
    pub flip_x: bool,
}
impl Default for SpriteStyle {
    fn default() -> Self {
        Self {
            color: WHITE,
            rotation: 0.0,
            flip_x: false,
        }
    }
}

/// Positions and sizes are logical game coordinates. Sprite dimensions and font
/// resources belong to the backend. Calls must preserve submission/blending order.
pub trait Renderer {
    /// Visible logical area, when known. Rendering may skip work outside it;
    /// simulation and collision updates must still include off-screen objects.
    fn viewport(&self) -> Option<Rect> {
        None
    }

    /// Draw a repeated sprite grid, visiting only tiles intersecting the viewport.
    fn tiles(&mut self, sprite: SpriteId, bounds: Rect, tile_size: Vec2) {
        let visible = match self.viewport() {
            Some(view) => match bounds.intersect(view) {
                Some(rect) if rect.w > 0.0 && rect.h > 0.0 => rect,
                _ => return,
            },
            None => bounds,
        };
        let first = ((visible.point() - bounds.point()) / tile_size)
            .floor()
            .max(Vec2::ZERO);
        let last = ((visible.point() + visible.size() - bounds.point()) / tile_size)
            .ceil()
            .min((bounds.size() / tile_size).ceil());
        for row in first.y as i32..last.y as i32 {
            for column in first.x as i32..last.x as i32 {
                self.sprite(
                    sprite,
                    bounds.point() + Vec2::new(column as f32, row as f32) * tile_size,
                );
            }
        }
    }

    fn sprite_ex(&mut self, sprite: SpriteId, position: Vec2, style: SpriteStyle);
    fn sprite(&mut self, sprite: SpriteId, position: Vec2) {
        self.sprite_ex(sprite, position, SpriteStyle::default());
    }
    /// Original bitmap font, positioned by the top-left of its 32px glyph cells.
    fn bitmap_text(&mut self, text: &str, position: Vec2, color: Color);
    /// Proportional/default font, centered horizontally at a baseline position.
    fn centered_text(&mut self, text: &str, baseline: Vec2, size: u16, color: Color);
    fn rectangle(&mut self, rect: Rect, color: Color);
}

/// A second renderer used by headless tests to inspect draw order and content.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub enum Command {
        Sprite(SpriteId, Vec2, SpriteStyle),
        BitmapText(String, Vec2, Color),
        CenteredText(String, Vec2, u16, Color),
        Rectangle(Rect, Color),
    }

    #[derive(Default)]
    pub struct RecordingRenderer {
        pub commands: Vec<Command>,
    }

    impl Renderer for RecordingRenderer {
        fn sprite_ex(&mut self, sprite: SpriteId, position: Vec2, style: SpriteStyle) {
            assert!(position.is_finite() && style.rotation.is_finite());
            self.commands.push(Command::Sprite(sprite, position, style));
        }
        fn bitmap_text(&mut self, text: &str, position: Vec2, color: Color) {
            self.commands
                .push(Command::BitmapText(text.into(), position, color));
        }
        fn centered_text(&mut self, text: &str, position: Vec2, size: u16, color: Color) {
            self.commands
                .push(Command::CenteredText(text.into(), position, size, color));
        }
        fn rectangle(&mut self, rect: Rect, color: Color) {
            self.commands.push(Command::Rectangle(rect, color));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{Command, RecordingRenderer};
    use super::*;
    struct Clipped(RecordingRenderer);
    impl Renderer for Clipped {
        fn viewport(&self) -> Option<Rect> {
            Some(Rect::new(0.0, 0.0, 64.0, 64.0))
        }
        fn sprite_ex(&mut self, id: SpriteId, position: Vec2, style: SpriteStyle) {
            self.0.sprite_ex(id, position, style);
        }
        fn bitmap_text(&mut self, _: &str, _: Vec2, _: Color) {
            unreachable!()
        }
        fn centered_text(&mut self, _: &str, _: Vec2, _: u16, _: Color) {
            unreachable!()
        }
        fn rectangle(&mut self, _: Rect, _: Color) {
            unreachable!()
        }
    }
    #[test]
    fn tiled_drawing_clips_large_objects_without_shifting_their_grid() {
        let mut renderer = Clipped(RecordingRenderer::default());
        renderer.tiles(
            SpriteId::Platform,
            Rect::new(-10000.0, -10000.0, 20000.0, 20000.0),
            Vec2::splat(32.0),
        );
        let positions: Vec<_> = renderer
            .0
            .commands
            .iter()
            .filter_map(|command| match command {
                Command::Sprite(_, position, _) => Some(*position),
                _ => None,
            })
            .collect();
        assert_eq!(positions.len(), 9);
        assert_eq!(positions[0], Vec2::splat(-16.0));
        assert_eq!(positions[8], Vec2::splat(48.0));
        renderer.0.commands.clear();
        renderer.tiles(
            SpriteId::Platform,
            Rect::new(64.0, 0.0, 3200.0, 3200.0),
            Vec2::splat(32.0),
        );
        assert!(renderer.0.commands.is_empty());
    }

    #[test]
    fn interpolated_tiles_are_culled_at_their_displayed_position() {
        let mut renderer = Clipped(RecordingRenderer::default());
        let mut translated = translated::Translated {
            renderer: &mut renderer,
            offset: Vec2::new(-48.0, 0.0),
        };
        assert_eq!(
            translated.viewport(),
            Some(Rect::new(48.0, 0.0, 64.0, 64.0))
        );
        translated.tiles(
            SpriteId::Platform,
            Rect::new(80.0, 0.0, 32.0, 32.0),
            Vec2::splat(32.0),
        );
        assert!(
            matches!(renderer.0.commands.as_slice(), [Command::Sprite(SpriteId::Platform, p, _)] if *p == Vec2::new(32.0, 0.0))
        );
    }
}
