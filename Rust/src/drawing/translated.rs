//! Presentation-only translation, including the visibility query used by tiles.
use super::{Color, Renderer, SpriteId, SpriteStyle};
use crate::math::{Rect, Vec2};

pub(crate) struct Translated<'a> {
    pub renderer: &'a mut dyn Renderer,
    pub offset: Vec2,
}
impl Renderer for Translated<'_> {
    fn viewport(&self) -> Option<Rect> {
        self.renderer
            .viewport()
            .map(|rect| rect.offset(-self.offset))
    }
    fn tiles(&mut self, sprite: SpriteId, bounds: Rect, tile_size: Vec2) {
        self.renderer
            .tiles(sprite, bounds.offset(self.offset), tile_size);
    }
    fn sprite_ex(&mut self, sprite: SpriteId, position: Vec2, style: SpriteStyle) {
        self.renderer
            .sprite_ex(sprite, position + self.offset, style);
    }
    fn bitmap_text(&mut self, text: &str, position: Vec2, color: Color) {
        self.renderer
            .bitmap_text(text, position + self.offset, color);
    }
    fn centered_text(&mut self, text: &str, position: Vec2, size: u16, color: Color) {
        self.renderer
            .centered_text(text, position + self.offset, size, color);
    }
    fn rectangle(&mut self, rect: Rect, color: Color) {
        self.renderer.rectangle(rect.offset(self.offset), color);
    }
}
