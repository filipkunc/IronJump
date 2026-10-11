use super::{
    collision::intersects,
    game_object::{GameObject, UpdateContext, WorldEffect},
    level_loader::parse_value,
    player::Player,
};
use crate::drawing::{Renderer, SpriteId};
use crate::math::*;

pub(super) struct Magnet {
    position: Vec2,
    width_segments: i32,
}

impl Magnet {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            width_segments: parse_value(node, "widthSegments"),
        }
    }
}

impl GameObject for Magnet {
    fn update(&mut self, _context: &UpdateContext<'_>) {}
    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        let rect = self.rect();
        let attraction = Rect::new(rect.x + 18.0, rect.y + 32.0, rect.w - 36.0, rect.h + 128.0);
        if intersects(attraction, player.rect()) && player.movement.y < 5.0 {
            player.movement.y = player.movement.y * 0.7 + 5.0 * 0.3;
        }
        None
    }
    fn draw(&self, renderer: &mut dyn Renderer) {
        renderer.tiles(SpriteId::Magnet, self.rect(), vec2(32.0, 32.0));
    }

    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width_segments as f32 * 32.0,
            32.0,
        )
    }
    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }
    fn is_platform(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnet_attracts_only_in_its_field_and_below_target_speed() {
        let doc = roxmltree::Document::parse(
            "<FPMagnet><x>0</x><y>0</y><widthSegments>2</widthSegments></FPMagnet>",
        )
        .unwrap();
        let mut magnet = Magnet::parse(doc.root_element());
        let mut player = Player::new();
        player.position = vec2(20.0, 64.0);
        player.movement.y = -5.0;
        magnet.interact(&mut player);
        assert_eq!(player.movement.y, -2.0);
        player.movement.y = 6.0;
        magnet.interact(&mut player);
        assert_eq!(player.movement.y, 6.0);
        player.position = vec2(80.0, 64.0);
        player.movement.y = -5.0;
        magnet.interact(&mut player);
        assert_eq!(player.movement.y, -5.0);
    }
}
