use super::{
    collision::intersects,
    game_object::{GameObject, UpdateContext, WorldEffect},
    level_loader::parse_value,
    player::Player,
};
use crate::assets::Assets;
use macroquad::prelude::*;

pub(super) struct SpeedPowerUp {
    position: Vec2,
    visible: bool,
    cooldown: u32,
}

impl SpeedPowerUp {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            visible: true,
            cooldown: 0,
        }
    }
}

impl GameObject for SpeedPowerUp {
    fn update(&mut self, _context: &UpdateContext<'_>) {}

    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        if self.cooldown > 0 {
            self.cooldown += 1;
            if self.cooldown > Player::MAX_SPEED_UP_COUNT {
                self.cooldown = 0;
                self.visible = true;
            }
        } else if self.visible && intersects(self.rect(), player.rect()) {
            self.visible = false;
            self.cooldown = 1;
            player.speed_up_counter = 1;
        }
        None
    }

    fn draw(&self, assets: &Assets) {
        assets.speed_power_up.draw(self.position.x, self.position.y);
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }

    fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 32.0, 32.0)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

#[cfg(test)]
mod tests {
    use super::super::{test_support::game_with, world::World};
    use super::*;

    #[test]
    fn speed_pickup_expires_and_respawns_after_360_ticks() {
        let mut game = game_with("<FPSpeedPowerUp><x>0</x><y>0</y></FPSpeedPowerUp>");
        game.world.update(&mut game.player);
        assert_eq!(game.player.speed_up_counter, 1);
        assert!(!game.world.game_objects[0].is_visible());
        // Move off the pickup so expiry cannot immediately re-collect it.
        game.player.position.x += 100.0;
        for _ in 0..359 {
            game.world.update(&mut game.player);
        }
        assert!(!game.world.game_objects[0].is_visible());
        game.world.update(&mut game.player);
        assert!(game.world.game_objects[0].is_visible());

        game.player.speed_up_counter = 1;
        for _ in 0..359 {
            game.player.update(&mut World::new(), Vec2::ZERO);
        }
        assert_eq!(game.player.speed_up_counter, 360);
        game.player.update(&mut World::new(), Vec2::ZERO);
        assert_eq!(game.player.speed_up_counter, 0);
    }
}
