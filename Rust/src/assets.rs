use macroquad::prelude::*;

/// A region of the original iPhone texture atlas (or a standalone UI texture).
pub struct Sprite {
    texture: Texture2D,
    source: Rect,
}

impl Sprite {
    fn region(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            texture: texture.clone(),
            source: Rect::new(x, y, w, h),
        }
    }

    fn image(bytes: &[u8]) -> Self {
        let texture = texture(bytes);
        Self::region(&texture, 0.0, 0.0, texture.width(), texture.height())
    }

    pub fn draw(&self, x: f32, y: f32) {
        self.draw_ex(vec2(x, y), WHITE, 0.0, false);
    }

    pub fn draw_ex(&self, position: Vec2, color: Color, rotation: f32, flip_x: bool) {
        if position.x > 480.0
            || position.y > 320.0
            || position.x + self.source.w < 0.0
            || position.y + self.source.h < 0.0
        {
            return;
        }
        draw_texture_ex(
            &self.texture,
            position.x,
            position.y,
            color,
            DrawTextureParams {
                source: Some(self.source),
                rotation,
                flip_x,
                ..Default::default()
            },
        );
    }
}

/// GPU resources, loaded once and borrowed when drawing.
pub struct Assets {
    pub ball: Sprite,
    pub diamond: Sprite,
    pub platform: Sprite,
    pub background: [Sprite; 2],
    pub movable: Sprite,
    pub elevator: [Sprite; 3],
    pub trampoline: [Sprite; 3],
    pub magnet: Sprite,
    pub speed_power_up: Sprite,
    pub speed_effect: Sprite,
    pub exit: Sprite,
    pub menu: Sprite,
    pub level_icons: [Sprite; 3],
    pub arrow: Sprite,
    pub play: Sprite,
    pub win: Sprite,
    font: Texture2D,
}

impl Assets {
    pub fn new() -> Self {
        let atlas = texture(include_bytes!("../../IronJump/Images/levelatlas.png"));
        let tile = |x, y, w, h| Sprite::region(&atlas, x, y, w, h);
        let mut font_image = Image::from_file_with_format(
            include_bytes!("../../IronJump/Images/AndaleMono.png"),
            Some(ImageFormat::Png),
        )
        .unwrap();
        // FPFont uses the red channel as alpha, with white RGB for tinting.
        for pixel in font_image.bytes.chunks_exact_mut(4) {
            let alpha = pixel[0];
            pixel.copy_from_slice(&[255, 255, 255, alpha]);
        }
        let font = Texture2D::from_image(&font_image);
        font.set_filter(FilterMode::Nearest);
        Self {
            ball: tile(1.0, 1.0, 32.0, 32.0),
            diamond: tile(34.0, 1.0, 32.0, 32.0),
            platform: tile(133.0, 1.0, 32.0, 32.0),
            background: [tile(67.0, 1.0, 32.0, 32.0), tile(34.0, 34.0, 32.0, 32.0)],
            movable: tile(100.0, 1.0, 32.0, 32.0),
            elevator: [
                tile(361.0, 1.0, 32.0, 32.0),
                tile(394.0, 1.0, 32.0, 32.0),
                tile(427.0, 1.0, 32.0, 32.0),
            ],
            trampoline: [
                tile(166.0, 1.0, 64.0, 32.0),
                tile(231.0, 1.0, 64.0, 32.0),
                tile(296.0, 1.0, 64.0, 32.0),
            ],
            magnet: tile(460.0, 1.0, 32.0, 32.0),
            speed_power_up: tile(1.0, 34.0, 32.0, 32.0),
            exit: tile(67.0, 34.0, 64.0, 64.0),
            speed_effect: tile(132.0, 34.0, 64.0, 64.0),
            menu: Sprite::image(include_bytes!("../../IronJump/Images/menu.png")),
            level_icons: [
                Sprite::image(include_bytes!("../../IronJump/Images/Level_black.png")),
                Sprite::image(include_bytes!("../../IronJump/Images/Level_blue.png")),
                Sprite::image(include_bytes!("../../IronJump/Images/Level_shine.png")),
            ],
            arrow: Sprite::image(include_bytes!("../../IronJump/Images/arrow.png")),
            play: Sprite::image(include_bytes!("../../IronJump/Images/play.png")),
            win: Sprite::image(include_bytes!("../../IronJump/Images/win.png")),
            font,
        }
    }

    pub fn text(&self, text: &str, x: f32, y: f32, color: Color) {
        for (index, character) in text.chars().enumerate() {
            let code = if character.is_ascii() {
                character as u32
            } else {
                b'?' as u32
            };
            if code == b' ' as u32 {
                continue;
            }
            draw_texture_ex(
                &self.font,
                x + index as f32 * 13.0,
                y,
                color,
                DrawTextureParams {
                    source: Some(Rect::new(
                        (code % 16) as f32 * 32.0,
                        (code / 16) as f32 * 32.0,
                        32.0,
                        32.0,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}

fn texture(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    texture.set_filter(FilterMode::Nearest);
    texture
}
