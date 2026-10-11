use super::{
    collision::intersects,
    game_object::{GameObject, UpdateContext, WorldEffect},
    level_loader::parse_value,
    player::Player,
};
use crate::drawing::{Renderer, SpriteId};
use crate::math::*;

pub(super) struct Trampoline {
    position: Vec2,
    width_segments: i32,
    animation_counter: u8,
    texture_index: i32,
    texture_direction: i32,
}

impl Trampoline {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            width_segments: parse_value(node, "widthSegments"),
            animation_counter: 0,
            texture_index: 0,
            texture_direction: 1,
        }
    }
}

impl GameObject for Trampoline {
    fn update(&mut self, _context: &UpdateContext<'_>) {
        self.animation_counter += 1;
        if self.animation_counter > 5 {
            self.texture_index += self.texture_direction;
            if self.texture_index < 0 || self.texture_index >= 2 {
                self.texture_index -= self.texture_direction;
                self.texture_direction = -self.texture_direction;
            }
            self.animation_counter = 0;
        }
    }
    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        let mut rect = player.rect();
        if !intersects(rect, self.rect()) {
            rect.h += Player::TOLERANCE;
            if intersects(rect, self.rect()) {
                player.movement.y = 9.5;
            }
        }
        Some(WorldEffect::BounceMovables {
            surface: self.rect(),
            speed: 8.0,
        })
    }
    fn draw(&self, renderer: &mut dyn Renderer) {
        for segment in 0..self.width_segments {
            renderer.sprite(
                SpriteId::Trampoline(self.texture_index as usize),
                vec2(self.position.x + segment as f32 * 64.0, self.position.y),
            );
        }
    }
    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width_segments as f32 * 64.0,
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
    use super::super::test_support::game_with;
    use super::*;

    #[test]
    fn trampoline_launches_player_from_above_but_not_from_inside() {
        let mut game = game_with(
            "<FPTrampoline><x>0</x><y>32</y><widthSegments>2</widthSegments></FPTrampoline>",
        );
        assert_eq!(game.world.game_objects[0].rect().w, 128.0);
        game.world.update(&mut game.player);
        assert_eq!(game.player.movement.y, 9.5);
        game.player.position.y += 4.0;
        game.player.movement.y = -2.0;
        game.world.update(&mut game.player);
        assert_eq!(game.player.movement.y, -2.0);
    }

    #[test]
    fn trampoline_requires_more_than_30_pixels_of_movable_overlap() {
        let mut game = game_with(
            "<FPMovablePlatform><x>66</x><y>0</y><widthSegments>1</widthSegments><heightSegments>1</heightSegments></FPMovablePlatform><FPTrampoline><x>0</x><y>32</y><widthSegments>2</widthSegments></FPTrampoline>",
        );
        // Trampoline ends at x=128; position the crate at x=98 for exactly 30px.
        game.world.game_objects[0].move_world(vec2(32.0, 0.0));
        let original_y = game.world.game_objects[0].rect().y;
        game.world.update(&mut game.player);
        assert!((game.world.game_objects[0].rect().y - original_y).abs() < 0.0001);
    }
}
