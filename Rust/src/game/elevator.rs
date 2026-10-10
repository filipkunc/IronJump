use super::*;

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
mod tests;
