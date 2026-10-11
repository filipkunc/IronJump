use super::{
    Game,
    game_object::{GameObject, UpdateContext},
    movable_platform::MovablePlatform,
    platform::Platform,
    player::Player,
    world::World,
};
use crate::drawing::Renderer;
use crate::math::*;

struct TestObject {
    rect: Rect,
    platform: bool,
    movable: bool,
}

impl GameObject for TestObject {
    fn update(&mut self, _context: &UpdateContext<'_>) {}
    fn draw(&self, _renderer: &mut dyn Renderer) {}

    fn move_world(&mut self, offset: Vec2) {
        self.rect = self.rect.offset(offset);
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn is_platform(&self) -> bool {
        self.platform
    }

    fn is_movable(&self) -> bool {
        self.movable
    }
}

pub(super) fn add_object(world: &mut World, rect: Rect, platform: bool, movable: bool) {
    world.game_objects.push(Box::new(TestObject {
        rect,
        platform,
        movable,
    }));
}

pub(super) fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
}

pub(super) fn game_with(objects: &str) -> Game {
    let xml =
        format!("<IronJumpLevel><FPPlayer><x>0</x><y>0</y></FPPlayer>{objects}</IronJumpLevel>");
    let mut game = Game::new();
    game.load(&xml).unwrap();
    game
}

pub(super) fn empty_context() -> UpdateContext<'static> {
    UpdateContext {
        player_rect: Rect::new(1000.0, 1000.0, 32.0, 32.0),
        before: &[],
        after: &[],
    }
}

pub(super) fn distant_player() -> Player {
    player_at(vec2(1000.0, 1000.0))
}

pub(super) fn player_at(position: Vec2) -> Player {
    let mut player = Player::new();
    player.position = position;
    player
}

pub(super) fn platform_at(position: Vec2, width: i32, height: i32) -> Platform {
    let xml = format!(
        "<FPPlatform><x>{}</x><y>{}</y><widthSegments>{width}</widthSegments><heightSegments>{height}</heightSegments></FPPlatform>",
        position.x, position.y
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    Platform::parse(doc.root_element())
}

pub(super) fn movable_at(x: f32, y: f32, move_y: f32) -> MovablePlatform {
    let xml = format!(
        "<FPMovablePlatform><x>{x}</x><y>{y}</y><widthSegments>1</widthSegments><heightSegments>1</heightSegments></FPMovablePlatform>"
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let mut movable = MovablePlatform::parse(doc.root_element());
    movable.set_vertical_speed(move_y);
    movable
}
