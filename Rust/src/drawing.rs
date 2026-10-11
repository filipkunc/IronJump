//! A small immediate-mode drawing contract. Implementations own GPU resources.
use crate::math::{Rect, Vec2};

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
