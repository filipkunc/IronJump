use std::{default, fmt::Debug, str::FromStr};

use crate::assets::Assets;
use macroquad::prelude::*;

pub trait GameObject {
    fn update(&mut self);
    fn draw(&self, assets: &Assets);
    fn move_world(&mut self, offset: Vec2);
    fn rect(&self) -> Rect;

    fn is_platform(&self) -> bool {
        false
    }

    fn is_movable(&self) -> bool {
        false
    }
}

pub struct Player {
    position: Vec2,
    movement: Vec2,
    rotation: f32,
    jumping: bool,
}

pub struct Game {
    world: World,
    player: Player,
    input_acceleration: Vec2
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

// Matches rectTolerance in SharedClasses/FPGraphics.m, not the landing tolerance.
const RECT_TOLERANCE: f32 = 0.1;

fn collision_intersection(a: Rect, b: Rect) -> Option<Rect> {
    a.intersect(b)
        .filter(|intersection| intersection.w >= RECT_TOLERANCE && intersection.h >= RECT_TOLERANCE)
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
        .parse::<T>()
        .expect("Invalid value")
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            player: Player::new(),
            input_acceleration: Vec2::new(0.0, 0.0)
        }
    }

    pub fn load(&mut self, xml: &String) {
        let mut player_offset = Vec2::new(0.0, 0.0);
        let doc = roxmltree::Document::parse(&xml).expect("Failed to load level");
        for node in doc.root_element().children().into_iter() {
            if node.is_element() {
                if node.tag_name().name() == "FPPlayer" {
                    player_offset.x = parse_value(node, "x");
                    player_offset.y = parse_value(node, "y");
                } else if node.tag_name().name() == "FPPlatform" {
                    self.world
                        .game_objects
                        .push(Box::new(Platform::parse(node)));
                }
            }
        }
        self.world
            .move_world(Vec2::new(240.0 - player_offset.x, 160.0 - player_offset.y));
    }

    fn handle_keyboard(&mut self) {
        self.input_acceleration.x = 0.0;
        self.input_acceleration.y = 0.0;
        if is_key_down(KeyCode::Left) {
            self.input_acceleration.x = -1.0;
        } else if is_key_down(KeyCode::Right) {
            self.input_acceleration.x = 1.0;
        }
        if is_key_down(KeyCode::Up) {
            self.input_acceleration.y = 1.0;
        }
    }

    pub fn update(&mut self) {
        self.handle_keyboard();
        self.world.update();
        self.player.update(&mut self.world, self.input_acceleration);
    }

    pub fn draw(&self, assets: &Assets) {
        self.world.draw(assets);
        self.player.draw(assets);
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

    pub fn update(&mut self) {
        for game_object in self.game_objects.iter_mut() {
            game_object.update();
        }
    }

    pub fn draw(&self, assets: &Assets) {
        const TILE_SIZE: f32 = 32.0;

        let offset_x = self.background_offset.x.rem_euclid(TILE_SIZE) - TILE_SIZE;
        let offset_y = self.background_offset.y.rem_euclid(TILE_SIZE) - TILE_SIZE;
        // One extra tile covers each edge while the background scrolls.
        let columns = (self.width / TILE_SIZE).ceil() as usize + 1;
        let rows = (self.height / TILE_SIZE).ceil() as usize + 1;

        for row in 0..rows {
            for column in 0..columns {
                let x = column as f32 * TILE_SIZE + offset_x;
                let y = row as f32 * TILE_SIZE + offset_y;
                draw_texture(&assets.background, x, y, WHITE);
            }
        }

        for game_object in self.game_objects.iter() {
            game_object.draw(assets);
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
        let current_max_speed = Self::MAX_SPEED;

        if input_acceleration.x < 0.0 {
            if self.movement.x < 0.0 {
                self.movement.x +=
                    input_acceleration.x.abs() * Self::ACCELERATION * Self::CHANGE_DIRECTION_SPEED;
            }
            if self.movement.x < current_max_speed {
                self.movement.x += input_acceleration.x.abs() * Self::ACCELERATION;
            }
            move_left_or_right = true;
        } else if input_acceleration.x > 0.0 {
            if self.movement.x > 0.0 {
                self.movement.x -=
                    input_acceleration.x.abs() * Self::ACCELERATION * Self::CHANGE_DIRECTION_SPEED;
            }
            if self.movement.x > -current_max_speed {
                self.movement.x -= input_acceleration.x.abs() * Self::ACCELERATION;
            }
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
    }

    pub fn draw(&self, assets: &Assets) {
        draw_texture_ex(
            &assets.ball,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                rotation: self.rotation.to_radians(),
                ..DrawTextureParams::default()
            },
        );
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

impl GameObject for Platform {
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

    fn update(&mut self) {}

    fn draw(&self, assets: &Assets) {
        for iy in 0..self.height_segments {
            for ix in 0..self.width_segments {
                draw_texture(
                    &assets.platform,
                    self.position.x + (ix as f32) * 32.0,
                    self.position.y + (iy as f32) * 32.0,
                    WHITE,
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
