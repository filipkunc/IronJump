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

fn player_at(position: Vec2) -> Player {
    let mut player = Player::new();
    player.position = position;
    player
}

fn distant_player() -> Player {
    player_at(vec2(1000.0, 1000.0))
}

fn add_platform(world: &mut World, position: Vec2) {
    world.game_objects.push(Box::new(Platform {
        position,
        width_segments: 1,
        height_segments: 1,
    }));
}

fn add_movable(world: &mut World, position: Vec2) {
    world.game_objects.push(Box::new(MovablePlatform {
        position,
        width_segments: 1,
        height_segments: 1,
        move_y: 0.0,
    }));
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
    game.load(include_str!("../../../../Canvas/Levels/jstest.xlevel"))
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
