use std::{fmt::Debug, str::FromStr};

use crate::assets::Assets;
use macroquad::prelude::*;

mod elevator;
pub use elevator::Elevator;
mod objects;
use objects::{Magnet, Pickup, PickupKind, Trampoline};

pub enum WorldEffect {
    BounceMovables { surface: Rect, speed: f32 },
}

pub trait GameObject {
    fn update(&mut self, context: &UpdateContext<'_>);
    fn draw(&self, assets: &Assets);
    fn move_world(&mut self, offset: Vec2);
    fn rect(&self) -> Rect;

    fn interact(&mut self, _player: &mut Player) -> Option<WorldEffect> {
        None
    }
    fn set_vertical_speed(&mut self, _speed: f32) {}
    fn is_visible(&self) -> bool {
        true
    }
    fn is_transparent(&self) -> bool {
        true
    }
    fn is_diamond(&self) -> bool {
        false
    }
    fn is_exit(&self) -> bool {
        false
    }

    fn is_platform(&self) -> bool {
        false
    }

    fn is_movable(&self) -> bool {
        false
    }

    fn as_elevator(&self) -> Option<&Elevator> {
        None
    }

    fn as_elevator_mut(&mut self) -> Option<&mut Elevator> {
        None
    }
}

// Read-only access to the player and every object except the one being updated.
pub struct UpdateContext<'a> {
    player_rect: Rect,
    before: &'a [Box<dyn GameObject>],
    after: &'a [Box<dyn GameObject>],
}

impl UpdateContext<'_> {
    fn platform_rects(&self) -> impl Iterator<Item = Rect> + '_ {
        self.before
            .iter()
            .chain(self.after.iter())
            .filter(|object| object.is_platform())
            .map(|object| object.rect())
    }
}

pub struct Player {
    position: Vec2,
    movement: Vec2,
    rotation: f32,
    jumping: bool,
    speed_up_counter: u32,
    alpha: f32,
}

pub struct Game {
    world: World,
    player: Player,
}

// The player lives outside World so both can be borrowed mutably during updates.
pub struct World {
    game_objects: Vec<Box<dyn GameObject>>,
    width: f32,
    height: f32,
    background_offset: Vec2,
}

pub struct Platform {
    position: Vec2,
    width_segments: i32,
    height_segments: i32,
}

pub struct MovablePlatform {
    position: Vec2,
    width_segments: i32,
    height_segments: i32,
    move_y: f32,
}

// Matches rectTolerance in SharedClasses/FPGraphics.m, not the landing tolerance.
const RECT_TOLERANCE: f32 = 0.1;

fn collision_intersection(a: Rect, b: Rect) -> Option<Rect> {
    a.intersect(b)
        .filter(|intersection| intersection.w >= RECT_TOLERANCE && intersection.h >= RECT_TOLERANCE)
}

fn intersects(a: Rect, b: Rect) -> bool {
    a.normalized()
        .intersect(b.normalized())
        .is_some_and(|r| r.w > 0.0 && r.h > 0.0)
}

fn parse_value<T: FromStr>(node: roxmltree::Node<'_, '_>, name: &str) -> T
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

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            player: Player::new(),
        }
    }

    pub fn load(&mut self, xml: &str) -> Result<(), String> {
        let doc = roxmltree::Document::parse(xml).map_err(|error| error.to_string())?;
        if doc.root_element().tag_name().name() != "IronJumpLevel" {
            return Err("Expected an IronJumpLevel XML root".into());
        }
        let mut game = Self::new();
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
                    "FPDiamond" => game
                        .world
                        .game_objects
                        .push(Box::new(Pickup::parse(node, PickupKind::Diamond))),
                    "FPExit" => game
                        .world
                        .game_objects
                        .push(Box::new(Pickup::parse(node, PickupKind::Exit))),
                    "FPSpeedPowerUp" => game
                        .world
                        .game_objects
                        .push(Box::new(Pickup::parse(node, PickupKind::Speed))),
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
        *self = game;
        Ok(())
    }

    pub fn update(&mut self, input: Vec2) {
        self.world.update(&mut self.player);
        self.player.update(&mut self.world, input);
    }

    pub fn draw(&self, assets: &Assets, custom: bool) {
        self.world.draw(assets, custom);
        self.player.draw(assets);
        let (picked, total) = self.diamonds();
        assets.text(
            &format!("Diamonds: {picked}/{total}"),
            3.0,
            0.0,
            Color::new(1.0, 1.0, 1.0, 0.8),
        );
        if self.player.speed_up_counter > 0 {
            let seconds = (Player::MAX_SPEED_UP_COUNT - self.player.speed_up_counter) as f32 / 60.0;
            assets.text(
                &format!("{seconds:.1}"),
                430.0,
                285.0,
                Color::new(0.5, 1.0, 1.0, 0.8),
            );
        }
    }

    pub fn diamonds(&self) -> (usize, usize) {
        let total = self
            .world
            .game_objects
            .iter()
            .filter(|o| o.is_diamond())
            .count();
        let picked = self
            .world
            .game_objects
            .iter()
            .filter(|o| o.is_diamond() && !o.is_visible())
            .count();
        (picked, total)
    }

    pub fn is_complete(&self) -> bool {
        self.world
            .game_objects
            .iter()
            .any(|o| o.is_exit() && !o.is_visible())
    }

    pub fn has_fallen(&self) -> bool {
        !self.world.game_objects.iter().any(|o| {
            o.is_platform() && !o.is_movable() && self.player.rect().top() < o.rect().bottom()
        })
    }
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
    fn platform_collision_left_right(&self, index: usize) -> bool {
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

impl Player {
    const TOLERANCE: f32 = 3.0;
    const MAX_SPEED: f32 = 5.8;
    const SPEED_POWER_UP: f32 = 1.5;
    const UP_SPEED: f32 = 7.0;
    const MAX_FALL_SPEED: f32 = -15.0;
    const ACCELERATION: f32 = 1.1;
    const DECELERATION: f32 = Self::ACCELERATION * 0.2;
    const CHANGE_DIRECTION_SPEED: f32 = 3.0;
    const MAX_SPEED_UP_COUNT: u32 = 60 * 6; // 60 FPS * 6 sec

    pub fn new() -> Self {
        Self {
            position: Vec2 {
                x: 480.0 / 2.0 - 32.0 / 2.0,
                y: 320.0 / 2.0 - 32.0 / 2.0,
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

    fn collision_left_right(&mut self, world: &mut World) -> bool {
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

    pub fn update(&mut self, world: &mut World, input_acceleration: Vec2) {
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

        if input_acceleration.x < 0.0 {
            if self.movement.x < 0.0 {
                self.movement.x +=
                    input_acceleration.x.abs() * Self::ACCELERATION * Self::CHANGE_DIRECTION_SPEED;
            }
            self.movement.x = (self.movement.x + input_acceleration.x.abs() * Self::ACCELERATION)
                .min(current_max_speed);
            move_left_or_right = true;
        } else if input_acceleration.x > 0.0 {
            if self.movement.x > 0.0 {
                self.movement.x -=
                    input_acceleration.x.abs() * Self::ACCELERATION * Self::CHANGE_DIRECTION_SPEED;
            }
            self.movement.x = (self.movement.x - input_acceleration.x.abs() * Self::ACCELERATION)
                .max(-current_max_speed);
            move_left_or_right = true;
        }
        if !self.jumping && input_acceleration.y > 0.0 {
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

    pub fn draw(&self, assets: &Assets) {
        assets
            .ball
            .draw_ex(self.position, WHITE, self.rotation.to_radians(), false);
        if self.speed_up_counter > 0 {
            assets.speed_effect.draw_ex(
                self.position - vec2(16.0, 16.0),
                Color::new(1.0, 1.0, 1.0, self.alpha.sin().abs() * 0.5 + 0.5),
                0.0,
                false,
            );
        }
    }
}

impl Platform {
    pub fn parse(node: roxmltree::Node) -> Self {
        Self {
            position: Vec2 {
                x: parse_value(node, "x"),
                y: parse_value(node, "y"),
            },
            width_segments: parse_value(node, "widthSegments"),
            height_segments: parse_value(node, "heightSegments"),
        }
    }
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

impl GameObject for Platform {
    fn is_transparent(&self) -> bool {
        false
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

    fn update(&mut self, _context: &UpdateContext<'_>) {}

    fn draw(&self, assets: &Assets) {
        for iy in 0..self.height_segments {
            for ix in 0..self.width_segments {
                assets.platform.draw(
                    self.position.x + (ix as f32) * 32.0,
                    self.position.y + (iy as f32) * 32.0,
                );
            }
        }
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
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

    fn draw(&self, assets: &Assets) {
        for iy in 0..self.height_segments {
            for ix in 0..self.width_segments {
                assets.movable.draw(
                    self.position.x + (ix as f32) * 32.0,
                    self.position.y + (iy as f32) * 32.0,
                );
            }
        }
    }

    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }
}

#[cfg(test)]
#[path = "game_tests.rs"]
mod tests;
