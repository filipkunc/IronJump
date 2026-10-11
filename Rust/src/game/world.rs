use super::{
    collision::collision_intersection,
    game_object::{GameObject, UpdateContext, WorldEffect},
    player::Player,
};
use crate::assets::Assets;
use macroquad::prelude::*;

// The player lives outside World so both can be borrowed mutably during updates.
pub struct World {
    pub(super) game_objects: Vec<Box<dyn GameObject>>,
    width: f32,
    height: f32,
    pub(super) background_offset: Vec2,
}

impl World {
    pub fn new() -> Self {
        Self {
            game_objects: Vec::new(),
            width: 480.0,
            height: 320.0,
            background_offset: Vec2::new(0.0, 0.0),
        }
    }

    pub fn update(&mut self, player: &mut Player) {
        // FPGame updates every non-movable object before the movable platforms.
        for movable_pass in [false, true] {
            for index in 0..self.game_objects.len() {
                if self.game_objects[index].is_movable() != movable_pass {
                    continue;
                }
                // Elevators can affect both the player and the entire world.
                if self.game_objects[index].as_elevator().is_some() {
                    self.update_elevator(index, player);
                    continue;
                }
                let (before, remaining) = self.game_objects.split_at_mut(index);
                let (object, after) = remaining.split_first_mut().unwrap();
                let context = UpdateContext {
                    player_rect: player.rect(),
                    before,
                    after,
                };
                // Later objects observe earlier objects' updated positions this frame.
                object.update(&context);
                let effect = object.interact(player);
                if let Some(WorldEffect::BounceMovables { surface, speed }) = effect {
                    for object in &mut self.game_objects {
                        if !object.is_movable() {
                            continue;
                        }
                        let mut rect = object.rect();
                        rect.h += Player::TOLERANCE;
                        if rect
                            .intersect(surface)
                            .is_some_and(|r| r.h > 0.0 && r.w > 30.0)
                        {
                            object.set_vertical_speed(speed);
                        }
                    }
                }
            }
        }
    }

    pub fn draw(&self, assets: &Assets, custom: bool) {
        const TILE_SIZE: f32 = 32.0;

        // Align the whole grid to the render target's pixels. At half-pixel
        // positions, nearest sampling can pick atlas padding along tile edges.
        // Keep the accumulated offset fractional so slow scrolling still advances.
        let offset_x = self.background_offset.x.round().rem_euclid(TILE_SIZE) - TILE_SIZE;
        let offset_y = self.background_offset.y.round().rem_euclid(TILE_SIZE) - TILE_SIZE;
        // One extra tile covers each edge while the background scrolls.
        let columns = (self.width / TILE_SIZE).ceil() as usize + 1;
        let rows = (self.height / TILE_SIZE).ceil() as usize + 1;

        for row in 0..rows {
            for column in 0..columns {
                let x = column as f32 * TILE_SIZE + offset_x;
                let y = row as f32 * TILE_SIZE + offset_y;
                assets.background[usize::from(!custom)].draw(x, y);
            }
        }

        for transparent in [false, true] {
            for game_object in &self.game_objects {
                if game_object.is_visible() && game_object.is_transparent() == transparent {
                    game_object.draw(assets);
                }
            }
        }
    }

    pub fn move_world(&mut self, offset: Vec2) {
        for game_object in self.game_objects.iter_mut() {
            game_object.move_world(offset);
        }
        self.background_offset += offset * 0.25;
    }

    // Matches FPMovablePlatform's horizontal collision query after a push.
    // An index identifies the object without holding a borrow across world changes.
    pub(super) fn platform_collision_left_right(&self, index: usize) -> bool {
        let rect = self.game_objects[index].rect();
        self.game_objects
            .iter()
            .enumerate()
            .any(|(other_index, object)| {
                other_index != index
                    && object.is_platform()
                    && collision_intersection(object.rect(), rect).is_some()
                    && (object.rect().left() > rect.left() || object.rect().right() < rect.right())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        add_object, assert_close, distant_player, game_with, movable_at,
    };
    use super::*;

    #[test]
    fn world_update_excludes_self_and_non_platforms_from_collision_context() {
        let mut world = World::new();
        world.game_objects.push(Box::new(movable_at(0.0, 0.0, 0.0)));
        add_object(&mut world, Rect::new(0.0, 32.0, 32.0, 32.0), false, false);

        for _ in 0..40 {
            world.update(&mut distant_player());
        }
        assert!(world.game_objects[0].rect().top() > 64.0);
        assert_eq!(world.game_objects[1].rect().top(), 32.0);
    }

    #[test]
    fn later_movable_observes_earlier_movable_after_its_update() {
        let mut world = World::new();
        // The lower block moves first, making room for the upper block to follow.
        world
            .game_objects
            .push(Box::new(movable_at(0.0, 64.0, -1.0)));
        world
            .game_objects
            .push(Box::new(movable_at(0.0, 32.0, -1.0)));

        world.update(&mut distant_player());

        assert_close(world.game_objects[0].rect().top(), 65.22);
        assert_close(world.game_objects[1].rect().top(), 33.22);
    }

    #[test]
    fn trampoline_updates_before_movable_even_when_movable_is_first_in_xml() {
        let mut game = game_with(
            "<FPMovablePlatform><x>96</x><y>0</y><widthSegments>1</widthSegments><heightSegments>1</heightSegments></FPMovablePlatform><FPTrampoline><x>96</x><y>32</y><widthSegments>1</widthSegments></FPTrampoline>",
        );
        let original_y = game.world.game_objects[0].rect().y;
        game.world.update(&mut game.player);
        assert!((game.world.game_objects[0].rect().y - (original_y - 7.78)).abs() < 0.0001);
    }
}
