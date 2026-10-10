use super::*;

struct TestObject {
    rect: Rect,
    platform: bool,
    movable: bool,
}

impl GameObject for TestObject {
    fn update(&mut self, _context: &UpdateContext<'_>) {}
    fn draw(&self, _assets: &Assets) {}

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

fn add_object(world: &mut World, rect: Rect, platform: bool, movable: bool) {
    world.game_objects.push(Box::new(TestObject {
        rect,
        platform,
        movable,
    }));
}

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
}

#[test]
fn intersection_filters_touching_and_sub_tolerance_overlaps() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    for other in [
        Rect::new(11.0, 0.0, 10.0, 10.0),
        Rect::new(10.0, 0.0, 10.0, 10.0),
        Rect::new(9.95, 0.0, 10.0, 10.0),
        Rect::new(0.0, 9.95, 10.0, 10.0),
    ] {
        assert_eq!(collision_intersection(rect, other), None);
    }
    let minimum = Rect::new(0.0, 0.0, RECT_TOLERANCE, RECT_TOLERANCE);
    assert_eq!(collision_intersection(rect, minimum), Some(minimum));
    assert_eq!(
        collision_intersection(rect, Rect::new(8.0, 7.0, 10.0, 10.0)),
        Some(Rect::new(8.0, 7.0, 2.0, 3.0)),
    );
}

#[test]
fn platform_bounds_match_its_tile_dimensions() {
    let platform = Platform {
        position: vec2(10.0, 20.0),
        width_segments: 3,
        height_segments: 2,
    };
    assert_eq!(platform.rect(), Rect::new(10.0, 20.0, 96.0, 64.0));
    assert!(platform.is_platform());
    assert!(!platform.is_movable());
}

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

        player.update(&mut world, vec2(input, 0.0));
        assert_eq!(player.movement.x, 0.0);
        assert_eq!(player.rotation, 0.0);
        assert_close(world.game_objects[0].rect().x, wall_x);
    }
}

#[test]
fn repeated_updates_keep_player_grounded_and_allow_jump_and_landing() {
    let mut player = Player::new();
    let mut world = World::new();
    world.game_objects.push(Box::new(Platform {
        position: vec2(192.0, 176.0),
        width_segments: 3,
        height_segments: 1,
    }));

    for _ in 0..120 {
        player.update(&mut world, Vec2::ZERO);
        assert!(!player.jumping);
        assert_eq!(player.movement.y, 0.0);
        assert_close(world.game_objects[0].rect().top(), 176.0);
    }

    player.update(&mut world, vec2(0.0, 1.0));
    assert!(player.jumping);
    assert!(player.movement.y > 0.0);
    for _ in 0..120 {
        player.update(&mut world, Vec2::ZERO);
    }
    assert!(!player.jumping);
    assert_eq!(player.movement.y, 0.0);
    assert_close(world.game_objects[0].rect().top(), 176.0);
}

fn movable_at(x: f32, y: f32, move_y: f32) -> MovablePlatform {
    MovablePlatform {
        position: vec2(x, y),
        width_segments: 1,
        height_segments: 1,
        move_y,
    }
}

fn empty_context() -> UpdateContext<'static> {
    UpdateContext {
        player_rect: Rect::new(1000.0, 1000.0, 32.0, 32.0),
        before: &[],
        after: &[],
    }
}

fn distant_player() -> Player {
    let mut player = Player::new();
    player.position = vec2(1000.0, 1000.0);
    player
}

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
        let floor = Box::new(Platform {
            position: vec2(0.0, 64.0),
            width_segments: 1,
            height_segments: 1,
        });
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
