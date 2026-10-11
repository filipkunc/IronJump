use super::{
    Game, diamond::Diamond, elevator::Elevator, exit::Exit, magnet::Magnet,
    movable_platform::MovablePlatform, platform::Platform, speed_power_up::SpeedPowerUp,
    trampoline::Trampoline,
};
use macroquad::prelude::*;
use std::{fmt::Debug, str::FromStr};

pub(super) fn parse_value<T: FromStr>(node: roxmltree::Node<'_, '_>, name: &str) -> T
where
    T::Err: Debug,
{
    node.children()
        .find(|child| child.has_tag_name(name))
        .expect("Missing value element")
        .text()
        .expect("Missing value text")
        .trim()
        .parse::<T>()
        .expect("Invalid value")
}

fn validate_object(node: roxmltree::Node<'_, '_>) -> Result<(), String> {
    let name = node.tag_name().name();
    let fields: &[&str] = match name {
        "FPPlayer" | "FPDiamond" | "FPExit" | "FPSpeedPowerUp" => &["x", "y"],
        "FPPlatform" | "FPMovablePlatform" => &["x", "y", "widthSegments", "heightSegments"],
        "FPTrampoline" | "FPMagnet" => &["x", "y", "widthSegments"],
        "FPElevator" => &["x", "y", "endX", "endY", "widthSegments"],
        _ => return Err(format!("Unsupported object: {name}")),
    };
    for field in fields {
        let mut matches = node.children().filter(|child| child.has_tag_name(*field));
        let value = matches
            .next()
            .and_then(|child| child.text())
            .ok_or_else(|| format!("Missing {name}.{field}"))?
            .trim();
        if matches.next().is_some() {
            return Err(format!("Duplicate {name}.{field}"));
        }
        let valid = if field.ends_with("Segments") {
            value.parse::<i32>().is_ok_and(|n| n > 0)
        } else {
            value.parse::<f32>().is_ok_and(|n| n.is_finite())
        };
        if !valid {
            return Err(format!("Invalid {name}.{field}: {value}"));
        }
    }
    Ok(())
}

pub(super) fn load(xml: &str) -> Result<Game, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|error| error.to_string())?;
    if doc.root_element().tag_name().name() != "IronJumpLevel" {
        return Err("Expected an IronJumpLevel XML root".into());
    }
    let mut game = Game::new();
    let mut player_offset = Vec2::new(0.0, 0.0);
    let mut players = 0;
    for node in doc.root_element().children() {
        if node.is_element() {
            validate_object(node)?;
            let name = node.tag_name().name();
            match name {
                "FPPlayer" => {
                    player_offset.x = parse_value(node, "x");
                    player_offset.y = parse_value(node, "y");
                    players += 1;
                }
                "FPPlatform" => game
                    .world
                    .game_objects
                    .push(Box::new(Platform::parse(node))),
                "FPMovablePlatform" => game
                    .world
                    .game_objects
                    .push(Box::new(MovablePlatform::parse(node))),
                "FPElevator" => game
                    .world
                    .game_objects
                    .push(Box::new(Elevator::parse(node))),
                "FPDiamond" => game.world.game_objects.push(Box::new(Diamond::parse(node))),
                "FPExit" => game.world.game_objects.push(Box::new(Exit::parse(node))),
                "FPSpeedPowerUp" => game
                    .world
                    .game_objects
                    .push(Box::new(SpeedPowerUp::parse(node))),
                "FPTrampoline" => game
                    .world
                    .game_objects
                    .push(Box::new(Trampoline::parse(node))),
                "FPMagnet" => game.world.game_objects.push(Box::new(Magnet::parse(node))),
                _ => unreachable!("validated object type"),
            }
        }
    }
    if players != 1 {
        return Err("A level must contain exactly one FPPlayer".into());
    }
    game.world.move_world(game.player.position - player_offset);
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::game_with;
    use super::*;

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
}
