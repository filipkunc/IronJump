use macroquad::prelude::*;

/// A region of the original iPhone texture atlas (or a standalone UI texture).
pub struct Sprite {
    texture: Texture2D,
    source: Rect,
    smooth: bool,
    texture_size: Vec2,
}

impl Sprite {
    fn region(texture: &Texture2D, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            texture: texture.clone(),
            source: Rect::new(x, y, w, h),
            smooth: false,
            texture_size: texture.size(),
        }
    }

    fn image(bytes: &[u8]) -> Self {
        let texture = texture(bytes);
        Self::region(&texture, 0.0, 0.0, texture.width(), texture.height())
    }

    pub fn draw_ex(
        &self,
        position: Vec2,
        color: Color,
        rotation: f32,
        flip_x: bool,
        viewport: Vec2,
    ) {
        let size = self.source.size();
        if !visible(position, size, rotation, viewport) {
            return;
        }
        let pivot = position + size * 0.5;
        let (sin, cos) = rotation.sin_cos();
        let corners = [Vec2::ZERO, vec2(size.x, 0.0), size, vec2(0.0, size.y)];
        let mut vertices = [Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, color); 4];
        for (vertex, corner) in vertices.iter_mut().zip(corners) {
            let offset = corner - size * 0.5;
            let point = pivot
                + vec2(
                    offset.x * cos - offset.y * sin,
                    offset.x * sin + offset.y * cos,
                );
            let uv_x = if flip_x { size.x - corner.x } else { corner.x };
            *vertex = Vertex::new(
                point.x,
                point.y,
                0.0,
                (self.source.x + uv_x) / self.texture_size.x,
                (self.source.y + corner.y) / self.texture_size.y,
                color,
            );
            vertex.normal = vec4(
                f32::from(self.smooth),
                self.texture_size.x,
                self.texture_size.y,
                0.0,
            );
        }
        // Submit geometry without allocating a Mesh per tile. Macroquad combines
        // consecutive submissions sharing the atlas and pipeline into one GPU draw.
        unsafe {
            let gl = macroquad::window::get_internal_gl().quad_gl;
            gl.texture(Some(&self.texture));
            gl.draw_mode(DrawMode::Triangles);
            gl.geometry(&vertices, &[0, 1, 2, 0, 2, 3]);
        }
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
        let mut atlas_image = Image::gen_image_color(512, 1024, Color::new(0.0, 0.0, 0.0, 0.0));
        let tiles = Image::from_file_with_format(
            include_bytes!("../../../IronJump/Images/levelatlas.png"),
            Some(ImageFormat::Png),
        )
        .unwrap();
        paste(&mut atlas_image, &tiles, 0, 0);
        let mut font_image = Image::from_file_with_format(
            include_bytes!("../../../IronJump/Images/AndaleMono.png"),
            Some(ImageFormat::Png),
        )
        .unwrap();
        // FPFont uses the red channel as alpha, with white RGB for tinting.
        for pixel in font_image.bytes.chunks_exact_mut(4) {
            let alpha = pixel[0];
            pixel.copy_from_slice(&[255, 255, 255, alpha]);
        }
        paste(&mut atlas_image, &font_image, 0, 128);
        let ball_image = Image::from_file_with_format(
            include_bytes!("../../../IronJump/Images/atlas/ball.png"),
            Some(ImageFormat::Png),
        )
        .unwrap();
        // Extrude the ball's edge into a one-pixel border for bilinear sampling.
        for y in 0_u32..34 {
            for x in 0_u32..34 {
                atlas_image.set_pixel(
                    x,
                    640 + y,
                    ball_image.get_pixel(x.saturating_sub(1).min(31), y.saturating_sub(1).min(31)),
                );
            }
        }
        let atlas = Texture2D::from_image(&atlas_image);
        atlas.set_filter(FilterMode::Linear);
        let tile = |x, y, w, h| Sprite::region(&atlas, x, y, w, h);
        let mut ball = tile(1.0, 641.0, 32.0, 32.0);
        ball.smooth = true;
        let font = atlas.clone();
        Self {
            ball,
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
            menu: Sprite::image(include_bytes!("../../../IronJump/Images/menu.png")),
            level_icons: [
                Sprite::image(include_bytes!("../../../IronJump/Images/Level_black.png")),
                Sprite::image(include_bytes!("../../../IronJump/Images/Level_blue.png")),
                Sprite::image(include_bytes!("../../../IronJump/Images/Level_shine.png")),
            ],
            arrow: Sprite::image(include_bytes!("../../../IronJump/Images/arrow.png")),
            play: Sprite::image(include_bytes!("../../../IronJump/Images/play.png")),
            win: Sprite::image(include_bytes!("../../../IronJump/Images/win.png")),
            font,
        }
    }

    pub fn text(&self, text: &str, x: f32, y: f32, color: Color, viewport: Vec2) {
        for (index, character) in text.chars().enumerate() {
            let code = if character.is_ascii() {
                character as u32
            } else {
                b'?' as u32
            };
            if code == b' ' as u32 {
                continue;
            }
            Sprite::region(
                &self.font,
                (code % 16) as f32 * 32.0,
                128.0 + (code / 16) as f32 * 32.0,
                32.0,
                32.0,
            )
            .draw_ex(
                vec2(x + index as f32 * 13.0, y),
                color,
                0.0,
                false,
                viewport,
            );
        }
    }
}

fn texture(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn paste(target: &mut Image, source: &Image, x: usize, y: usize) {
    for row in 0..source.height as usize {
        let from = row * source.width as usize * 4;
        let to = ((row + y) * target.width as usize + x) * 4;
        target.bytes[to..to + source.width as usize * 4]
            .copy_from_slice(&source.bytes[from..from + source.width as usize * 4]);
    }
}

fn visible(position: Vec2, size: Vec2, rotation: f32, viewport: Vec2) -> bool {
    let (sin, cos) = rotation.sin_cos();
    let extent = vec2(
        size.x * cos.abs() + size.y * sin.abs(),
        size.x * sin.abs() + size.y * cos.abs(),
    ) * 0.5;
    let center = position + size * 0.5;
    (center + extent).cmpgt(Vec2::ZERO).all() && (center - extent).cmplt(viewport).all()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn culling_keeps_rotated_corners_and_uses_current_viewport() {
        let viewport = vec2(700.0, 320.0);
        assert!(visible(vec2(600.0, 10.0), Vec2::splat(32.0), 0.0, viewport));
        assert!(!visible(
            vec2(700.0, 10.0),
            Vec2::splat(32.0),
            0.0,
            viewport
        ));
        assert!(!visible(
            vec2(-34.0, 10.0),
            Vec2::splat(32.0),
            0.0,
            viewport
        ));
        assert!(visible(
            vec2(-34.0, 10.0),
            Vec2::splat(32.0),
            std::f32::consts::FRAC_PI_4,
            viewport
        ));
    }
}
