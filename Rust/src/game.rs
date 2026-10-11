use crate::assets::Assets;
use macroquad::prelude::*;

mod collision;
mod diamond;
mod elevator;
mod exit;
mod game_object;
mod level_loader;
mod magnet;
mod movable_platform;
mod platform;
mod player;
mod speed_power_up;
mod trampoline;
mod world;

use player::Player;
use world::World;

pub struct Game {
    world: World,
    player: Player,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            player: Player::new(),
        }
    }

    pub fn load(&mut self, xml: &str) -> Result<(), String> {
        *self = level_loader::load(xml)?;
        Ok(())
    }

    pub fn update(&mut self, input: Vec2) {
        self.world.update(&mut self.player);
        self.player.update(&mut self.world, input);
    }

    pub fn draw(&self, assets: &Assets, custom: bool) {
        self.world.draw(assets, custom);
        self.player.draw(assets);
        let (picked, total) = self.diamonds();
        assets.text(
            &format!("Diamonds: {picked}/{total}"),
            3.0,
            0.0,
            Color::new(1.0, 1.0, 1.0, 0.8),
        );
        if self.player.speed_up_counter > 0 {
            let seconds = (Player::MAX_SPEED_UP_COUNT - self.player.speed_up_counter) as f32 / 60.0;
            assets.text(
                &format!("{seconds:.1}"),
                430.0,
                285.0,
                Color::new(0.5, 1.0, 1.0, 0.8),
            );
        }
    }

    pub fn diamonds(&self) -> (usize, usize) {
        let total = self
            .world
            .game_objects
            .iter()
            .filter(|o| o.is_diamond())
            .count();
        let picked = self
            .world
            .game_objects
            .iter()
            .filter(|o| o.is_diamond() && !o.is_visible())
            .count();
        (picked, total)
    }

    pub fn is_complete(&self) -> bool {
        self.world
            .game_objects
            .iter()
            .any(|o| o.is_exit() && !o.is_visible())
    }

    pub fn has_fallen(&self) -> bool {
        !self.world.game_objects.iter().any(|o| {
            o.is_platform() && !o.is_movable() && self.player.rect().top() < o.rect().bottom()
        })
    }
}

#[cfg(test)]
mod test_support;
