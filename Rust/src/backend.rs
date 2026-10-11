//! Macroquad implementation of the game's drawing contract.
mod assets;

use assets::Assets;
use ironjump::{
    drawing::{self, GAME_HEIGHT, GAME_WIDTH, Renderer, SpriteId, SpriteStyle},
    math::{Rect, Vec2},
    viewport::Viewport,
};
use macroquad::prelude as mq;

pub struct MacroquadRenderer {
    assets: Assets,
    target: mq::RenderTarget,
    camera: mq::Camera2D,
    logical_size: Vec2,
}
impl MacroquadRenderer {
    pub fn new() -> Self {
        let target = mq::render_target(GAME_WIDTH as u32, GAME_HEIGHT as u32);
        target.texture.set_filter(mq::FilterMode::Nearest);
        let mut camera =
            mq::Camera2D::from_display_rect(mq::Rect::new(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT));
        camera.render_target = Some(target.clone());
        Self {
            assets: Assets::new(),
            target,
            camera,
            logical_size: mq::vec2(GAME_WIDTH, GAME_HEIGHT),
        }
    }

    pub fn begin_frame(&mut self, view: Viewport) {
        if self.logical_size != view.logical_size {
            let size = view.logical_size.ceil();
            if self.target.texture.size() != size {
                self.target = mq::render_target(size.x as u32, size.y as u32);
                self.target.texture.set_filter(mq::FilterMode::Nearest);
            }
            self.camera = mq::Camera2D::from_display_rect(mq::Rect::new(
                0.0,
                0.0,
                view.logical_size.x,
                view.logical_size.y,
            ));
            self.camera.render_target = Some(self.target.clone());
            self.logical_size = view.logical_size;
        }
        mq::set_camera(&self.camera);
        mq::clear_background(mq::Color::from_rgba(101, 97, 85, 255));
    }

    pub fn present(&self, view: Viewport) {
        mq::set_default_camera();
        mq::clear_background(mq::BLACK);
        mq::draw_texture_ex(
            &self.target.texture,
            view.rect.x,
            view.rect.y,
            mq::WHITE,
            mq::DrawTextureParams {
                dest_size: Some(view.rect.size()),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    /// Platform-specific control hints, rasterized after presenting the game at
    /// window resolution to keep small text readable at any window scale.
    pub fn controls(&self, viewport: Viewport, text: &str) {
        let view = viewport.rect;
        let scale = viewport.scale();
        let height = 18.0 * scale;
        let top = view.bottom() - height;
        mq::draw_rectangle(
            view.x,
            top,
            view.w,
            height,
            mq::Color::new(0.0, 0.0, 0.0, 0.85),
        );
        let font_size = (14.0 * scale).round().max(1.0) as u16;
        let measured = mq::measure_text(text, None, font_size, 1.0);
        let font_scale = ((view.w - 24.0 * scale) / measured.width).min(1.0);
        let dimensions = mq::measure_text(text, None, font_size, font_scale);
        mq::draw_text_ex(
            text,
            view.x + (view.w - dimensions.width) * 0.5,
            top + (height - dimensions.height) * 0.5 + dimensions.offset_y,
            mq::TextParams {
                font_size,
                font_scale,
                color: mq::WHITE,
                ..Default::default()
            },
        );
    }

    fn sprite_asset(&self, sprite: SpriteId) -> &assets::Sprite {
        match sprite {
            SpriteId::Ball => &self.assets.ball,
            SpriteId::Diamond => &self.assets.diamond,
            SpriteId::Platform => &self.assets.platform,
            SpriteId::Background(frame) => &self.assets.background[frame],
            SpriteId::Movable => &self.assets.movable,
            SpriteId::Elevator(frame) => &self.assets.elevator[frame],
            SpriteId::Trampoline(frame) => &self.assets.trampoline[frame],
            SpriteId::Magnet => &self.assets.magnet,
            SpriteId::SpeedPowerUp => &self.assets.speed_power_up,
            SpriteId::SpeedEffect => &self.assets.speed_effect,
            SpriteId::Exit => &self.assets.exit,
            SpriteId::Menu => &self.assets.menu,
            SpriteId::LevelIcon(frame) => &self.assets.level_icons[frame],
            SpriteId::Arrow => &self.assets.arrow,
            SpriteId::Play => &self.assets.play,
            SpriteId::Win => &self.assets.win,
        }
    }
}
impl Renderer for MacroquadRenderer {
    fn sprite_ex(&mut self, sprite: SpriteId, position: Vec2, style: SpriteStyle) {
        self.sprite_asset(sprite).draw_ex(
            position,
            color(style.color),
            style.rotation,
            style.flip_x,
        );
    }
    fn bitmap_text(&mut self, text: &str, position: Vec2, tint: drawing::Color) {
        self.assets.text(text, position.x, position.y, color(tint));
    }
    fn centered_text(&mut self, text: &str, baseline: Vec2, size: u16, tint: drawing::Color) {
        let width = mq::measure_text(text, None, size, 1.0).width;
        mq::draw_text(
            text,
            baseline.x - width * 0.5,
            baseline.y,
            size as f32,
            color(tint),
        );
    }
    fn rectangle(&mut self, rect: Rect, tint: drawing::Color) {
        mq::draw_rectangle(rect.x, rect.y, rect.w, rect.h, color(tint));
    }
}
fn color(color: drawing::Color) -> mq::Color {
    mq::Color::new(color.r, color.g, color.b, color.a)
}
