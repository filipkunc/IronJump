use super::{collision::collision_intersection, world::World};
use crate::drawing::{Color, GAME_HEIGHT, GAME_WIDTH, Renderer, SpriteId, SpriteStyle, WHITE};
use crate::input::GameInput;
use crate::math::*;

pub struct Player {
    pub(super) position: Vec2,
    pub(super) movement: Vec2,
    rotation: f32,
    jumping: bool,
    pub(super) speed_up_counter: u32,
    alpha: f32,
}

impl Player {
    pub(super) const TOLERANCE: f32 = 3.0;
    const MAX_SPEED: f32 = 5.8;
    const SPEED_POWER_UP: f32 = 1.5;
    const UP_SPEED: f32 = 7.0;
    pub(super) const MAX_FALL_SPEED: f32 = -15.0;
    const ACCELERATION: f32 = 1.1;
    pub(super) const DECELERATION: f32 = Self::ACCELERATION * 0.2;
    const CHANGE_DIRECTION_SPEED: f32 = 3.0;
    pub(super) const MAX_SPEED_UP_COUNT: u32 = 60 * 6; // 60 FPS * 6 sec

    pub fn new() -> Self {
        Self {
            position: Vec2 {
                x: GAME_WIDTH / 2.0 - 32.0 / 2.0,
                y: GAME_HEIGHT / 2.0 - 32.0 / 2.0,
            },
            movement: Vec2 { x: 0.0, y: 0.0 },
            rotation: 0.0,
            jumping: false,
            speed_up_counter: 0,
            alpha: 1.0,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 32.0, 32.0)
    }

    pub(super) fn collision_left_right(&mut self, world: &mut World) -> bool {
        let mut is_colliding = false;
        let player_rect = self.rect();

        // Re-read each object's rectangle after any previous world correction.
        for index in 0..world.game_objects.len() {
            let object = &world.game_objects[index];
            if !object.is_platform() {
                continue;
            }
            let platform_rect = object.rect();
            let Some(intersection) = collision_intersection(platform_rect, player_rect) else {
                continue;
            };

            let offset = if platform_rect.left() > player_rect.left() {
                Vec2::new(intersection.w, 0.0)
            } else if platform_rect.right() < player_rect.right() {
                Vec2::new(-intersection.w, 0.0)
            } else {
                continue;
            };

            if object.is_movable() {
                world.game_objects[index].move_world(offset);
                if !world.platform_collision_left_right(index) {
                    continue;
                }
                // A blocked push is undone before moving the whole world.
                world.game_objects[index].move_world(-offset);
            }

            world.move_world(offset);
            is_colliding = true;
        }

        is_colliding
    }

    fn collision_up_down(&mut self, world: &mut World) -> bool {
        let mut is_colliding = false;
        let player_rect = self.rect();

        for index in 0..world.game_objects.len() {
            let object = &world.game_objects[index];
            if !object.is_platform() {
                continue;
            }
            let platform_rect = object.rect();
            let Some(intersection) = collision_intersection(platform_rect, player_rect) else {
                continue;
            };

            if platform_rect.bottom() < player_rect.bottom() {
                if self.movement.y > 0.0 {
                    self.movement.y = 0.0;
                }
                world.move_world(Vec2::new(0.0, -intersection.h));
                is_colliding = true;
            } else if self.movement.y < 0.0 {
                if platform_rect.top() > player_rect.bottom() - Self::TOLERANCE + self.movement.y {
                    self.movement.y = 0.0;
                    self.jumping = false;
                    world.move_world(Vec2::new(0.0, intersection.h));
                    is_colliding = true;
                }
            } else if platform_rect.top() > player_rect.bottom() - Self::TOLERANCE + self.movement.y
            {
                self.jumping = false;
                world.move_world(Vec2::new(0.0, intersection.h));
                is_colliding = true;
            }
        }

        is_colliding
    }

    pub fn update(&mut self, world: &mut World, input: GameInput) {
        let mut move_left_or_right = false;
        if self.speed_up_counter > 0 {
            self.speed_up_counter += 1;
            if self.speed_up_counter > Self::MAX_SPEED_UP_COUNT {
                self.speed_up_counter = 0;
            }
        }
        let current_max_speed = if self.speed_up_counter > 0 {
            Self::MAX_SPEED * Self::SPEED_POWER_UP
        } else {
            Self::MAX_SPEED
        };

        if input.move_axis < 0.0 {
            if self.movement.x < 0.0 {
                self.movement.x +=
                    input.move_axis.abs() * Self::ACCELERATION * Self::CHANGE_DIRECTION_SPEED;
            }
            self.movement.x = (self.movement.x + input.move_axis.abs() * Self::ACCELERATION)
                .min(current_max_speed);
            move_left_or_right = true;
        } else if input.move_axis > 0.0 {
            if self.movement.x > 0.0 {
                self.movement.x -=
                    input.move_axis.abs() * Self::ACCELERATION * Self::CHANGE_DIRECTION_SPEED;
            }
            self.movement.x = (self.movement.x - input.move_axis.abs() * Self::ACCELERATION)
                .max(-current_max_speed);
            move_left_or_right = true;
        }
        if !self.jumping && input.jump {
            if self.movement.y < Self::UP_SPEED {
                self.movement.y = Self::UP_SPEED;
            }
            self.jumping = true;
        }

        if !move_left_or_right {
            if self.movement.x.abs() < Self::DECELERATION {
                self.movement.x = 0.0;
            } else if self.movement.x > 0.0 {
                self.movement.x -= Self::DECELERATION;
            } else if self.movement.x < 0.0 {
                self.movement.x += Self::DECELERATION;
            }
        }

        self.movement.y -= Self::DECELERATION;
        if self.movement.y < Self::MAX_FALL_SPEED {
            self.movement.y = Self::MAX_FALL_SPEED;
        }
        self.jumping = true;

        world.move_world(Vec2::new(self.movement.x, 0.0));
        if self.collision_left_right(world) {
            self.movement.x = 0.0;
        }
        world.move_world(Vec2::new(0.0, self.movement.y));
        self.collision_up_down(world);
        self.rotation -= self.movement.x * 3.0;
        self.alpha = (self.alpha + 0.07) % std::f32::consts::PI;
    }

    pub fn draw(&self, renderer: &mut dyn Renderer) {
        renderer.sprite_ex(
            SpriteId::Ball,
            self.position,
            SpriteStyle {
                color: WHITE,
                rotation: self.rotation.to_radians(),
                flip_x: false,
            },
        );
        if self.speed_up_counter > 0 {
            renderer.sprite_ex(
                SpriteId::SpeedEffect,
                self.position - vec2(16.0, 16.0),
                SpriteStyle {
                    color: Color::new(1.0, 1.0, 1.0, self.alpha.sin().abs() * 0.5 + 0.5),
                    rotation: 0.0,
                    flip_x: false,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{add_object, assert_close, platform_at};
    use super::*;

    #[test]
    fn side_collisions_move_the_whole_world_in_both_directions() {
        for (x, correction) in [(254.0, 2.0), (194.0, -2.0)] {
            let mut player = Player::new();
            let original_position = player.position;
            let mut world = World::new();
            add_object(&mut world, Rect::new(x, 144.0, 32.0, 32.0), true, false);
            add_object(&mut world, Rect::new(0.0, 0.0, 32.0, 32.0), false, false);

            assert!(player.collision_left_right(&mut world));
            assert_eq!(world.game_objects[0].rect().x, x + correction);
            assert_eq!(world.game_objects[1].rect().x, correction);
            assert_eq!(player.position, original_position);
        }
    }

    #[test]
    fn non_platform_objects_do_not_block_the_player() {
        let mut player = Player::new();
        let mut world = World::new();
        let rect = Rect::new(254.0, 174.0, 32.0, 32.0);
        add_object(&mut world, rect, false, false);
        player.movement.y = -4.0;
        player.jumping = true;

        assert!(!player.collision_left_right(&mut world));
        assert!(!player.collision_up_down(&mut world));
        assert_eq!(world.game_objects[0].rect(), rect);
        assert_eq!(player.movement.y, -4.0);
        assert!(player.jumping);
    }

    #[test]
    fn later_collision_checks_use_corrected_world_positions() {
        let mut player = Player::new();
        let mut world = World::new();
        add_object(&mut world, Rect::new(254.0, 144.0, 32.0, 32.0), true, false);
        add_object(&mut world, Rect::new(255.0, 144.0, 32.0, 32.0), true, false);

        assert!(player.collision_left_right(&mut world));
        assert_eq!(world.game_objects[0].rect().x, 256.0);
        assert_eq!(world.game_objects[1].rect().x, 257.0);
    }

    #[test]
    fn unblocked_movable_platforms_are_pushed_without_scrolling() {
        for (x, correction) in [(254.0, 2.0), (194.0, -2.0)] {
            let mut player = Player::new();
            let mut world = World::new();
            add_object(&mut world, Rect::new(x, 144.0, 32.0, 32.0), true, true);
            // An overlapping decoration must not block the push.
            let decoration = Rect::new(x, 144.0, 32.0, 32.0);
            add_object(&mut world, decoration, false, false);

            assert!(!player.collision_left_right(&mut world));
            assert_eq!(world.game_objects[0].rect().x, x + correction);
            assert_eq!(world.game_objects[1].rect(), decoration);
        }
    }

    #[test]
    fn blocked_pushes_are_undone_before_world_correction() {
        for (x, blocker_x, correction) in [(254.0, 286.0, 2.0), (194.0, 162.0, -2.0)] {
            let mut player = Player::new();
            let mut world = World::new();
            add_object(&mut world, Rect::new(x, 144.0, 32.0, 32.0), true, true);
            add_object(
                &mut world,
                Rect::new(blocker_x, 144.0, 32.0, 32.0),
                true,
                false,
            );

            assert!(player.collision_left_right(&mut world));
            assert_eq!(world.game_objects[0].rect().x, x + correction);
            assert_eq!(world.game_objects[1].rect().x, blocker_x + correction);
        }
    }

    #[test]
    fn landing_stops_falling_and_restores_jump_state() {
        for speed in [-4.0, 0.0] {
            let mut player = Player::new();
            let mut world = World::new();
            add_object(&mut world, Rect::new(192.0, 174.0, 96.0, 32.0), true, false);
            player.movement.y = speed;
            player.jumping = true;

            assert!(player.collision_up_down(&mut world));
            assert_eq!(player.movement.y, 0.0);
            assert!(!player.jumping);
            assert_eq!(world.game_objects[0].rect().top(), player.rect().bottom());
        }
    }

    #[test]
    fn ceiling_collision_cancels_upward_but_not_downward_motion() {
        for (speed, expected) in [(4.0, 0.0), (-4.0, -4.0)] {
            let mut player = Player::new();
            let mut world = World::new();
            add_object(&mut world, Rect::new(192.0, 114.0, 96.0, 32.0), true, false);
            player.movement.y = speed;
            player.jumping = true;

            assert!(player.collision_up_down(&mut world));
            assert_eq!(player.movement.y, expected);
            assert!(player.jumping);
            assert_eq!(world.game_objects[0].rect().bottom(), player.rect().top());
        }
    }

    #[test]
    fn landing_threshold_matches_the_original_strict_velocity_adjusted_check() {
        for (speed, top, lands) in [
            (-4.0, 169.0, false),
            (-4.0, 170.0, true),
            (0.0, 173.0, false),
            (0.0, 174.0, true),
            (1.0, 174.0, false),
            (1.0, 175.0, true),
        ] {
            let mut player = Player::new();
            let mut world = World::new();
            add_object(&mut world, Rect::new(192.0, top, 96.0, 32.0), true, false);
            player.movement.y = speed;
            player.jumping = true;

            assert_eq!(player.collision_up_down(&mut world), lands);
            assert_eq!(player.jumping, !lands);
            assert_eq!(
                player.movement.y,
                if lands && speed < 0.0 { 0.0 } else { speed }
            );
            assert_eq!(
                world.game_objects[0].rect().top(),
                if lands { 176.0 } else { top }
            );
        }
    }

    #[test]
    fn update_stops_horizontal_velocity_at_a_wall() {
        for (wall_x, speed, input) in [(256.0, -2.0, 1.0), (192.0, 2.0, -1.0)] {
            let mut player = Player::new();
            let mut world = World::new();
            add_object(
                &mut world,
                Rect::new(wall_x, 100.0, 32.0, 100.0),
                true,
                false,
            );
            player.movement.x = speed;

            player.update(&mut world, GameInput::new(input, false));
            assert_eq!(player.movement.x, 0.0);
            assert_eq!(player.rotation, 0.0);
            assert_close(world.game_objects[0].rect().x, wall_x);
        }
    }

    #[test]
    fn repeated_updates_keep_player_grounded_and_allow_jump_and_landing() {
        let mut player = Player::new();
        let mut world = World::new();
        world
            .game_objects
            .push(Box::new(platform_at(vec2(192.0, 176.0), 3, 1)));

        for _ in 0..120 {
            player.update(&mut world, GameInput::NONE);
            assert!(!player.jumping);
            assert_eq!(player.movement.y, 0.0);
            assert_close(world.game_objects[0].rect().top(), 176.0);
        }

        player.update(&mut world, GameInput::new(0.0, true));
        assert!(player.jumping);
        assert!(player.movement.y > 0.0);
        for _ in 0..120 {
            player.update(&mut world, GameInput::NONE);
        }
        assert!(!player.jumping);
        assert_eq!(player.movement.y, 0.0);
        assert_close(world.game_objects[0].rect().top(), 176.0);
    }

    #[test]
    fn player_speed_is_clamped_to_original_normal_and_boosted_limits() {
        for (boosted, expected) in [(false, 5.8), (true, 8.7)] {
            for direction in [-1.0, 1.0] {
                let mut player = Player::new();
                if boosted {
                    player.speed_up_counter = 1;
                }
                let mut world = World::new();
                for _ in 0..30 {
                    player.update(&mut world, GameInput::new(direction, false));
                }
                assert!((player.movement.x + direction * expected).abs() < 0.0001);
            }
        }
    }
}
