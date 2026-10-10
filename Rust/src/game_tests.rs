use super::*;

struct TestObject {
    rect: Rect,
    platform: bool,
    movable: bool,
}

impl GameObject for TestObject {
    fn update(&mut self) {}
    fn draw(&self, _assets: &Assets) {}

    fn moveWorld(&mut self, offset: Vec2) {
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
