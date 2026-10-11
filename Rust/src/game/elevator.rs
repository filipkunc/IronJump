use super::{
    collision::collision_intersection,
    game_object::{GameObject, UpdateContext},
    level_loader::parse_value,
    player::Player,
    world::World,
};
use crate::assets::Assets;
use macroquad::prelude::*;

pub struct Elevator {
    position: Vec2,
    start: Vec2,
    end: Vec2,
    width_segments: i32,
    moving_to_end: bool,
    animation_counter: u8,
    texture_index: usize,
    // World objects are not removed or reordered during play. These indices
    // identify the objects attached when the elevator first updates.
    affected_objects: Option<Vec<usize>>,
}

impl Elevator {
    pub fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        let position = vec2(parse_value(node, "x"), parse_value(node, "y"));
        Self {
            position,
            start: position,
            end: vec2(parse_value(node, "endX"), parse_value(node, "endY")),
            width_segments: parse_value(node, "widthSegments"),
            moving_to_end: true,
            animation_counter: 0,
            texture_index: 0,
            affected_objects: None,
        }
    }

    fn next_movement(&mut self) -> Vec2 {
        const SPEED: f32 = 2.0;
        let target = if self.moving_to_end {
            self.end
        } else {
            self.start
        };
        let difference = target - self.position;
        let movement = vec2(
            difference.x.clamp(-SPEED, SPEED),
            difference.y.clamp(-SPEED, SPEED),
        );
        if movement.x.abs() < 0.1 && movement.y.abs() < 0.1 {
            self.moving_to_end = !self.moving_to_end;
        }
        movement
    }

    fn animate(&mut self, requested_y: f32) {
        // The original animation follows the requested direction, even if blocked.
        if requested_y == 0.0 {
            self.texture_index = 1;
        } else {
            self.animation_counter += 1;
            if self.animation_counter > 2 {
                self.animation_counter = 0;
                if requested_y < 0.0 {
                    self.texture_index = (self.texture_index + 1).min(2);
                } else {
                    self.texture_index = self.texture_index.saturating_sub(1);
                }
            }
        }
    }
}

impl GameObject for Elevator {
    // World coordinates the complete elevator update so no object borrow is held
    // while scrolling the world or resolving player collisions.
    fn update(&mut self, _context: &UpdateContext<'_>) {}

    fn draw(&self, assets: &Assets) {
        for segment in 0..self.width_segments {
            assets.elevator[self.texture_index]
                .draw(self.position.x + segment as f32 * 32.0, self.position.y);
        }
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
        self.start += offset;
        self.end += offset;
    }

    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width_segments as f32 * 32.0,
            32.0,
        )
    }

    fn is_platform(&self) -> bool {
        true
    }

    fn as_elevator(&self) -> Option<&Elevator> {
        Some(self)
    }

    fn as_elevator_mut(&mut self) -> Option<&mut Elevator> {
        Some(self)
    }
}

fn swept_rect(rect: Rect, movement: Vec2) -> Rect {
    rect.combine_with(rect.offset(movement))
}

fn with_landing_tolerance(mut rect: Rect) -> Rect {
    rect.h += Player::TOLERANCE;
    rect
}

// Initial attachment detection uses CGRectIntersectsRect rather than the 0.1
// collision tolerance. Edge-only contact is still not an intersection.
fn intersects(a: Rect, b: Rect) -> bool {
    a.intersect(b)
        .is_some_and(|rect| rect.w > 0.0 && rect.h > 0.0)
}

impl World {
    fn elevator_attachments(&self, index: usize) -> Vec<usize> {
        let elevator_rect = self.game_objects[index].rect();
        let mut affected: Vec<usize> = self
            .game_objects
            .iter()
            .enumerate()
            .filter(|(other, object)| {
                *other != index && intersects(with_landing_tolerance(object.rect()), elevator_rect)
            })
            .map(|(other, _)| other)
            .collect();

        // Preserve the original second pass, adding each indirect attachment once.
        for (other, object) in self.game_objects.iter().enumerate() {
            if other == index || affected.contains(&other) {
                continue;
            }
            let rect = with_landing_tolerance(object.rect());
            if affected
                .iter()
                .any(|&attached| intersects(rect, self.game_objects[attached].rect()))
            {
                affected.push(other);
            }
        }
        affected
    }

    pub(super) fn update_elevator(&mut self, index: usize, player: &mut Player) {
        if self.game_objects[index]
            .as_elevator()
            .unwrap()
            .affected_objects
            .is_none()
        {
            let affected = self.elevator_attachments(index);
            self.game_objects[index]
                .as_elevator_mut()
                .unwrap()
                .affected_objects = Some(affected);
        }
        let (requested, affected) = {
            let elevator = self.game_objects[index].as_elevator_mut().unwrap();
            (
                elevator.next_movement(),
                elevator.affected_objects.as_ref().unwrap().clone(),
            )
        };
        let mut movement = requested;
        let proposed_rect = swept_rect(self.game_objects[index].rect(), movement);

        let blocked = collision_intersection(player.rect(), proposed_rect).is_some()
            || self.game_objects.iter().any(|object| {
                object.is_movable()
                    && collision_intersection(object.rect(), proposed_rect).is_some()
            });
        if blocked {
            if movement.y > 0.0 {
                movement.y = 0.0;
            }
            movement.x = 0.0;
        }

        let player_rect = with_landing_tolerance(player.rect());
        let carries_player = collision_intersection(player_rect, proposed_rect).is_some()
            || affected.iter().any(|&attached| {
                collision_intersection(
                    player_rect,
                    swept_rect(self.game_objects[attached].rect(), movement),
                )
                .is_some()
            });
        if carries_player {
            self.carry_player(player, movement);
        }

        // Find a loose movable supported by the elevator or one of its attachments.
        // An initially attached movable is already carried below; do not move it twice.
        let movable = self
            .game_objects
            .iter()
            .enumerate()
            .find_map(|(other, object)| {
                if !object.is_movable() || affected.contains(&other) {
                    return None;
                }
                let rect = with_landing_tolerance(object.rect());
                let supported =
                    std::iter::once(index)
                        .chain(affected.iter().copied())
                        .any(|support| {
                            let support_rect =
                                swept_rect(self.game_objects[support].rect(), movement);
                            rect.bottom() < support_rect.bottom()
                                && collision_intersection(rect, support_rect).is_some()
                        });
                supported.then_some(other)
            });

        if let Some(movable) = movable {
            let moved_rect = swept_rect(self.game_objects[movable].rect(), movement);
            let carries_player = collision_intersection(player_rect, moved_rect).is_some();
            self.game_objects[movable].move_world(vec2(movement.x, 0.0));
            if carries_player {
                self.move_world(vec2(-movement.x, 0.0));
                player.collision_left_right(self);
            }
            self.game_objects[movable].move_world(vec2(0.0, movement.y));
            if carries_player {
                self.move_world(vec2(0.0, -movement.y));
            }
        }

        // Route motion changes only the elevator's current position. Camera motion
        // above has already translated its current position and both endpoints.
        let elevator = self.game_objects[index].as_elevator_mut().unwrap();
        elevator.position += movement;
        elevator.animate(requested.y);
        for attached in affected {
            self.game_objects[attached].move_world(movement);
        }
    }

    fn carry_player(&mut self, player: &mut Player, movement: Vec2) {
        self.move_world(vec2(-movement.x, 0.0));
        player.collision_left_right(self);
        self.move_world(vec2(0.0, -movement.y));
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        Game,
        test_support::{distant_player, movable_at, platform_at, player_at},
    };
    use super::*;

    fn elevator(position: Vec2, end: Vec2) -> Elevator {
        Elevator {
            position,
            start: position,
            end,
            width_segments: 3,
            moving_to_end: true,
            animation_counter: 0,
            texture_index: 0,
            affected_objects: Some(Vec::new()),
        }
    }

    fn add_platform(world: &mut World, position: Vec2) {
        world
            .game_objects
            .push(Box::new(platform_at(position, 1, 1)));
    }

    fn add_movable(world: &mut World, position: Vec2) {
        world
            .game_objects
            .push(Box::new(movable_at(position.x, position.y, 0.0)));
    }

    #[test]
    fn elevator_moves_two_pixels_per_axis_and_reverses_at_endpoints() {
        let mut world = World::new();
        world
            .game_objects
            .push(Box::new(elevator(Vec2::ZERO, vec2(5.0, -3.0))));
        let mut player = distant_player();
        for expected in [
            vec2(2.0, -2.0),
            vec2(4.0, -3.0),
            vec2(5.0, -3.0),
            vec2(5.0, -3.0), // Direction flips on the following update, as in Obj-C.
            vec2(3.0, -1.0),
            vec2(1.0, 0.0),
            Vec2::ZERO,
            Vec2::ZERO,
            vec2(2.0, -2.0),
        ] {
            world.update(&mut player);
            assert_eq!(world.game_objects[0].rect().point(), expected);
        }
    }

    #[test]
    fn world_scroll_translates_both_endpoints_without_changing_route() {
        let mut lift = elevator(vec2(10.0, 20.0), vec2(110.0, -80.0));
        lift.move_world(vec2(-300.0, 40.0));
        assert_eq!(lift.position, vec2(-290.0, 60.0));
        assert_eq!(lift.start, vec2(-290.0, 60.0));
        assert_eq!(lift.end, vec2(-190.0, -40.0));
        assert_eq!(lift.next_movement(), vec2(2.0, -2.0));
    }

    #[test]
    fn player_rides_elevator_in_each_direction_while_staying_on_screen() {
        for direction in [
            vec2(2.0, 0.0),
            vec2(-2.0, 0.0),
            vec2(0.0, 2.0),
            vec2(0.0, -2.0),
        ] {
            let mut world = World::new();
            let start = vec2(0.0, 64.0);
            world
                .game_objects
                .push(Box::new(elevator(start, start + direction * 20.0)));
            let mut player = player_at(vec2(32.0, 32.0));

            world.update(&mut player);

            assert_eq!(player.position, vec2(32.0, 32.0));
            assert_eq!(world.game_objects[0].rect().point(), start);
            assert_eq!(world.background_offset, -direction * 0.25);
            let lift = world.game_objects[0].as_elevator().unwrap();
            assert_eq!(lift.end - lift.start, direction * 20.0);
        }
    }

    #[test]
    fn carrying_player_resolves_contact_with_a_wall() {
        let mut world = World::new();
        world
            .game_objects
            .push(Box::new(elevator(vec2(0.0, 64.0), vec2(100.0, 64.0))));
        add_platform(&mut world, vec2(64.0, 32.0));
        let mut player = player_at(vec2(32.0, 32.0));

        world.update(&mut player);

        assert_eq!(world.game_objects[1].rect().left(), player.rect().right());
        assert_eq!(world.game_objects[0].rect().left(), 2.0);
        assert_eq!(world.background_offset, Vec2::ZERO);
    }

    #[test]
    fn descending_elevator_stops_above_player_or_movable() {
        for movable_blocks in [false, true] {
            let mut world = World::new();
            world
                .game_objects
                .push(Box::new(elevator(vec2(0.0, 64.0), vec2(0.0, 128.0))));
            let mut player = if movable_blocks {
                add_movable(&mut world, vec2(32.0, 96.0));
                distant_player()
            } else {
                player_at(vec2(32.0, 96.0))
            };

            world.update_elevator(0, &mut player);
            assert_eq!(world.game_objects[0].rect().top(), 64.0);
            assert_eq!(world.background_offset, Vec2::ZERO);
        }
    }

    #[test]
    fn horizontal_elevator_stops_at_player_or_movable() {
        for movable_blocks in [false, true] {
            let mut world = World::new();
            world
                .game_objects
                .push(Box::new(elevator(vec2(0.0, 64.0), vec2(100.0, 64.0))));
            let mut player = if movable_blocks {
                add_movable(&mut world, vec2(96.0, 64.0));
                distant_player()
            } else {
                player_at(vec2(96.0, 64.0))
            };

            world.update_elevator(0, &mut player);
            assert_eq!(world.game_objects[0].rect().left(), 0.0);
        }
    }

    #[test]
    fn loose_movable_rides_elevator_without_scrolling_world() {
        // The original combined sweep blocks horizontal motion when lifting into a
        // movable. Vertical lifting still proceeds and carries the block upward.
        let mut world = World::new();
        world
            .game_objects
            .push(Box::new(elevator(vec2(0.0, 64.0), vec2(100.0, 0.0))));
        add_movable(&mut world, vec2(32.0, 32.0));

        world.update_elevator(0, &mut distant_player());

        assert_eq!(world.game_objects[0].rect().point(), vec2(0.0, 62.0));
        assert_eq!(world.game_objects[1].rect().point(), vec2(32.0, 30.0));
        assert_eq!(world.background_offset, Vec2::ZERO);
    }

    #[test]
    fn player_rides_a_loose_movable_on_the_elevator() {
        let mut world = World::new();
        world
            .game_objects
            .push(Box::new(elevator(vec2(0.0, 64.0), vec2(100.0, 64.0))));
        add_movable(&mut world, vec2(32.0, 32.0));
        let mut player = player_at(vec2(32.0, 0.0));

        world.update_elevator(0, &mut player);

        assert_eq!(player.position, vec2(32.0, 0.0));
        assert_eq!(world.game_objects[1].rect().point(), vec2(32.0, 32.0));
        assert_eq!(world.game_objects[0].rect().point(), vec2(0.0, 64.0));
        assert_eq!(world.background_offset, vec2(-0.5, 0.0));
    }

    #[test]
    fn initial_direct_and_indirect_attachments_move_with_elevator() {
        let mut world = World::new();
        let mut lift = elevator(vec2(0.0, 96.0), vec2(100.0, 96.0));
        lift.affected_objects = None;
        world.game_objects.push(Box::new(lift));
        add_platform(&mut world, vec2(0.0, 32.0)); // Indirect, above the next platform.
        add_platform(&mut world, vec2(0.0, 64.0));
        add_platform(&mut world, vec2(200.0, 64.0)); // Unrelated.

        world.update(&mut distant_player());

        assert_eq!(world.game_objects[0].rect().left(), 2.0);
        assert_eq!(world.game_objects[1].rect().left(), 2.0);
        assert_eq!(world.game_objects[2].rect().left(), 2.0);
        assert_eq!(world.game_objects[3].rect().left(), 200.0);
    }

    #[test]
    fn initially_attached_movable_is_carried_once_per_update() {
        let mut world = World::new();
        let mut lift = elevator(vec2(0.0, 64.0), vec2(100.0, 64.0));
        lift.affected_objects = None;
        world.game_objects.push(Box::new(lift));
        add_movable(&mut world, vec2(32.0, 32.0));

        for step in 1..=3 {
            world.update(&mut distant_player());
            assert_eq!(world.game_objects[0].rect().left(), step as f32 * 2.0);
            assert_eq!(
                world.game_objects[1].rect().left(),
                32.0 + step as f32 * 2.0
            );
        }
    }

    #[test]
    fn elevator_carries_player_standing_on_an_attached_platform() {
        let mut world = World::new();
        let mut lift = elevator(vec2(0.0, 96.0), vec2(100.0, 96.0));
        lift.affected_objects = None;
        world.game_objects.push(Box::new(lift));
        add_platform(&mut world, vec2(32.0, 64.0));
        let mut player = player_at(vec2(32.0, 32.0));

        world.update(&mut player);

        assert_eq!(world.game_objects[0].rect().left(), 0.0);
        assert_eq!(world.game_objects[1].rect().left(), 32.0);
        assert_eq!(world.background_offset, vec2(-0.5, 0.0));
    }

    #[test]
    fn elevator_animation_tracks_direction_every_three_updates() {
        let mut lift = elevator(Vec2::ZERO, Vec2::ZERO);
        for _ in 0..3 {
            lift.animate(-2.0);
        }
        assert_eq!(lift.texture_index, 1);
        for _ in 0..9 {
            lift.animate(-2.0);
        }
        assert_eq!(lift.texture_index, 2);
        for _ in 0..3 {
            lift.animate(2.0);
        }
        assert_eq!(lift.texture_index, 1);
        for _ in 0..9 {
            lift.animate(2.0);
        }
        assert_eq!(lift.texture_index, 0);
        lift.animate(0.0);
        assert_eq!(lift.texture_index, 1);
    }

    #[test]
    fn level_loader_loads_elevators_and_translates_routes() {
        let mut game = Game::new();
        game.load(include_str!("../../../Canvas/Levels/jstest.xlevel"))
            .unwrap();
        let elevators: Vec<_> = game
            .world
            .game_objects
            .iter()
            .filter_map(|object| object.as_elevator())
            .collect();
        assert_eq!(elevators.len(), 2);
        assert_eq!(elevators[0].start, vec2(-64.0, 272.0));
        assert_eq!(elevators[0].end, vec2(-64.0, 80.0));
        assert_eq!(elevators[0].rect().w, 224.0);

        // Exercise the same object-then-player update sequence as the running game.
        for _ in 0..240 {
            game.world.update(&mut game.player);
            game.player.update(&mut game.world, Vec2::ZERO);
        }
        assert!(game.player.position.is_finite());
        for object in &game.world.game_objects {
            assert!(object.rect().point().is_finite());
        }
    }
}
