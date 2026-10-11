use super::{collision::collision_intersection, player::Player};
use super::{
    game_object::{GameObject, UpdateContext},
    level_loader::parse_value,
};
use crate::drawing::{Renderer, SpriteId};
use crate::math::*;

pub struct MovablePlatform {
    position: Vec2,
    width_segments: i32,
    height_segments: i32,
    move_y: f32,
}

impl MovablePlatform {
    pub fn parse(node: roxmltree::Node) -> Self {
        Self {
            position: Vec2 {
                x: parse_value(node, "x"),
                y: parse_value(node, "y"),
            },
            width_segments: parse_value(node, "widthSegments"),
            height_segments: parse_value(node, "heightSegments"),
            move_y: 0.0,
        }
    }

    fn collision_up_down(&mut self, context: &UpdateContext<'_>) -> bool {
        let mut is_colliding = false;

        for platform_rect in context.platform_rects() {
            // Recompute after each correction, matching the original object order.
            let rect = self.rect();
            let Some(intersection) = collision_intersection(platform_rect, rect) else {
                continue;
            };

            if platform_rect.bottom() < rect.bottom() {
                if self.move_y > 0.0 {
                    self.move_y = 0.0;
                }
                self.position.y += intersection.h;
                is_colliding = true;
            } else if self.move_y < 0.0 {
                if platform_rect.top() > rect.bottom() - Player::TOLERANCE + self.move_y {
                    self.move_y = 0.0;
                    self.position.y -= intersection.h;
                    is_colliding = true;
                }
            } else if platform_rect.top() > rect.bottom() - Player::TOLERANCE + self.move_y {
                self.position.y -= intersection.h;
                is_colliding = true;
            }
        }

        is_colliding
    }
}

impl GameObject for MovablePlatform {
    fn is_transparent(&self) -> bool {
        false
    }
    fn set_vertical_speed(&mut self, speed: f32) {
        self.move_y = speed;
    }
    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width_segments as f32 * 32.0,
            self.height_segments as f32 * 32.0,
        )
    }

    fn is_platform(&self) -> bool {
        true
    }

    fn is_movable(&self) -> bool {
        true
    }

    fn update(&mut self, context: &UpdateContext<'_>) {
        // The original player and movable platform share these physics values.
        self.move_y = (self.move_y - Player::DECELERATION).max(Player::MAX_FALL_SPEED);

        // Cover the entire proposed movement, as CGRectWithMove does.
        let rect = self.rect();
        let swept_rect = rect.combine_with(rect.offset(vec2(0.0, -self.move_y)));
        if collision_intersection(context.player_rect, swept_rect).is_some() {
            self.move_y = 0.0;
        }

        self.position.y -= self.move_y;
        self.collision_up_down(context);
    }

    fn draw(&self, renderer: &mut dyn Renderer) {
        renderer.tiles(SpriteId::Movable, self.rect(), vec2(32.0, 32.0));
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        test_support::{
            add_object, assert_close, distant_player, empty_context, movable_at, platform_at,
        },
        world::World,
    };
    use super::*;

    #[test]
    fn movable_accelerates_downward_and_caps_fall_speed() {
        let mut movable = movable_at(0.0, 0.0, 0.0);
        let context = empty_context();
        movable.update(&context);
        assert_close(movable.move_y, -0.22);
        assert_close(movable.position.y, 0.22);

        for _ in 0..100 {
            movable.update(&context);
        }
        assert_eq!(movable.move_y, -15.0);
        let previous_y = movable.position.y;
        movable.update(&context);
        assert_close(movable.position.y - previous_y, 15.0);
    }

    #[test]
    fn movable_checks_player_against_the_entire_swept_rectangle() {
        // Includes initial overlap while moving away, not just destination overlap.
        for (move_y, player_y) in [(-15.0, 33.0), (-15.0, 0.0), (15.0, -2.0), (15.0, 30.0)] {
            let mut movable = movable_at(0.0, 0.0, move_y);
            let context = UpdateContext {
                player_rect: Rect::new(0.0, player_y, 32.0, 1.0),
                ..empty_context()
            };
            movable.update(&context);
            assert_eq!(movable.position.y, 0.0);
            assert_eq!(movable.move_y, 0.0);
        }
    }

    #[test]
    fn movable_lands_on_platforms_before_or_after_it_in_the_world() {
        for floor_first in [false, true] {
            let mut world = World::new();
            let floor = Box::new(platform_at(vec2(0.0, 64.0), 1, 1));
            let movable = Box::new(movable_at(0.0, 0.0, 0.0));
            let (movable_index, floor_index) = if floor_first {
                world.game_objects.push(floor);
                world.game_objects.push(movable);
                (1, 0)
            } else {
                world.game_objects.push(movable);
                world.game_objects.push(floor);
                (0, 1)
            };

            for _ in 0..120 {
                world.update(&mut distant_player());
            }
            assert_close(world.game_objects[movable_index].rect().bottom(), 64.0);
            assert_eq!(world.game_objects[floor_index].rect().top(), 64.0);
            assert_eq!(world.background_offset, Vec2::ZERO);
        }
    }

    #[test]
    fn movable_ceiling_collision_corrects_only_itself() {
        let mut world = World::new();
        add_object(&mut world, Rect::new(0.0, 0.0, 32.0, 32.0), true, false);
        let context = UpdateContext {
            before: &world.game_objects,
            ..empty_context()
        };

        for (speed, expected_speed) in [(4.0, 0.0), (-4.0, -4.0)] {
            let mut movable = movable_at(0.0, 30.0, speed);
            assert!(movable.collision_up_down(&context));
            assert_eq!(movable.position.y, 32.0);
            assert_eq!(movable.move_y, expected_speed);
            assert_eq!(world.game_objects[0].rect().top(), 0.0);
        }
    }

    #[test]
    fn movable_landing_uses_original_velocity_adjusted_threshold() {
        for (speed, floor_top, lands) in [
            (-4.0, 25.0, false),
            (-4.0, 26.0, true),
            (0.0, 29.0, false),
            (0.0, 30.0, true),
            (1.0, 30.0, false),
            (1.0, 31.0, true),
        ] {
            let mut world = World::new();
            add_object(
                &mut world,
                Rect::new(0.0, floor_top, 32.0, 32.0),
                true,
                false,
            );
            let context = UpdateContext {
                after: &world.game_objects,
                ..empty_context()
            };
            let mut movable = movable_at(0.0, 0.0, speed);

            assert_eq!(movable.collision_up_down(&context), lands);
            assert_eq!(
                movable.position.y,
                if lands { floor_top - 32.0 } else { 0.0 }
            );
            assert_eq!(
                movable.move_y,
                if lands && speed < 0.0 { 0.0 } else { speed }
            );
        }
    }

    #[test]
    fn movable_rechecks_its_bounds_after_each_vertical_correction() {
        let mut world = World::new();
        add_object(&mut world, Rect::new(0.0, 30.0, 32.0, 32.0), true, false);
        add_object(&mut world, Rect::new(0.0, 31.0, 32.0, 32.0), true, false);
        let context = UpdateContext {
            after: &world.game_objects,
            ..empty_context()
        };
        let mut movable = movable_at(0.0, 0.0, -4.0);

        assert!(movable.collision_up_down(&context));
        assert_eq!(movable.position.y, -2.0);
        assert_eq!(movable.move_y, 0.0);
    }

    #[test]
    fn player_can_push_a_real_movable_platform_until_blocked() {
        let mut player = Player::new();
        let mut world = World::new();
        world
            .game_objects
            .push(Box::new(movable_at(254.0, 144.0, 0.0)));

        assert!(!player.collision_left_right(&mut world));
        assert_eq!(world.game_objects[0].rect().left(), 256.0);
        assert_eq!(world.background_offset, Vec2::ZERO);

        // Bring the block back into contact, then prevent the next push.
        world.game_objects[0].move_world(vec2(-2.0, 0.0));
        add_object(&mut world, Rect::new(286.0, 144.0, 32.0, 32.0), true, false);
        assert!(player.collision_left_right(&mut world));
        assert_eq!(world.game_objects[0].rect().left(), 256.0);
        assert_eq!(world.game_objects[1].rect().left(), 288.0);
        assert_eq!(world.background_offset, vec2(0.5, 0.0));
    }
}
