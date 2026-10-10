use crate::assets::Assets;
use macroquad::prelude::*;

const SPRITE_SIZE: f32 = 64.0;
const SPEED: f32 = 240.0; // Screen units per second.

/// Rendering/input demonstration only; replace this with your own game state.
pub struct Demo {
    position: Vec2,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            position: vec2(80.0, 220.0),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut direction = Vec2::ZERO;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            direction.x += 1.0;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            direction.y += 1.0;
        }
        self.position += direction.normalize_or_zero() * SPEED * dt;

        // Recompute bounds each frame so resizing also keeps the sprite visible.
        self.position.x = self
            .position
            .x
            .clamp(0.0, (screen_width() - SPRITE_SIZE).max(0.0));
        self.position.y = self
            .position
            .y
            .clamp(0.0, (screen_height() - SPRITE_SIZE).max(0.0));
    }

    pub fn draw(&self, assets: &Assets) {
        // Coordinates start at the top left; x goes right and y goes down.
        // Draw order is back to front. PNG transparency is blended automatically.
        draw_texture(&assets.diamond, 240.0, 220.0, WHITE);
        draw_texture_ex(
            &assets.ball,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(32.0, 32.0)),
                rotation: 30.0,
                ..Default::default()
            },
        );

        // Macroquad supplies a default font. Text y is its baseline.
        draw_text("IronJump / Rust", 24.0, 48.0, 32.0, WHITE);
        draw_text(
            "Arrows or WASD: move the image",
            24.0,
            82.0,
            22.0,
            LIGHTGRAY,
        );
        draw_text("Esc: quit", 24.0, 110.0, 22.0, LIGHTGRAY);
        draw_text(
            format!(
                "{:.0}, {:.0}  |  {} FPS",
                self.position.x,
                self.position.y,
                get_fps()
            ),
            24.0,
            screen_height() - 24.0,
            20.0,
            LIGHTGRAY,
        );
    }
}
