use crate::drawing::{Color, GAME_HEIGHT, GAME_WIDTH, Renderer};
use crate::input::GameInput;
use crate::math::*;

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
    viewport_size: Vec2,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            player: Player::new(),
            viewport_size: vec2(GAME_WIDTH, GAME_HEIGHT),
        }
    }

    pub fn with_viewport(size: Vec2) -> Self {
        let mut game = Self::new();
        game.resize(size);
        game
    }

    pub fn viewport_size(&self) -> Vec2 {
        self.viewport_size
    }

    /// Recenter the camera without changing any object's position relative to
    /// the player, velocity, pickup state, or elevator route.
    pub fn resize(&mut self, size: Vec2) {
        if !size.is_finite() || size.min_element() <= 0.0 || size == self.viewport_size {
            return;
        }
        let offset = (size - self.viewport_size) * 0.5;
        self.player.position += offset;
        self.world.move_world(offset);
        self.viewport_size = size;
    }

    pub fn load(&mut self, xml: &str) -> Result<(), String> {
        *self = level_loader::load(xml, self.viewport_size)?;
        Ok(())
    }

    pub fn update(&mut self, input: GameInput) {
        self.world.update(&mut self.player);
        self.player.update(&mut self.world, input);
    }

    pub fn draw(&self, renderer: &mut dyn Renderer, custom: bool) {
        self.world.draw(renderer, custom, self.viewport_size);
        self.player.draw(renderer);
        let (picked, total) = self.diamonds();
        renderer.bitmap_text(
            &format!("Diamonds: {picked}/{total}"),
            vec2(3.0, 0.0),
            Color::new(1.0, 1.0, 1.0, 0.8),
        );
        if self.player.speed_up_counter > 0 {
            let seconds = (Player::MAX_SPEED_UP_COUNT - self.player.speed_up_counter) as f32 / 60.0;
            renderer.bitmap_text(
                &format!("{seconds:.1}"),
                self.viewport_size - vec2(50.0, 35.0),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawing::{
        SpriteId, WHITE,
        test_support::{Command, RecordingRenderer},
    };

    #[test]
    fn resizing_preserves_live_state_and_relative_object_positions() {
        let mut game = test_support::game_with(
            "<FPDiamond><x>0</x><y>0</y></FPDiamond><FPPlatform><x>0</x><y>32</y><widthSegments>8</widthSegments><heightSegments>1</heightSegments></FPPlatform>",
        );
        game.update(GameInput::new(1.0, false));
        game.player.speed_up_counter = 42;
        let movement = game.player.movement;
        let relative: Vec<_> = game
            .world
            .game_objects
            .iter()
            .map(|object| object.rect().offset(-game.player.position))
            .collect();
        for size in [vec2(712.0, 320.0), vec2(320.0, 712.0), vec2(480.0, 320.0)] {
            game.resize(size);
            assert_eq!(game.player.position + Vec2::splat(16.0), size * 0.5);
            assert_eq!(game.player.movement, movement);
            assert_eq!(game.player.speed_up_counter, 42);
            assert_eq!(game.diamonds(), (1, 1));
            for (object, expected) in game.world.game_objects.iter().zip(&relative) {
                let actual = object.rect().offset(-game.player.position);
                assert!((actual.point() - expected.point()).length() < 0.0001);
                assert_eq!(actual.size(), expected.size());
            }
            let mut renderer = RecordingRenderer::default();
            game.draw(&mut renderer, false);
            assert!(renderer.commands.iter().any(|command| matches!(command,
                Command::BitmapText(text, position, _) if text == "5.3" && *position == size - vec2(50.0, 35.0))));
        }
    }

    #[test]
    fn drawing_preserves_layers_and_hides_collected_objects_without_gpu_resources() {
        // Put the transparent diamond before the solid platform in XML to check
        // that rendering still draws solid objects before transparent ones.
        let mut game = test_support::game_with(
            "<FPDiamond><x>0</x><y>0</y></FPDiamond><FPPlatform><x>0</x><y>32</y><widthSegments>1</widthSegments><heightSegments>1</heightSegments></FPPlatform>",
        );
        let mut renderer = RecordingRenderer::default();
        game.draw(&mut renderer, false);
        let objects: Vec<_> = renderer
            .commands
            .iter()
            .filter_map(|command| match command {
                Command::Sprite(SpriteId::Background(_), _, _) => None,
                Command::Sprite(id, _, _) => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(
            objects,
            [SpriteId::Platform, SpriteId::Diamond, SpriteId::Ball]
        );
        assert!(matches!(
            renderer.commands.first(),
            Some(Command::Sprite(SpriteId::Background(1), _, _))
        ));
        assert!(
            matches!(renderer.commands.last(), Some(Command::BitmapText(text, _, _)) if text == "Diamonds: 0/1")
        );

        game.world.update(&mut game.player);
        game.player
            .update(&mut game.world, GameInput::new(1.0, false));
        renderer.commands.clear();
        game.draw(&mut renderer, true);
        assert!(
            !renderer
                .commands
                .iter()
                .any(|command| matches!(command, Command::Sprite(SpriteId::Diamond, _, _)))
        );
        assert!(renderer.commands.iter().any(|command| matches!(command,
            Command::Sprite(SpriteId::Ball, position, style)
                if *position == game.player.position
                    && (style.rotation - 3.3_f32.to_radians()).abs() < 0.000001
                    && style.color == WHITE && !style.flip_x
        )));
        assert!(matches!(
            renderer.commands.first(),
            Some(Command::Sprite(SpriteId::Background(0), _, _))
        ));
        assert!(
            matches!(renderer.commands.last(), Some(Command::BitmapText(text, _, _)) if text == "Diamonds: 1/1")
        );
    }
}
