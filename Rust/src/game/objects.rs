use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PickupKind {
    Diamond,
    Exit,
    Speed,
}

pub(super) struct Pickup {
    position: Vec2,
    kind: PickupKind,
    visible: bool,
    cooldown: u32,
}

impl Pickup {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>, kind: PickupKind) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            kind,
            visible: true,
            cooldown: 0,
        }
    }
}

impl GameObject for Pickup {
    fn update(&mut self, _context: &UpdateContext<'_>) {}

    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        if self.kind == PickupKind::Speed && self.cooldown > 0 {
            self.cooldown += 1;
            if self.cooldown > Player::MAX_SPEED_UP_COUNT {
                self.cooldown = 0;
                self.visible = true;
            }
        } else if self.visible && intersects(self.rect(), player.rect()) {
            self.visible = false;
            if self.kind == PickupKind::Speed {
                self.cooldown = 1;
                player.speed_up_counter = 1;
            }
        }
        None
    }

    fn draw(&self, assets: &Assets) {
        let sprite = match self.kind {
            PickupKind::Diamond => &assets.diamond,
            PickupKind::Exit => &assets.exit,
            PickupKind::Speed => &assets.speed_power_up,
        };
        sprite.draw(self.position.x, self.position.y);
    }
    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }
    fn rect(&self) -> Rect {
        let size = if self.kind == PickupKind::Exit {
            64.0
        } else {
            32.0
        };
        Rect::new(self.position.x, self.position.y, size, size)
    }
    fn is_visible(&self) -> bool {
        self.visible
    }
    fn is_diamond(&self) -> bool {
        self.kind == PickupKind::Diamond
    }
    fn is_exit(&self) -> bool {
        self.kind == PickupKind::Exit
    }
}

pub(super) struct Trampoline {
    position: Vec2,
    width_segments: i32,
    animation_counter: u8,
    texture_index: i32,
    texture_direction: i32,
}

impl Trampoline {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            width_segments: parse_value(node, "widthSegments"),
            animation_counter: 0,
            texture_index: 0,
            texture_direction: 1,
        }
    }
}

impl GameObject for Trampoline {
    fn update(&mut self, _context: &UpdateContext<'_>) {
        self.animation_counter += 1;
        if self.animation_counter > 5 {
            self.texture_index += self.texture_direction;
            if self.texture_index < 0 || self.texture_index >= 2 {
                self.texture_index -= self.texture_direction;
                self.texture_direction = -self.texture_direction;
            }
            self.animation_counter = 0;
        }
    }
    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        let mut rect = player.rect();
        if !intersects(rect, self.rect()) {
            rect.h += Player::TOLERANCE;
            if intersects(rect, self.rect()) {
                player.movement.y = 9.5;
            }
        }
        Some(WorldEffect::BounceMovables {
            surface: self.rect(),
            speed: 8.0,
        })
    }
    fn draw(&self, assets: &Assets) {
        for segment in 0..self.width_segments {
            assets.trampoline[self.texture_index as usize]
                .draw(self.position.x + segment as f32 * 64.0, self.position.y);
        }
    }
    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width_segments as f32 * 64.0,
            32.0,
        )
    }
    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }
    fn is_platform(&self) -> bool {
        true
    }
}

pub(super) struct Magnet {
    position: Vec2,
    width_segments: i32,
}

impl Magnet {
    pub(super) fn parse(node: roxmltree::Node<'_, '_>) -> Self {
        Self {
            position: vec2(parse_value(node, "x"), parse_value(node, "y")),
            width_segments: parse_value(node, "widthSegments"),
        }
    }
}

impl GameObject for Magnet {
    fn update(&mut self, _context: &UpdateContext<'_>) {}
    fn interact(&mut self, player: &mut Player) -> Option<WorldEffect> {
        let rect = self.rect();
        let attraction = Rect::new(rect.x + 18.0, rect.y + 32.0, rect.w - 36.0, rect.h + 128.0);
        if intersects(attraction, player.rect()) && player.movement.y < 5.0 {
            player.movement.y = player.movement.y * 0.7 + 5.0 * 0.3;
        }
        None
    }
    fn draw(&self, assets: &Assets) {
        for segment in 0..self.width_segments {
            assets
                .magnet
                .draw(self.position.x + segment as f32 * 32.0, self.position.y);
        }
    }
    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width_segments as f32 * 32.0,
            32.0,
        )
    }
    fn move_world(&mut self, offset: Vec2) {
        self.position += offset;
    }
    fn is_platform(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests;
