use super::*;

fn game_with(objects: &str) -> Game {
    let xml =
        format!("<IronJumpLevel><FPPlayer><x>0</x><y>0</y></FPPlayer>{objects}</IronJumpLevel>");
    let mut game = Game::new();
    game.load(&xml).unwrap();
    game
}

#[test]
fn diamonds_are_collected_once_and_exit_does_not_require_all_diamonds() {
    let mut game = game_with(
        "<FPDiamond><x>0</x><y>0</y></FPDiamond><FPDiamond><x>500</x><y>0</y></FPDiamond><FPExit><x>0</x><y>0</y></FPExit>",
    );
    assert_eq!(game.diamonds(), (0, 2));
    assert!(!game.is_complete());
    game.world.update(&mut game.player);
    assert_eq!(game.diamonds(), (1, 2));
    assert!(game.is_complete());
    game.world.update(&mut game.player);
    assert_eq!(game.diamonds(), (1, 2));
    assert_eq!(game.world.game_objects.len(), 3); // Stable elevator indices.
}

#[test]
fn speed_pickup_expires_and_respawns_after_360_ticks() {
    let mut game = game_with("<FPSpeedPowerUp><x>0</x><y>0</y></FPSpeedPowerUp>");
    game.world.update(&mut game.player);
    assert_eq!(game.player.speed_up_counter, 1);
    assert!(!game.world.game_objects[0].is_visible());
    // Move off the pickup so expiry cannot immediately re-collect it.
    game.player.position.x += 100.0;
    for _ in 0..359 {
        game.world.update(&mut game.player);
    }
    assert!(!game.world.game_objects[0].is_visible());
    game.world.update(&mut game.player);
    assert!(game.world.game_objects[0].is_visible());

    game.player.speed_up_counter = 1;
    for _ in 0..359 {
        game.player.update(&mut World::new(), Vec2::ZERO);
    }
    assert_eq!(game.player.speed_up_counter, 360);
    game.player.update(&mut World::new(), Vec2::ZERO);
    assert_eq!(game.player.speed_up_counter, 0);
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
                player.update(&mut world, vec2(direction, 0.0));
            }
            assert!((player.movement.x + direction * expected).abs() < 0.0001);
        }
    }
}

#[test]
fn trampoline_launches_player_from_above_but_not_from_inside() {
    let mut game =
        game_with("<FPTrampoline><x>0</x><y>32</y><widthSegments>2</widthSegments></FPTrampoline>");
    assert_eq!(game.world.game_objects[0].rect().w, 128.0);
    game.world.update(&mut game.player);
    assert_eq!(game.player.movement.y, 9.5);
    game.player.position.y += 4.0;
    game.player.movement.y = -2.0;
    game.world.update(&mut game.player);
    assert_eq!(game.player.movement.y, -2.0);
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

#[test]
fn trampoline_requires_more_than_30_pixels_of_movable_overlap() {
    let mut game = game_with(
        "<FPMovablePlatform><x>66</x><y>0</y><widthSegments>1</widthSegments><heightSegments>1</heightSegments></FPMovablePlatform><FPTrampoline><x>0</x><y>32</y><widthSegments>2</widthSegments></FPTrampoline>",
    );
    // Trampoline ends at x=128; position the crate at x=98 for exactly 30px.
    game.world.game_objects[0].move_world(vec2(32.0, 0.0));
    let original_y = game.world.game_objects[0].rect().y;
    game.world.update(&mut game.player);
    assert!((game.world.game_objects[0].rect().y - original_y).abs() < 0.0001);
}

#[test]
fn magnet_attracts_only_in_its_field_and_below_target_speed() {
    let doc = roxmltree::Document::parse(
        "<FPMagnet><x>0</x><y>0</y><widthSegments>2</widthSegments></FPMagnet>",
    )
    .unwrap();
    let mut magnet = Magnet::parse(doc.root_element());
    let mut player = Player::new();
    player.position = vec2(20.0, 64.0);
    player.movement.y = -5.0;
    magnet.interact(&mut player);
    assert_eq!(player.movement.y, -2.0);
    player.movement.y = 6.0;
    magnet.interact(&mut player);
    assert_eq!(player.movement.y, 6.0);
    player.position = vec2(80.0, 64.0);
    player.movement.y = -5.0;
    magnet.interact(&mut player);
    assert_eq!(player.movement.y, -5.0);
}

#[test]
fn loading_replaces_old_state_and_rejects_invalid_levels_without_mutation() {
    let mut game = game_with("<FPDiamond><x>0</x><y>0</y></FPDiamond>");
    game.world.update(&mut game.player);
    assert_eq!(game.diamonds(), (1, 1));
    let malformed = "<IronJumpLevel><FPPlayer><x>NaN</x><y>0</y></FPPlayer></IronJumpLevel>";
    assert!(game.load(malformed).is_err());
    assert_eq!(game.diamonds(), (1, 1));
    game.load("<IronJumpLevel><FPPlayer><x>0</x><y>0</y></FPPlayer></IronJumpLevel>")
        .unwrap();
    assert_eq!(game.diamonds(), (0, 0));
    assert_eq!(game.player.speed_up_counter, 0);
    assert!(!game.is_complete()); // No exit is not a victory.
}

#[test]
fn every_original_level_loads_all_objects_and_simulates_without_invalid_numbers() {
    let mut total = 0;
    for level in crate::levels::bundled() {
        let doc = roxmltree::Document::parse(&level.xml).unwrap();
        let objects = doc
            .root_element()
            .children()
            .filter(|node| node.is_element())
            .count();
        let mut game = Game::new();
        game.load(&level.xml)
            .unwrap_or_else(|error| panic!("{}: {error}", level.name));
        assert_eq!(
            game.world.game_objects.len() + 1,
            objects,
            "{} lost objects",
            level.name
        );
        total += objects;
        for frame in 0..300 {
            let x = if frame < 100 {
                1.0
            } else if frame < 200 {
                -1.0
            } else {
                0.0
            };
            game.update(vec2(x, if frame % 60 < 10 { 1.0 } else { 0.0 }));
            assert!(game.player.movement.is_finite(), "{}", level.name);
            for object in &game.world.game_objects {
                assert!(object.rect().point().is_finite(), "{}", level.name);
            }
        }
    }
    assert_eq!(total, 935);
}
