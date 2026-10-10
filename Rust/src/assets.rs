use std::str::FromStr;

use macroquad::prelude::*;

/// GPU resources, loaded once and borrowed when drawing.
pub struct Assets {
    pub ball: Texture2D,
    pub diamond: Texture2D,
    pub platform: Texture2D,
    pub background: Texture2D,
    pub movable: Texture2D,
    pub level: String
}

impl Assets {
    pub fn new() -> Self {
        Self {
            ball: texture(include_bytes!("../../Canvas/Images/ball.png")),
            diamond: texture(include_bytes!("../../Canvas/Images/diamond.png")),
            platform: texture(include_bytes!("../../Canvas/Images/plos_marble.png")),
            background: texture(include_bytes!("../../Canvas/Images/marbleblue.png")),
            movable: texture(include_bytes!("../../Canvas/Images/movable.png")),
            level: String::from(include_str!("../../Canvas/Levels/jstest.xlevel")),
        }
    }
}

fn texture(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    texture.set_filter(FilterMode::Nearest);
    texture
}
